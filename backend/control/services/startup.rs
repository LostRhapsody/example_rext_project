use sea_orm::DatabaseConnection;
use std::env;

use crate::infrastructure::{
    database::DatabaseManager, job_queue::JobQueueManager, scheduler::SchedulerManager,
    server::ServerManager,
};

/// Application startup orchestrator
pub struct StartupService;

impl StartupService {
    /// Initializes the application and returns the database connection
    pub async fn initialize() -> Result<DatabaseConnection, Box<dyn std::error::Error>> {
        // Load environment variables from .env file
        dotenvy::dotenv().ok();

        // Get environment configuration
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        println!("Environment: {}", environment);

        // Create database connection
        let db = DatabaseManager::create_connection().await?;

        // Create pool for job queue
        let pool = DatabaseManager::create_pool().await?;

        // Setup job queue storage
        DatabaseManager::setup_job_queue_storage(&pool).await?;

        // Create job storage
        let job_storage = JobQueueManager::create_storage(pool);

        // Queue test job
        println!("Queuing test job!");
        JobQueueManager::produce_messages(&job_storage).await?;

        Ok(db)
    }

    /// Runs the server task
    pub async fn run_server(db: DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
        let router = ServerManager::create_router(db);
        ServerManager::start_server(router).await?;
        Ok(())
    }

    /// Runs the job queue monitor task
    pub async fn run_job_queue_monitor() -> Result<(), Box<dyn std::error::Error>> {
        let pool = DatabaseManager::create_pool().await?;
        let job_storage = JobQueueManager::create_storage(pool);

        JobQueueManager::run_job_queue_monitor(job_storage).await?;
        Ok(())
    }

    /// Runs the task scheduler
    pub async fn run_scheduler() -> Result<(), Box<dyn std::error::Error>> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
        SchedulerManager::run_scheduler(&database_url).await?;
        Ok(())
    }
}
