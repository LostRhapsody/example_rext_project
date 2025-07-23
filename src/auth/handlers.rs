use axum::{
    Json,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use sea_orm::*;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand_core::OsRng;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entities::{prelude::*, *};
use crate::AUTH_TAG;

// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // subject (user id)
    pub exp: usize,  // expiration time
}

// Request/Response types
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    /// User's email address
    #[schema(example = "user@example.com")]
    pub email: String,

    /// User's password
    #[schema(example = "securepassword123")]
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    /// Success message
    #[schema(example = "User created successfully")]
    pub message: String,

    /// The newly created user's ID
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub user_id: String,

    /// User's email address
    #[schema(example = "user@example.com")]
    pub email: String,

    /// Timestamp when the user was created (ISO 8601 format)
    #[schema(example = "2024-01-20T15:30:00Z", format = "date-time", nullable = true)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponse {
    /// Response message
    #[schema(example = "Operation completed successfully")]
    pub message: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// Error message describing what went wrong
    #[schema(example = "Email and password are required")]
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct ProfileResponse {
    pub id: String,
    pub email: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

// Custom error type
#[derive(Debug)]
pub struct AppError {
    pub message: String,
    pub status_code: StatusCode,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            message: self.message,
        });
        (self.status_code, body).into_response()
    }
}

// JWT token extractor
#[derive(Clone)]
pub struct AuthUser {
    pub user_id: uuid::Uuid,
}

// JWT middleware
pub async fn auth_middleware(mut request: Request, next: Next) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = match auth_header {
        Some(header) => header,
        None => {
            return Err(AppError {
                message: "Missing Authorization header".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            });
        }
    };

    let token = match auth_header.strip_prefix("Bearer ") {
        Some(token) => token,
        None => {
            return Err(AppError {
                message: "Invalid Authorization header format".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            });
        }
    };

    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
    let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());

    let token_data = match decode::<Claims>(token, &decoding_key, &Validation::default()) {
        Ok(data) => data,
        Err(_) => {
            return Err(AppError {
                message: "Invalid token".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            });
        }
    };

    // Check if token is expired
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    if token_data.claims.exp < current_time {
        return Err(AppError {
            message: "Token expired".to_string(),
            status_code: StatusCode::UNAUTHORIZED,
        });
    }

    // Add user to request extensions
    request.extensions_mut().insert(AuthUser {
        user_id: uuid::Uuid::parse_str(&token_data.claims.sub).unwrap(),
    });

    Ok(next.run(request).await)
}

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
    let salt = SaltString::generate(&mut OsRng);
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
    description = "Logs out the current user. Since JWT is stateless, this just returns a success message. In production, you might maintain a token blacklist.",
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