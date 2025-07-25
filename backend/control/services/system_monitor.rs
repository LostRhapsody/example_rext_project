use sysinfo::System;
use sea_orm::DatabaseConnection;

/// System monitoring service for collecting system metrics
pub struct SystemMonitorService;

/// System metrics data structure
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_available: u64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub disk_available: u64,
    pub network_bytes_sent: u64,
    pub network_bytes_received: u64,
    pub uptime: u64,
    pub process_count: usize,
    pub database_connections: Option<u32>,
}

impl SystemMonitorService {
    /// Get current system metrics
    pub async fn get_system_metrics(db: &DatabaseConnection) -> SystemMetrics {
        let mut sys = System::new_all();
        sys.refresh_all();

        // Get CPU usage (average across all cores)
        let cpu_usage = sys.global_cpu_usage();

        // Get memory information
        let memory_total = sys.total_memory();
        let memory_used = sys.used_memory();
        let memory_available = sys.free_memory();

        // For now, we'll set disk and network to 0 since the methods aren't available
        // We can enhance this later with proper disk and network monitoring
        let disk_total = 0;
        let disk_used = 0;
        let disk_available = 0;
        let network_bytes_sent = 0;
        let network_bytes_received = 0;

        // Get system uptime (using the associated function)
        let uptime = System::uptime();

        // Get process count
        let process_count = sys.processes().len();

        // Get database connection count (if available)
        let database_connections = Self::get_database_connections(db).await;

        SystemMetrics {
            cpu_usage,
            memory_total,
            memory_used,
            memory_available,
            disk_total,
            disk_used,
            disk_available,
            network_bytes_sent,
            network_bytes_received,
            uptime,
            process_count,
            database_connections,
        }
    }

    /// Get database connection count from SeaORM connection
    async fn get_database_connections(db: &DatabaseConnection) -> Option<u32> {
        // Try to get the underlying sqlx pool from SeaORM
        // This is a bit of a hack since SeaORM doesn't expose the pool directly
        // We'll try to execute a simple query to check if the connection is alive
        // and return None if we can't get the connection count
        match db.ping().await {
            Ok(_) => {
                // For now, we'll return None since we can't easily get the connection count
                // from SeaORM's DatabaseConnection. We could implement a more sophisticated
                // approach if needed.
                None
            }
            Err(_) => None,
        }
    }

    /// Get memory usage percentage
    pub fn get_memory_usage_percentage(metrics: &SystemMetrics) -> f32 {
        if metrics.memory_total > 0 {
            (metrics.memory_used as f32 / metrics.memory_total as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Get disk usage percentage
    pub fn get_disk_usage_percentage(metrics: &SystemMetrics) -> f32 {
        if metrics.disk_total > 0 {
            (metrics.disk_used as f32 / metrics.disk_total as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Format bytes to human readable format
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.1} {}", size, UNITS[unit_index])
    }

    /// Format uptime to human readable format
    pub fn format_uptime(seconds: u64) -> String {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        let minutes = (seconds % 3600) / 60;

        if days > 0 {
            format!("{}d {}h {}m", days, hours, minutes)
        } else if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }

    /// Get system health status based on metrics
    pub fn get_health_status(metrics: &SystemMetrics) -> String {
        let cpu_usage = metrics.cpu_usage;
        let memory_usage = Self::get_memory_usage_percentage(metrics);
        let disk_usage = Self::get_disk_usage_percentage(metrics);

        // Define thresholds
        if cpu_usage > 90.0 || memory_usage > 90.0 || disk_usage > 90.0 {
            "Critical".to_string()
        } else if cpu_usage > 80.0 || memory_usage > 80.0 || disk_usage > 80.0 {
            "Warning".to_string()
        } else if cpu_usage > 70.0 || memory_usage > 70.0 || disk_usage > 70.0 {
            "Degraded".to_string()
        } else {
            "Healthy".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(SystemMonitorService::format_bytes(1024), "1.0 KB");
        assert_eq!(SystemMonitorService::format_bytes(1048576), "1.0 MB");
        assert_eq!(SystemMonitorService::format_bytes(1073741824), "1.0 GB");
        assert_eq!(SystemMonitorService::format_bytes(512), "512.0 B");
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(SystemMonitorService::format_uptime(3661), "1h 1m");
        assert_eq!(SystemMonitorService::format_uptime(86400), "1d 0h 0m");
        assert_eq!(SystemMonitorService::format_uptime(3600), "1h 0m");
        assert_eq!(SystemMonitorService::format_uptime(120), "2m");
    }

    #[test]
    fn test_memory_usage_percentage() {
        let metrics = SystemMetrics {
            cpu_usage: 0.0,
            memory_total: 1000,
            memory_used: 500,
            memory_available: 500,
            disk_total: 0,
            disk_used: 0,
            disk_available: 0,
            network_bytes_sent: 0,
            network_bytes_received: 0,
            uptime: 0,
            process_count: 0,
            database_connections: None,
        };

        assert_eq!(SystemMonitorService::get_memory_usage_percentage(&metrics), 50.0);
    }

    #[test]
    fn test_health_status() {
        let healthy_metrics = SystemMetrics {
            cpu_usage: 30.0,
            memory_total: 1000,
            memory_used: 300,
            memory_available: 700,
            disk_total: 0,
            disk_used: 0,
            disk_available: 0,
            network_bytes_sent: 0,
            network_bytes_received: 0,
            uptime: 0,
            process_count: 0,
            database_connections: None,
        };

        assert_eq!(SystemMonitorService::get_health_status(&healthy_metrics), "Healthy");
    }
}