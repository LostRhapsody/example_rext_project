use sea_orm::DatabaseConnection;
use std::env;

use crate::infrastructure::{
    database::DatabaseManager, job_queue::JobQueueManager, scheduler::SchedulerManager,
    server::ServerManager,
};
use crate::control::services::{user_service::UserService, server_config::ServerConfigService};

/// Application startup orchestrator
pub struct StartupService;

impl StartupService {
    /// Initializes the application and returns the database connection
    pub async fn initialize() -> Result<DatabaseConnection, Box<dyn std::error::Error>> {
        // Load environment variables from .env file
        dotenvy::dotenv().ok();

        // Initialize server configuration
        ServerConfigService::initialize();

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

        // Seed admin user if enabled
        Self::seed_admin_user(&db).await?;

        Ok(db)
    }

    /// Seeds the admin user if it doesn't exist
    async fn seed_admin_user(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
        // Check if admin user creation is enabled
        let create_admin = env::var("CREATE_ADMIN_USER")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        if !create_admin {
            println!("Admin user creation is disabled");
            return Ok(());
        }

        // Get admin credentials from environment variables
        let admin_email = env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@localhost".to_string());
        let admin_password = env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin".to_string());

        // Check if admin user already exists
        match UserService::find_user_by_email(db, &admin_email).await {
            Ok(Some(_)) => {
                println!("Admin user already exists: {}", admin_email);
                Ok(())
            }
            Ok(None) => {
                // Create admin user
                match UserService::create_user_with_admin(db, admin_email.clone(), admin_password, true).await {
                    Ok(user) => {
                        println!("✅ Admin user created successfully: {} (ID: {})", admin_email, user.id);
                        println!("⚠️  IMPORTANT: Change the default admin password immediately!");
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to create admin user: {}", e.message);
                        Err(Box::new(e))
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error checking for existing admin user: {}", e.message);
                Err(Box::new(e))
            }
        }
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
