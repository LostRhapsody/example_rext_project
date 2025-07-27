use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

pub const ADMIN_TAG: &str = "Admin";

// Admin Authentication
#[derive(Deserialize, ToSchema)]
pub struct AdminLoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct AdminLoginResponse {
    pub token: String,
    pub admin_id: String,
    pub email: String,
}

// Pagination
#[derive(Serialize, ToSchema)]
pub struct PaginationMeta {
    pub page: u64,
    pub limit: u64,
    pub total: u64,
    pub total_pages: u64,
}

#[derive(Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

// Request History (Audit Logs)
#[derive(Deserialize, ToSchema, IntoParams)]
pub struct LogsQueryParams {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
    pub method: Option<String>,
    pub status_code: Option<i32>,
    pub user_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct AuditLogResponse {
    pub id: String,
    pub timestamp: Option<String>,
    pub method: String,
    pub path: String,
    pub status_code: Option<i32>,
    pub response_time_ms: Option<i32>,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
}

// User Management
#[derive(Deserialize, ToSchema, IntoParams)]
pub struct UsersQueryParams {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
    pub search: Option<String>,
    pub is_admin: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub is_admin: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub password: Option<String>,
    pub is_admin: Option<bool>,
}

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub created_at: Option<String>,
    pub is_admin: Option<bool>,
}

// Database Inspection
#[derive(Serialize, ToSchema)]
pub struct DatabaseTableResponse {
    pub name: String,
    pub record_count: u64,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct TableRecordsQueryParams {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
}

#[derive(Serialize, ToSchema)]
pub struct TableRecordResponse {
    pub columns: Vec<String>,
    pub records: Vec<Vec<serde_json::Value>>,
}

// Database Performance Metrics
#[derive(Serialize, ToSchema)]
pub struct DatabasePerformanceResponse {
    pub total_queries: u64,
    pub avg_execution_time_ms: f64,
    pub p50_execution_time_ms: f64,
    pub p95_execution_time_ms: f64,
    pub p99_execution_time_ms: f64,
    pub max_execution_time_ms: f64,
    pub error_rate: f64,
    pub queries_per_second: f64,
    pub slow_query_count: u64,
    pub critical_query_count: u64,
}

// System Health
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub uptime: String,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub memory_total: String,
    pub memory_used: String,
    pub memory_available: String,
    pub disk_usage: f32,
    pub disk_total: String,
    pub disk_used: String,
    pub disk_available: String,
    pub network_bytes_sent: String,
    pub network_bytes_received: String,
    pub process_count: usize,
    pub database_connections: Option<u32>,
    pub database_status: String,
    pub database_performance: Option<DatabasePerformanceResponse>,
    // User Analytics
    pub total_users: u64,
    pub active_users_7_days: u64,
    pub new_users_24_hours: u64,
    pub new_users_7_days: u64,
    pub new_users_30_days: u64,
}

#[derive(Serialize, ToSchema)]
pub struct SystemInfoResponse {
    pub version: String,
    pub environment: String,
    pub start_time: String,
    pub database_type: String,
    pub database_tables: u64,
    pub database_status: String,
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub system_metrics: HealthResponse,
}

// Helper functions for defaults
fn default_page() -> u64 { 1 }
fn default_limit() -> u64 { 25 }