use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use sea_orm::{DatabaseConnection, EntityTrait};
use tracing::{error, info, warn};

use crate::{
    bridge::types::auth::AuthUser,
    control::services::database_service::DatabaseService,
    control::services::token_service::TokenService,
    entity::models::{roles, users},
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

    // Check if user has admin privileges by checking their role
    let user = DatabaseService::find_one_with_tracking(
        &db,
        "users",
        users::Entity::find_by_id(user_id),
    )
    .await
    .map_err(|e| {
        error!(request_id = %request_id, error = ?e, "Database error checking admin status");
        AppError {
            message: "Failed to verify admin status".to_string(),
            status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?
    .ok_or(AppError {
        message: "Failed to verify admin status".to_string(),
        status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    // Check if user has admin role
    let has_admin_permissions = if let Some(role_id) = user.role_id {
        let role = roles::Entity::find_by_id(role_id)
            .one(&db)
            .await
            .map_err(|e| {
                error!(request_id = %request_id, error = ?e, "Database error checking role");
                AppError {
                    message: "Failed to verify role".to_string(),
                    status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                }
            })?;

        if let Some(role_model) = role {
            let permissions: Vec<String> =
                serde_json::from_str(&role_model.permissions).unwrap_or_else(|_| vec![]);
            permissions.contains(&"*".to_string())
        } else {
            false
        }
    } else {
        false
    };

    if !has_admin_permissions {
        return Err(AppError {
            message: "Access denied: Admin privileges required".to_string(),
            status_code: axum::http::StatusCode::FORBIDDEN,
        });
    }

    info!(
        request_id = %request_id,
        user_id = %user_id,
        email = %user.email,
        "Admin access granted"
    );

    // Add both AuthUser and AdminUser to request extensions for downstream handlers
    request.extensions_mut().insert(AuthUser { user_id });

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
