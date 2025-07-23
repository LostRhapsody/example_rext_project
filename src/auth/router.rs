// use axum::middleware;
use utoipa_axum::{router::OpenApiRouter, routes};
use sea_orm::DatabaseConnection;

pub fn router(db: DatabaseConnection) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(super::handlers::register_handler))
        .routes(routes!(super::handlers::login_handler))
        .routes(routes!(super::handlers::logout_handler))
        .routes(routes!(super::handlers::profile_handler))
        .with_state(db)
} 