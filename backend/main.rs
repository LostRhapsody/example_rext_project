mod bridge;
mod control;
mod domain;
pub mod entity;
mod infrastructure;

use control::services::startup::StartupService;
use infrastructure::{logging::LoggingManager, websocket::start_metrics_broadcaster};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging first
    LoggingManager::initialize();

    tracing::info!("Starting the Rext Server 🦖");

    // Initialize the application
    let db = StartupService::initialize().await?;

    // Run all services concurrently
    let _result = tokio::join!(
        StartupService::run_server(db),
        StartupService::run_job_queue_monitor(),
        StartupService::run_scheduler(),
        start_metrics_broadcaster()
    );

    Ok(())
}
