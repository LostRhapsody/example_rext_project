use axum::middleware;
use sea_orm::DatabaseConnection;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router(db: DatabaseConnection) -> OpenApiRouter {
    // Routes that don't need authentication
    let public_routes = OpenApiRouter::new()
        .routes(routes!(super::handlers::register_handler))
        .routes(routes!(super::handlers::login_handler))
        .routes(routes!(super::handlers::logout_handler));

    // Routes that need authentication
    let protected_routes = OpenApiRouter::new()
        .routes(routes!(super::handlers::profile_handler))
        .route_layer(middleware::from_fn(super::handlers::auth_middleware));

    // Combine both route groups - retains the middleware layers
    public_routes.merge(protected_routes).with_state(db)
}
