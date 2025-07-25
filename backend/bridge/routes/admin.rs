use axum::{middleware, routing::{get, post, put, delete}};
use sea_orm::DatabaseConnection;
use utoipa_axum::router::OpenApiRouter;

use crate::bridge::handlers::admin::*;
use crate::bridge::middleware::admin::admin_middleware;

pub fn admin_router(db: DatabaseConnection) -> OpenApiRouter {
    // Admin authentication routes (no middleware needed)
    let auth_routes = OpenApiRouter::new()
        .route("/login", post(admin_login_handler))
        .route("/logout", post(admin_logout_handler));

    // Protected admin routes (require admin middleware)
    let protected_routes = OpenApiRouter::new()
        // Audit logs
        .route("/logs", get(get_audit_logs_handler))
        // User management
        .route("/users", get(get_users_handler))
        .route("/users", post(create_user_handler))
        .route("/users/{id}", get(get_user_handler))
        .route("/users/{id}", put(update_user_handler))
        .route("/users/{id}", delete(delete_user_handler))
        // Database inspection
        .route("/database/tables", get(get_database_tables_handler))
        .route("/database/tables/{table_name}", get(get_table_records_handler))
        // System health
        .route("/health", get(health_handler))
        .route_layer(middleware::from_fn(admin_middleware));

    // Combine auth and protected routes
    auth_routes.merge(protected_routes).with_state(db)
}