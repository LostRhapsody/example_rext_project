use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tracing::{error, info, warn};

use crate::{
    bridge::types::auth::AuthUser,
    control::services::token_service::TokenService,
    entity::models::users,
    infrastructure::{app_error::AppError, logging::LoggingManager},
};

/// Admin middleware that handles JWT extraction and admin verification
pub async fn admin_middleware(
    State(db): State<DatabaseConnection>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let request_id = LoggingManager::generate_request_id();

    // Extract and validate token using TokenService
    let user_id = TokenService::extract_and_validate_token(&request)?;

    // Check if user has admin privileges
    let user = users::Entity::find_by_id(user_id)
        .filter(users::Column::IsAdmin.eq(true))
        .one(&db)
        .await
        .map_err(|e| {
            error!(request_id = %request_id, error = ?e, "Database error checking admin status");
            AppError {
                message: "Failed to verify admin status".to_string(),
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?
        .ok_or(AppError {
            message: "Access denied: Admin privileges required".to_string(),
            status_code: axum::http::StatusCode::FORBIDDEN,
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