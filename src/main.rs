mod entities;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    Json, Router,
    extract::{Request, State},
    http::{Method, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use entities::{prelude::*, *};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand_core::OsRng;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use tower_http::cors::{Any, CorsLayer};
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(register_handler),
    components(
        schemas(RegisterRequest, RegisterResponse, MessageResponse, ErrorResponse)
    )
)]
struct ApiDoc;

// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // subject (user id)
    exp: usize,  // expiration time
}

// Request/Response types
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct RegisterRequest {
    /// User's email address
    #[schema(example = "user@example.com")]
    email: String,

    /// User's password
    #[schema(example = "securepassword123")]
    password: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct RegisterResponse {
    /// Success message
    #[schema(example = "User created successfully")]
    message: String,

    /// The newly created user's ID
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    user_id: String,

    /// User's email address
    #[schema(example = "user@example.com")]
    email: String,

    /// Timestamp when the user was created (ISO 8601 format)
    #[schema(example = "2024-01-20T15:30:00Z", format = "date-time", nullable = true)]
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<String>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct MessageResponse {
    /// Response message
    #[schema(example = "Operation completed successfully")]
    message: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    /// Error message describing what went wrong
    #[schema(example = "Email and password are required")]
    message: String,
}

#[derive(Serialize)]
struct ProfileResponse {
    id: String,
    email: String,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

// Custom error type
#[derive(Debug)]
struct AppError {
    message: String,
    status_code: StatusCode,
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
struct AuthUser {
    user_id: uuid::Uuid,
}

// JWT middleware
async fn auth_middleware(mut request: Request, next: Next) -> Result<Response, AppError> {
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

// Handler functions
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
    tag = "Authentication"
)]
async fn register_handler(
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

async fn login_handler(
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

async fn logout_handler() -> impl IntoResponse {
    // Since JWT is stateless, we can't really invalidate it on the server side
    // In a production app, you might maintain a blacklist of tokens
    // For now, we just return a success message
    Json(MessageResponse {
        message: "Logged out successfully".to_string(),
    })
}

async fn profile_handler(
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

async fn root_handler() -> &'static str {
    "Hello from Axum server with authentication!"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Get database URL from environment
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    // Connect to database
    let db: DatabaseConnection = Database::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("Connected to database: {}", database_url);

    // Create CORS layer for development
    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:5173"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    // Create router with authentication routes
    let app = Router::new()
        // TODO: Add swagger ui
        // .merge(SwaggerUi::new("/swagger-ui")
        // .url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", get(root_handler))
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
        .route(
            "/profile",
            get(profile_handler).layer(middleware::from_fn(auth_middleware)),
        )
        .layer(cors)
        .with_state(db);

    // Start the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    println!("Server running on http://127.0.0.1:3000");
    println!("Endpoints:");
    println!("  POST /register - Register a new user");
    println!("  POST /login    - Login and get JWT token");
    println!("  POST /logout   - Logout (placeholder)");
    println!("  GET  /profile  - Get user profile (requires authentication)");
    println!("{}", ApiDoc::openapi().to_pretty_json().unwrap());

    axum::serve(listener, app).await?;

    Ok(())
}
