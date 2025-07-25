use axum::{body::{to_bytes, Body}, extract::{Request, State}, http::StatusCode, middleware::Next, response::Response};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use std::time::Instant;
use tracing::{error, info};

use crate::{
    bridge::types::auth::AuthUser,
    entity::models::audit_logs,
    infrastructure::logging::LoggingManager,
};

const MAX_BODY_LOG_BYTES: usize = 4096; // 4KB

/// Request logging middleware for auditing all API requests
pub async fn request_logging_middleware(
    State(db): State<DatabaseConnection>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let request_id = LoggingManager::generate_request_id();

    // Extract request info
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let ip_address = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| request.extensions().get::<std::net::SocketAddr>().map(|addr| addr.ip().to_string()));
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Try to get user_id from extensions (set by auth middleware)
    let user_id = request
        .extensions()
        .get::<AuthUser>()
        .map(|u| u.user_id);

    // Capture request body up to MAX_BODY_LOG_BYTES
    let (parts, body) = request.into_parts();
    let body_bytes = to_bytes(body, MAX_BODY_LOG_BYTES).await.unwrap_or_default();
    let truncated = &body_bytes[..]; // Already limited by to_bytes
    let request_body = Some(String::from_utf8_lossy(truncated).to_string());
    let request = Request::from_parts(parts, Body::from(body_bytes));

    // Run the next handler and capture response
    let result = next.run(request).await;
    let duration = start.elapsed();
    let response_time_ms = duration.as_millis() as i32;
    let status_code = result.status().as_u16() as i32;

    // Capture response body up to MAX_BODY_LOG_BYTES
    let (parts, body) = result.into_parts();
    let body_bytes = to_bytes(body, MAX_BODY_LOG_BYTES).await.unwrap_or_default();
    let truncated = &body_bytes[..]; // Already limited by to_bytes
    let response_body = Some(String::from_utf8_lossy(truncated).to_string());
    let result = Response::from_parts(parts, Body::from(body_bytes));

    // Error message if status is error
    let error_message = if status_code >= 400 {
        Some(format!("Error status: {}", status_code))
    } else {
        None
    };

    // Clone values needed after move
    let method_clone = method.clone();
    let path_clone = path.clone();
    let ip_address_clone = ip_address.clone();
    let user_agent_clone = user_agent.clone();
    let request_id_clone = request_id.clone();
    let error_message_clone = error_message.clone();
    let user_id_clone = user_id.clone();
    let response_body_clone = response_body.clone();

    // Insert audit log asynchronously (don't block response)
    let audit_log = audit_logs::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        timestamp: Set(Some(chrono::Utc::now().into())),
        method: Set(method),
        path: Set(path),
        status_code: Set(Some(status_code)),
        response_time_ms: Set(Some(response_time_ms)),
        user_id: Set(user_id),
        ip_address: Set(ip_address),
        user_agent: Set(user_agent),
        request_body: Set(request_body),
        response_body: Set(response_body_clone),
        error_message: Set(error_message_clone.clone()),
    };
    let db_clone = db.clone();
    tokio::spawn(async move {
        if let Err(e) = audit_log.insert(&db_clone).await {
            error!(request_id = %request_id_clone, error = ?e, "Failed to insert audit log");
        } else {
            info!(request_id = %request_id_clone, "Audit log inserted");
        }
    });

    // Optionally log to tracing
    if let Some(ref err) = error_message_clone {
        error!(request_id = %request_id, status_code, user_id = ?user_id_clone, path = %path_clone, method = %method_clone, ip_address = ?ip_address_clone, user_agent = ?user_agent_clone, response_time_ms, error = %err, "Request error");
    } else {
        info!(request_id = %request_id, status_code, user_id = ?user_id_clone, path = %path_clone, method = %method_clone, ip_address = ?ip_address_clone, user_agent = ?user_agent_clone, response_time_ms, "Request completed");
    }

    Ok(result)
}
