mod entities;
mod auth;

use axum::{
    http::Method,
    routing::get,
};
use sea_orm::*;
use tokio::net::{TcpListener};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;
use std::{env, io::Error, net::{Ipv4Addr, SocketAddr}};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

// Re-export types from auth module for OpenAPI documentation
use auth::handlers::{RegisterRequest, RegisterResponse, MessageResponse, ErrorResponse};

pub const AUTH_TAG: &str = "Authentication";

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = AUTH_TAG, description = "Authentication endpoints")
    ),
    components(
        schemas(RegisterRequest, RegisterResponse, MessageResponse, ErrorResponse)
    )
)]
struct ApiDoc;

async fn root_handler() -> &'static str {
    "Rext Example Server Root, API docs at /scalar, frontend at http://localhost:5173"
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Get database URL from environment
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    // Connect to database
    let db: DatabaseConnection = Database::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("Connected to database: {}", database_url);

    // Create CORS layer for development
    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:5173"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    // Create the OpenAPI Router
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api/v1/auth", auth::router(db.clone()))
        .split_for_parts();

    let mut router = router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()))
        .merge(Redoc::with_url("/redoc", api.clone()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .merge(Scalar::with_url("/scalar", api))
        .route("/", get(root_handler))
        .layer(cors);

    // Check if we're in production mode and serve static files
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    if environment == "production" {
        println!("Production mode detected - serving static files from /dist directory");
        // Serve the Vue dist/ directory as a fallback for unmatched routes
        router = router.fallback_service(ServeDir::new("dist"));
    } else {
        println!("Development mode - static files not served by backend");
    }

    // Start the server
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3000));
    let listener = TcpListener::bind(&address).await?;
    println!("Server running on http://localhost:{}", address.port());
    println!("Frontend running on http://localhost:5173");
    println!("View API docs at:");
    println!("  http://localhost:{}/swagger-ui 📱 Swagger UI", address.port());
    println!("  http://localhost:{}/redoc 📖 Redoc", address.port());
    println!("  http://localhost:{}/api-docs/openapi.json ✏️ The OpenAPI JSON file", address.port());
    println!("  http://localhost:{}/scalar ⭐ Recommended for API testing", address.port());

    axum::serve(listener, router.into_make_service()).await
}
