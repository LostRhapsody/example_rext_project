use axum::{routing::get, Router};
use sea_orm::{Database, DatabaseConnection};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Get database URL from environment
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    // Connect to database
    let db: DatabaseConnection = Database::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("Connected to database: {}", database_url);

    // Create router with a simple root route
    let app = Router::new()
        .route("/", get(root_handler));

    // Start the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await?;

    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

// Simple root handler
async fn root_handler() -> &'static str {
    "Hello from Axum server with SQLite connection!"
}
