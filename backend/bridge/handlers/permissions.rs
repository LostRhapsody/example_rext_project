//! Permission handlers
//! client connection functions to manage permissions through an API.
//!
//! TOOD implement this on the router.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{
    bridge::types::admin::ADMIN_TAG,
    bridge::types::auth::AuthUser,
    control::services::permission_service::PermissionService,
    domain::permissions::Permission,
    infrastructure::app_error::{AppError, ErrorResponse},
};

/// Get user permissions endpoint
#[utoipa::path(
    get,
    path = "/permissions/user/{user_id}",
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User permissions retrieved successfully", body = UserPermissionsResponse),
        (status = 400, description = "Bad request - invalid user ID", body = ErrorResponse),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Get user permissions",
    description = "Retrieves all permissions for a specific user",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
#[allow(dead_code)]
pub async fn get_user_permissions_handler(
    State(db): State<DatabaseConnection>,
    Path(user_id): Path<String>,
    _auth_user: AuthUser, // This ensures the user is authenticated
) -> Result<impl IntoResponse, AppError> {
    let user_id = Uuid::parse_str(&user_id).map_err(|_| AppError {
        message: "Invalid user ID format".to_string(),
        status_code: StatusCode::BAD_REQUEST,
    })?;

    let permissions = PermissionService::get_user_permissions(&db, user_id).await?;

    let response = UserPermissionsResponse {
        user_id: user_id.to_string(),
        permissions: permissions.to_strings(),
        permission_count: permissions.to_vec().len(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Check specific permission endpoint
#[utoipa::path(
    post,
    path = "/permissions/check/{user_id}",
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    request_body = CheckPermissionRequest,
    responses(
        (status = 200, description = "Permission check completed", body = CheckPermissionResponse),
        (status = 400, description = "Bad request - invalid user ID", body = ErrorResponse),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Check specific permission",
    description = "Checks if a user has a specific permission",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
#[allow(dead_code)]
pub async fn check_specific_permission_handler(
    State(db): State<DatabaseConnection>,
    Path(user_id): Path<String>,
    Json(payload): Json<CheckPermissionRequest>,
    _auth_user: AuthUser, // This ensures the user is authenticated
) -> Result<impl IntoResponse, AppError> {
    let user_id = Uuid::parse_str(&user_id).map_err(|_| AppError {
        message: "Invalid user ID format".to_string(),
        status_code: StatusCode::BAD_REQUEST,
    })?;

    let permission = Permission::from_string(&payload.permission);
    let has_permission = PermissionService::has_permission(&db, user_id, &permission).await?;

    let response = CheckPermissionResponse {
        user_id: user_id.to_string(),
        permission: payload.permission,
        has_permission,
        permission_description: permission.description().to_string(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Get all available permissions endpoint
#[utoipa::path(
    get,
    path = "/permissions/available",
    responses(
        (status = 200, description = "Available permissions retrieved successfully", body = AvailablePermissionsResponse),
        (status = 401, description = "Unauthorized - authentication required", body = ErrorResponse),
        (status = 403, description = "Forbidden - admin privileges required", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    summary = "Get available permissions",
    description = "Retrieves all available permissions in the system",
    tag = ADMIN_TAG,
    security(
        ("jwt_token" = [])
    )
)]
#[allow(dead_code)]
pub async fn get_available_permissions_handler(
    _auth_user: AuthUser, // This ensures the user is authenticated
) -> Result<impl IntoResponse, AppError> {
    let permissions = PermissionService::get_all_permissions();
    let categories = PermissionService::get_permissions_by_category();

    let response = AvailablePermissionsResponse {
        permissions: permissions
            .iter()
            .map(|p| PermissionInfo {
                name: p.to_string(),
                description: p.description().to_string(),
                category: p.category().to_string(),
            })
            .collect(),
        categories: categories.keys().cloned().collect(),
        total_count: permissions.len(),
    };

    Ok((StatusCode::OK, Json(response)))
}

// Response types
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct UserPermissionsResponse {
    pub user_id: String,
    pub permissions: Vec<String>,
    pub permission_count: usize,
}

#[derive(Deserialize, ToSchema)]
pub struct CheckPermissionRequest {
    #[allow(dead_code)]
    pub permission: String,
}

#[derive(Serialize, ToSchema)]
pub struct CheckPermissionResponse {
    pub user_id: String,
    pub permission: String,
    pub has_permission: bool,
    pub permission_description: String,
}

#[derive(Serialize, ToSchema)]
pub struct AvailablePermissionsResponse {
    pub permissions: Vec<PermissionInfo>,
    pub categories: Vec<String>,
    pub total_count: usize,
}

#[derive(Serialize, ToSchema)]
pub struct PermissionInfo {
    pub name: String,
    pub description: String,
    pub category: String,
}
