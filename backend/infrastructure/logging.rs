use std::env;
use tracing_subscriber::{
    fmt::{format::FmtSpan, time::UtcTime},
    EnvFilter,
};

/// Logging configuration manager
pub struct LoggingManager;

impl LoggingManager {
    /// Initialize logging with environment-based configuration
    pub fn initialize() {
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        let log_level = env::var("RUST_LOG").unwrap_or_else(|_| {
            if environment == "development" {
                "debug".to_string()
            } else {
                "info".to_string()
            }
        });

        // Create environment filter
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(log_level));

        // Configure tracing subscriber
        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_timer(UtcTime::rfc_3339())
            .with_span_events(FmtSpan::CLOSE)
            .with_target(false)
            .with_thread_ids(true)
            .with_thread_names(true);

        // Use JSON format in production, pretty format in development
        if environment == "production" {
            subscriber.json().init();
        } else {
            subscriber.pretty().init();
        }

        tracing::info!("Logging initialized for environment: {}", environment);
    }

    /// Create a request ID for tracking requests across the system
    pub fn generate_request_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }
} 