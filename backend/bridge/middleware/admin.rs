use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};

use crate::{
    bridge::types::auth::AuthUser,
    entity::models::users,
    infrastructure::{app_error::AppError, jwt_claims::Claims, logging::LoggingManager},
};

/// Admin middleware that handles JWT extraction and admin verification
pub async fn admin_middleware(
    State(db): State<DatabaseConnection>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let request_id = LoggingManager::generate_request_id();

    // Extract JWT token from Authorization header
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

    // Verify JWT token
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

    // Extract user ID from token
    let user_id = uuid::Uuid::parse_str(&token_data.claims.sub).unwrap();

    // Check if user has admin privileges
    let user = users::Entity::find_by_id(user_id)
        .filter(users::Column::IsAdmin.eq(true))
        .one(&db)
        .await
        .map_err(|e| {
            error!(request_id = %request_id, error = ?e, "Database error checking admin status");
            AppError {
                message: "Failed to verify admin status".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?
        .ok_or(AppError {
            message: "Access denied: Admin privileges required".to_string(),
            status_code: StatusCode::FORBIDDEN,
        })?;

    info!(
        request_id = %request_id,
        user_id = %user_id,
        email = %user.email,
        "Admin access granted"
    );

    // Add both AuthUser and AdminUser to request extensions for downstream handlers
    request.extensions_mut().insert(AuthUser {
        user_id,
    });
    
    request.extensions_mut().insert(AdminUser {
        user_id,
        email: user.email.clone(),
    });

    Ok(next.run(request).await)
}



/// Admin user information for downstream handlers
#[allow(dead_code)]
#[derive(Clone)]
pub struct AdminUser {
    pub user_id: uuid::Uuid,
    pub email: String,
}

/// Log admin access attempts (for security monitoring)
pub fn _log_admin_access_attempt(
    user_id: Option<uuid::Uuid>,
    success: bool,
    path: &str,
    ip_address: Option<&str>,
) {
    if success {
        info!(
            user_id = ?user_id,
            path = %path,
            ip_address = ?ip_address,
            "Admin access granted"
        );
    } else {
        warn!(
            user_id = ?user_id,
            path = %path,
            ip_address = ?ip_address,
            "Admin access denied"
        );
    }
}