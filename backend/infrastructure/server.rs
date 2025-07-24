use axum::{Router, routing::get};
use sea_orm::DatabaseConnection;
use std::{
    env,
    io::Error,
    net::{Ipv4Addr, SocketAddr},
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

use crate::bridge::routes::auth::auth_router;
use crate::infrastructure::cors::CorsManager;
use crate::infrastructure::openapi::ApiDoc;

/// Server manager
pub struct ServerManager;

impl ServerManager {
    /// Root handler
    pub async fn root_handler() -> &'static str {
        "Rext Example Server Root, API docs at /scalar, frontend at http://localhost:5173"
    }

    /// Creates the main router with all endpoints
    pub fn create_router(db: DatabaseConnection) -> Router {
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        // Create the OpenAPI Router
        let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
            .nest("/api/v1/auth", auth_router(db))
            .split_for_parts();

        let mut router = router
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()))
            .merge(Redoc::with_url("/redoc", api.clone()))
            .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
            .merge(Scalar::with_url("/scalar", api))
            .route("/", get(Self::root_handler));

        // Add CORS layer for development
        if environment == "development" {
            router = router.layer(CorsManager::create_cors_layer());
        }

        // Check if we're in production mode and serve static files
        if environment == "production" {
            println!("Production mode detected - serving static files from /dist directory");
            router = router.fallback_service(ServeDir::new("dist"));
        } else {
            println!("Development mode - static files not served by backend");
        }

        router
    }

    /// Starts the server
    pub async fn start_server(router: Router) -> Result<(), Error> {
        let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3000));
        let listener = TcpListener::bind(&address).await?;

        println!("Server running on http://localhost:{}", address.port());
        println!("Frontend running on http://localhost:5173");
        println!("View API docs at:");
        println!(
            "  http://localhost:{}/swagger-ui 📱 Swagger UI",
            address.port()
        );
        println!("  http://localhost:{}/redoc 📖 Redoc", address.port());
        println!(
            "  http://localhost:{}/api-docs/openapi.json ✏️ The OpenAPI JSON file",
            address.port()
        );
        println!(
            "  http://localhost:{}/scalar ⭐ Recommended for API testing",
            address.port()
        );

        axum::serve(listener, router.into_make_service())
            .await
            .map_err(|e| Error::new(std::io::ErrorKind::Interrupted, e))
    }
}
