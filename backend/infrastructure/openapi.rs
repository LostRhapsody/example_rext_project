use utoipa::OpenApi;

use crate::bridge::types::auth::{AUTH_TAG, RegisterRequest, RegisterResponse};
use crate::infrastructure::app_error::{ErrorResponse, MessageResponse};

/// OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = AUTH_TAG, description = "Authentication endpoints")
    ),
    components(
        schemas(RegisterRequest, RegisterResponse, MessageResponse, ErrorResponse)
    )
)]
pub struct ApiDoc;
