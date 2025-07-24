mod bridge;
mod control;
mod domain;
mod entity;
mod infrastructure;

use apalis::prelude::*;
use apalis_cron::{CronStream, Schedule};
use apalis_sql::sqlite::SqliteStorage;
use axum::{
    http::{HeaderName, HeaderValue, Method},
    routing::get,
};
use chrono::{DateTime, Utc};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::{
    env,
    io::Error,
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

// Re-export types from auth module for OpenAPI documentation
use bridge::routes::auth::auth_router;
use bridge::types::auth::{AUTH_TAG, RegisterRequest, RegisterResponse};
use infrastructure::app_error::{ErrorResponse, MessageResponse};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    to: String,
    text: String,
    subject: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct Reminder(DateTime<Utc>);
impl From<DateTime<Utc>> for Reminder {
    fn from(t: DateTime<Utc>) -> Self {
        Reminder(t)
    }
}

async fn root_handler() -> &'static str {
    "Rext Example Server Root, API docs at /scalar, frontend at http://localhost:5173"
}

async fn produce_messages(storage: &SqliteStorage<Message>) -> Result<(), Error> {
    let mut storage = storage.clone();
    for i in 0..1 {
        storage
            .schedule(
                Message {
                    to: format!("test{i}@example.com"),
                    text: "Test background job from apalis".to_string(),
                    subject: "Background email job".to_string(),
                },
                (Utc::now() + chrono::Duration::seconds(4)).timestamp(),
            )
            .await
            .unwrap();
    }
    Ok(())
}

async fn send_message(message: Message) -> Result<(), Error> {
    println!("Sending message: {:?}", message);
    Ok(())
}

async fn handle_tick(job: Reminder) -> Result<(), Error> {
    println!("Handling tick: {:?}", job);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Starting the Rext Server 🦖");

    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Get database URL and environment (fallback to development) from .env file
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    println!("Environment: {}", environment);

    // Create DB pool
    let pool = sqlx::SqlitePool::connect(&database_url).await.unwrap();
    // Run migrations for apalis job queue (sets up necessary tables, usually a one-time thing)
    SqliteStorage::setup(&pool)
        .await
        .expect("unable to run migrations for sqlite");

    // Connect SEAORM to database
    let db: DatabaseConnection = Database::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("Connected to database: {}", database_url);

    // Connect the job queue to the database
    let job_storage: SqliteStorage<Message> = SqliteStorage::new(pool);

    // a test job that is queued for 4 seconds into the future in our job storage
    println!("Queuing test job!");
    produce_messages(&job_storage).await?;

    // Create the Axum web server
    let server_task = async {
        // Create CORS layer for development
        let cors =
            if environment == "development" {
                CorsLayer::new()
                    .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
                    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                    .allow_headers([
                        "authorization".parse::<HeaderName>().unwrap(),
                        "content-type".parse::<HeaderName>().unwrap(),
                        "accept".parse::<HeaderName>().unwrap(),
                        "origin".parse::<HeaderName>().unwrap(),
                        "x-requested-with".parse::<HeaderName>().unwrap(),
                    ])
                    .allow_credentials(true)
            } else {
                // Production CORS configuration
                let allowed_origin = env::var("ALLOWED_ORIGIN")
                    .unwrap_or_else(|_| "https://yourdomain.com".to_string());

                CorsLayer::new()
                    .allow_origin(allowed_origin.parse::<HeaderValue>().unwrap_or_else(|_| {
                        "https://yourdomain.com".parse::<HeaderValue>().unwrap()
                    }))
                    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                    .allow_headers([
                        "authorization".parse::<HeaderName>().unwrap(),
                        "content-type".parse::<HeaderName>().unwrap(),
                        "accept".parse::<HeaderName>().unwrap(),
                        "origin".parse::<HeaderName>().unwrap(),
                        "x-requested-with".parse::<HeaderName>().unwrap(),
                    ])
                    .allow_credentials(true)
                    .max_age(std::time::Duration::from_secs(3600)) // Cache preflight for 1 hour
            };

        // Create the OpenAPI Router
        let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
            .nest("/api/v1/auth", auth_router(db.clone()))
            .split_for_parts();

        let mut router = router
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()))
            .merge(Redoc::with_url("/redoc", api.clone()))
            .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
            .merge(Scalar::with_url("/scalar", api))
            .route("/", get(root_handler));

        if environment == "development" {
            router = router.layer(cors);
        }

        // Check if we're in production mode and serve static files
        if environment == "production" {
            println!("Production mode detected - serving static files from /dist directory");
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
    };

    // Create job queue monitor task (runs the worker that processes the jobs)
    let job_queue_task = async {
        Monitor::new()
            .register({
                WorkerBuilder::new("tasty-banana")
                    // .enable_tracing()
                    .backend(job_storage)
                    .build_fn(send_message)
            })
            .run()
            .await
            .unwrap();
        Ok::<(), Error>(())
    };

    // Create task scheduler task
    let schedule_task = async {
        // You can use various cron expressions, but remember this includes seconds, where standard cron does not
        // https://crontab.guru helps
        // Create DB pool
        let cron_pool = sqlx::SqlitePool::connect(&database_url).await.unwrap();
        let schedule = Schedule::from_str("0 */1 * * * *").unwrap(); // every minute
        println!("Starting cron worker with schedule: {}", schedule);
        let cron_stream = CronStream::new(schedule);
        let sqlite_storage = SqliteStorage::new(cron_pool);
        let cron_backend = cron_stream.pipe_to_storage(sqlite_storage);

        let worker = WorkerBuilder::new("morning-cereal")
            .backend(cron_backend)
            .build_fn(handle_tick);

        Monitor::new().register(worker).run().await.unwrap();
        Ok::<(), Error>(())
    };

    // Run both tasks concurrently
    let _result = tokio::join!(server_task, job_queue_task, schedule_task);

    Ok(())
}
