use jsonwebtoken::{EncodingKey, Header, encode};
use sea_orm::*;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::control::services::user_service::UserService;
use crate::domain::{auth::*, user::*, validation::*};
use crate::infrastructure::app_error::AppError;
use crate::infrastructure::jwt_claims::Claims;
use axum::http::StatusCode;

/// Service for authentication-related business operations
pub struct AuthService;

impl AuthService {
    /// Authenticates a user and returns a JWT token
    pub async fn authenticate_user(
        db: &DatabaseConnection,
        login: UserLogin,
    ) -> Result<AuthToken, AppError> {
        // Validate input
        validate_login_input(&login.email, &login.password)?;

        // Find user by email
        let user = UserService::find_user_by_email(db, &login.email)
            .await?
            .ok_or(AppError {
                message: "Invalid credentials".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            })?;

        // Verify password
        let is_valid = UserService::verify_password(&user, &login.password)?;
        if !is_valid {
            return Err(AppError {
                message: "Invalid credentials".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            });
        }

        // Update last login timestamp (non-blocking)
        let db_clone = db.clone();
        let user_id = user.id;
        tokio::spawn(async move {
            let _ = UserService::update_last_login(&db_clone, user_id).await;
        });

        // Generate JWT token
        let token = Self::generate_jwt_token(&user.id)?;

        Ok(token)
    }

    /// Generates a JWT token for a user
    fn generate_jwt_token(user_id: &uuid::Uuid) -> Result<AuthToken, AppError> {
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
        let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());

        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 24 * 60 * 60; // 24 hours

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration as usize,
        };

        let token_string =
            encode(&Header::default(), &claims, &encoding_key).map_err(|_| AppError {
                message: "Failed to generate token".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        let expires_at = chrono::DateTime::from_timestamp(expiration as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now() + chrono::Duration::hours(24));

        let auth_token = AuthToken::new(token_string, *user_id, expires_at);

        Ok(auth_token)
    }
}
