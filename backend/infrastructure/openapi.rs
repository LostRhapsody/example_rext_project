use utoipa::OpenApi;

use crate::bridge::types::auth::{AUTH_TAG, RegisterRequest, RegisterResponse};
use crate::bridge::types::admin::{
    ADMIN_TAG, AdminLoginRequest, AdminLoginResponse, AuditLogResponse,
    LogsQueryParams, UsersQueryParams, CreateUserRequest, UpdateUserRequest, UserResponse,
    DatabaseTableResponse, TableRecordsQueryParams, TableRecordResponse, HealthResponse
};
use crate::infrastructure::app_error::{ErrorResponse, MessageResponse};

/// OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = AUTH_TAG, description = "Authentication endpoints"),
        (name = ADMIN_TAG, description = "Admin panel endpoints")
    ),
    components(
        schemas(
            RegisterRequest, RegisterResponse, MessageResponse, ErrorResponse,
            AdminLoginRequest, AdminLoginResponse, AuditLogResponse,
            LogsQueryParams, UsersQueryParams, CreateUserRequest, UpdateUserRequest, UserResponse,
            DatabaseTableResponse, TableRecordsQueryParams, TableRecordResponse, HealthResponse
        )
    )
)]
pub struct ApiDoc;
