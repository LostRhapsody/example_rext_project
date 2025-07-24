use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::Serialize;
use utoipa::ToSchema;

// Custom error type
#[derive(Debug)]
pub struct AppError {
    pub message: String,
    pub status_code: StatusCode,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            message: self.message,
        });
        (self.status_code, body).into_response()
    }
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponse {
    /// Response message
    #[schema(example = "Operation completed successfully")]
    pub message: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// Error message describing what went wrong
    #[schema(example = "Email and password are required")]
    pub message: String,
}