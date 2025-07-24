use argon2::{
    password_hash::{
        PasswordHasher,
        PasswordVerifier,
        PasswordHash,
        SaltString,
    },
    Argon2,
};
use axum::{
    Json,
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
};
use jsonwebtoken::{EncodingKey, Header, encode};
use sea_orm::*;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::entity::models::{prelude::*, *};
use crate::infrastructure::app_error::{AppError, ErrorResponse, MessageResponse};
use crate::infrastructure::jwt_claims::Claims;
use crate::bridge::types::auth::{RegisterRequest, RegisterResponse, LoginRequest, LoginResponse, ProfileResponse, AuthUser, AUTH_TAG};

/// Registers a new user
#[utoipa::path(
    post,
    path = "/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User created successfully", body = RegisterResponse),
        (status = 400, description = "Bad request - validation errors", body = ErrorResponse, examples(
            ("empty_fields" = (value = json!({"message": "Email and password are required"}))),
        )),
        (status = 409, description = "Conflict - user already exists", body = ErrorResponse, examples(
            ("user_exists" = (value = json!({"message": "User already exists"})))
        )),
        (status = 500, description = "Internal server error", body = ErrorResponse, examples(
            ("hash_error" = (value = json!({"message": "Failed to hash password"}))),
            ("database_error" = (value = json!({"message": "Failed to create user"})))
        ))
    ),
    summary = "Register a new user",
    description = "Creates a new user account with email and password. Password is securely hashed using Argon2.",
    tag = AUTH_TAG
)]
pub async fn register_handler(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(AppError {
            message: "Email and password are required".to_string(),
            status_code: StatusCode::BAD_REQUEST,
        });
    }

    if payload.password.len() < 6 {
        return Err(AppError {
            message: "Password must be at least 6 characters".to_string(),
            status_code: StatusCode::BAD_REQUEST,
        });
    }

    let existing_user: Option<users::Model> = Users::find()
        .filter(users::Column::Email.eq(payload.email.clone()))
        .one(&db)
        .await
        .unwrap();

    if existing_user.is_some() {
        return Err(AppError {
            message: "User already exists".to_string(),
            status_code: StatusCode::CONFLICT,
        });
    }

    // Hash password
    let salt = SaltString::generate(&mut rand_core::OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| AppError {
            message: "Failed to hash password".to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?
        .to_string();

    // Create user
    let user_id = uuid::Uuid::new_v4();
    let created_at = chrono::Utc::now().fixed_offset();
    let new_user = users::ActiveModel {
        id: Set(user_id),
        email: Set(payload.email.clone()),
        password_hash: Set(password_hash),
        created_at: Set(Some(created_at)),
    };

    Users::insert(new_user)
        .exec(&db)
        .await
        .map_err(|_| AppError {
            message: "Failed to create user".to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    // Return comprehensive response
    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            message: "User created successfully".to_string(),
            user_id: user_id.to_string(),
            email: payload.email,
            created_at: Some(created_at.to_utc().to_rfc3339()),
        }),
    ))
}

/// Logs in an existing user
#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 400, description = "Bad request - validation errors", body = ErrorResponse, examples(
            ("empty_fields" = (value = json!({"message": "Email and password are required"}))),
        )),
        (status = 401, description = "Unauthorized - invalid credentials", body = ErrorResponse, examples(
            ("invalid_credentials" = (value = json!({"message": "Invalid credentials"})))
        )),
        (status = 500, description = "Internal server error", body = ErrorResponse, examples(
            ("hash_error" = (value = json!({"message": "Invalid password hash"}))),
            ("token_error" = (value = json!({"message": "Failed to generate token"})))
        ))
    ),
    summary = "Login user",
    description = "Authenticates a user with email and password, returns a JWT token on success.",
    tag = AUTH_TAG
)]
pub async fn login_handler(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user: Option<users::Model> = Users::find()
        .filter(users::Column::Email.eq(payload.email.clone()))
        .one(&db)
        .await
        .unwrap();

    let user = match user {
        Some(user) => user,
        None => {
            return Err(AppError {
                message: "Invalid credentials".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            });
        }
    };

    // Verify password
    let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|_| AppError {
        message: "Invalid password hash".to_string(),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    let argon2 = Argon2::default();
    if argon2
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err(AppError {
            message: "Invalid credentials".to_string(),
            status_code: StatusCode::UNAUTHORIZED,
        });
    }

    // Generate JWT
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
    let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());

    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 24 * 60 * 60; // 24 hours

    let claims = Claims {
        sub: user.id.to_string(),
        exp: expiration as usize,
    };

    let token = encode(&Header::default(), &claims, &encoding_key).map_err(|_| AppError {
        message: "Failed to generate token".to_string(),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    Ok(Json(LoginResponse { token }))
}

/// Logs out the current user
#[utoipa::path(
    post,
    path = "/logout",
    responses(
        (status = 200, description = "Logout successful", body = MessageResponse, examples(
            ("success" = (value = json!({"message": "Logged out successfully"})))
        ))
    ),
    summary = "Logout user",
    description = "Logs out the current user. Since JWT is stateless, this just returns a success message.",
    tag = AUTH_TAG
)]
pub async fn logout_handler() -> impl IntoResponse {
    // Since JWT is stateless, we can't really invalidate it on the server side
    // In a production app, you might maintain a blacklist of tokens
    // For now, we just return a success message
    Json(MessageResponse {
        message: "Logged out successfully".to_string(),
    })
}

/// Gets the current user's profile information
#[utoipa::path(
    get,
    path = "/profile",
    responses(
        (status = 200, description = "Profile retrieved successfully", body = ProfileResponse),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse, examples(
            ("not_authenticated" = (value = json!({"message": "User not authenticated"})))
        )),
        (status = 404, description = "Not found - user not found", body = ErrorResponse, examples(
            ("user_not_found" = (value = json!({"message": "User not found"})))
        )),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Get user profile",
    description = "Retrieves the authenticated user's profile information including ID, email, and creation timestamp.",
    tag = AUTH_TAG,
    security(
        ("jwt_token" = [])
    )
)]
pub async fn profile_handler(
    State(db): State<DatabaseConnection>,
    request: Request,
) -> Result<impl IntoResponse, AppError> {
    // Extract user from request extensions (set by middleware)
    let auth_user = request.extensions().get::<AuthUser>().ok_or(AppError {
        message: "User not authenticated".to_string(),
        status_code: StatusCode::UNAUTHORIZED,
    })?;

    // Find user in database
    let user: Option<users::Model> = Users::find_by_id(auth_user.user_id).one(&db).await.unwrap();

    let user = match user {
        Some(user) => user,
        None => {
            return Err(AppError {
                message: "User not found".to_string(),
                status_code: StatusCode::NOT_FOUND,
            });
        }
    };

    Ok(Json(ProfileResponse {
        id: user.id.to_string(),
        email: user.email,
        created_at: user.created_at.map(|dt| dt.to_utc()),
    }))
}
