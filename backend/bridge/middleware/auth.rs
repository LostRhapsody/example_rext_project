use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

use crate::{
    bridge::types::auth::AuthUser,
    control::services::token_service::TokenService,
    infrastructure::app_error::AppError,
};

// JWT middleware
pub async fn auth_middleware(mut request: Request, next: Next) -> Result<Response, AppError> {
    // Extract and validate token using TokenService
    let user_id = TokenService::extract_and_validate_token(&request)?;

    // Add user to request extensions
    request.extensions_mut().insert(AuthUser {
        user_id,
    });

    Ok(next.run(request).await)
}
