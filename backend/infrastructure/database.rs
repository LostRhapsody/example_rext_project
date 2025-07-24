use sea_orm::*;
use sqlx::SqlitePool;
use std::env;

/// Database connection manager
pub struct DatabaseManager;

impl DatabaseManager {
    /// Creates and configures the database connection
    pub async fn create_connection() -> Result<DatabaseConnection, Box<dyn std::error::Error>> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

        let db: DatabaseConnection = Database::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        println!("Connected to database: {}", database_url);
        Ok(db)
    }

    /// Creates a SQLite pool for job queue operations
    pub async fn create_pool() -> Result<SqlitePool, Box<dyn std::error::Error>> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
        let pool = sqlx::SqlitePool::connect(&database_url).await?;
        Ok(pool)
    }

    /// Sets up job queue storage tables
    pub async fn setup_job_queue_storage(
        pool: &SqlitePool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use apalis_sql::sqlite::SqliteStorage;

        SqliteStorage::setup(pool)
            .await
            .expect("unable to run migrations for sqlite");

        Ok(())
    }
}
