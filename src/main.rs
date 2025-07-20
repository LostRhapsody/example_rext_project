mod entities;
use entities::{prelude::*, *};
use axum::{
    extract::{Request, State},
    http::{header, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use tower_http::cors::{CorsLayer, Any};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand_core::OsRng;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,  // subject (user id)
    exp: usize,   // expiration time
}

// Request/Response types
#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
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

#[derive(Serialize)]
struct MessageResponse {
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
        let body = Json(MessageResponse {
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
async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
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
    let new_user = users::ActiveModel {
        id: Set(user_id),
        email: Set(payload.email.clone()),
        password_hash: Set(password_hash),
        created_at: Set(Some(chrono::Utc::now().fixed_offset())),
    };

    Users::insert(new_user).exec(&db).await.map_err(|_| AppError {
        message: "Failed to create user".to_string(),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    Ok((
        StatusCode::CREATED,
        Json(MessageResponse {
            message: "User created successfully".to_string(),
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
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    // Connect to database
    let db: DatabaseConnection = Database::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("Connected to database: {}", database_url);

    // Create CORS layer for development
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    // Create router with authentication routes
    let app = Router::new()
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

    axum::serve(listener, app).await?;

    Ok(())
}

