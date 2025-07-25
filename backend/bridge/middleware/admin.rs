use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tracing::{error, info, warn};

use crate::{
    bridge::types::auth::AuthUser,
    entity::models::users,
    infrastructure::{app_error::AppError, logging::LoggingManager},
};

/// Admin authentication middleware that requires admin privileges
pub async fn _admin_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let request_id = LoggingManager::generate_request_id();

    // Extract user from request extensions (set by auth middleware)
    let auth_user = request.extensions().get::<AuthUser>().ok_or(AppError {
        message: "User not authenticated".to_string(),
        status_code: StatusCode::UNAUTHORIZED,
    })?;

    // Get database connection from request extensions
    let db = request.extensions().get::<DatabaseConnection>().ok_or(AppError {
        message: "Database connection not available".to_string(),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    // Check if user has admin privileges
    let user = users::Entity::find_by_id(auth_user.user_id)
        .filter(users::Column::IsAdmin.eq(true))
        .one(db)
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
        user_id = %auth_user.user_id,
        email = %user.email,
        "Admin access granted"
    );

    // Add admin user info to request extensions for downstream handlers
    let user_id = auth_user.user_id;
    let email = user.email.clone();
    request.extensions_mut().insert(_AdminUser {
        user_id,
        email,
    });

    Ok(next.run(request).await)
}

/// Admin user information for downstream handlers
#[derive(Clone)]
pub struct _AdminUser {
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