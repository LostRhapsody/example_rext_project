use sea_orm::*;
use uuid::Uuid;
use base64::Engine;

use crate::{
    bridge::types::admin::*,
    domain::{validation::*},
    entity::models::{audit_logs, users},
    infrastructure::{app_error::AppError, jwt_claims::Claims},
    control::services::user_service::UserService,
};
use axum::http::StatusCode;
use jsonwebtoken::{EncodingKey, Header, encode};
use std::env;

/// Service for admin-related business operations
pub struct AdminService;

impl AdminService {
    /// Authenticates an admin user and returns a JWT token
    pub async fn authenticate_admin(
        db: &DatabaseConnection,
        login: AdminLoginRequest,
    ) -> Result<AdminLoginResponse, AppError> {
        // Validate input
        validate_login_input(&login.email, &login.password)?;

        // Find user by email using UserService
        let user = UserService::find_user_by_email(db, &login.email)
            .await?
            .ok_or(AppError {
                message: "Invalid credentials".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            })?;

        // Check if user is admin by querying the database directly for the is_admin field
        let user_model = users::Entity::find()
            .filter(users::Column::Email.eq(&login.email))
            .one(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        let user_model = user_model.ok_or(AppError {
            message: "Invalid credentials".to_string(),
            status_code: StatusCode::UNAUTHORIZED,
        })?;

        if !user_model.is_admin.unwrap_or(false) {
            return Err(AppError {
                message: "Access denied: Admin privileges required".to_string(),
                status_code: StatusCode::FORBIDDEN,
            });
        }

        // Verify password using UserService
        let is_valid = UserService::verify_password(&user, &login.password)?;
        if !is_valid {
            return Err(AppError {
                message: "Invalid credentials".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            });
        }

        // Generate JWT token
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
        let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());

        let claims = Claims {
            sub: user.id.to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        };

        let token = encode(&Header::default(), &claims, &encoding_key)
            .map_err(|_| AppError {
                message: "Failed to generate token".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        Ok(AdminLoginResponse {
            token,
            admin_id: user.id.to_string(),
            email: user.email,
        })
    }

    /// Get paginated audit logs with filtering
    pub async fn get_audit_logs(
        db: &DatabaseConnection,
        params: LogsQueryParams,
    ) -> Result<PaginatedResponse<AuditLogResponse>, AppError> {
        let offset = (params.page - 1) * params.limit;

        // Build query with filters
        let mut query = audit_logs::Entity::find();

        if let Some(method) = params.method {
            query = query.filter(audit_logs::Column::Method.eq(method));
        }

        if let Some(status_code) = params.status_code {
            query = query.filter(audit_logs::Column::StatusCode.eq(status_code));
        }

        if let Some(user_id) = params.user_id {
            if let Ok(uuid) = Uuid::parse_str(&user_id) {
                query = query.filter(audit_logs::Column::UserId.eq(uuid));
            }
        }

        if let Some(start_date) = params.start_date {
            if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(&start_date) {
                query = query.filter(audit_logs::Column::Timestamp.gte(datetime));
            }
        }

        if let Some(end_date) = params.end_date {
            if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(&end_date) {
                query = query.filter(audit_logs::Column::Timestamp.lte(datetime));
            }
        }

        // Get total count
        let total = query.clone().count(db).await.map_err(|e| AppError {
            message: format!("Database error: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        // Get paginated results
        let logs = query
            .order_by_desc(audit_logs::Column::Timestamp)
            .offset(offset as u64)
            .limit(params.limit as u64)
            .all(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        let data = logs
            .into_iter()
            .map(|log| AuditLogResponse {
                id: log.id.to_string(),
                timestamp: log.timestamp.map(|t| t.to_rfc3339()),
                method: log.method,
                path: log.path,
                status_code: log.status_code,
                response_time_ms: log.response_time_ms,
                user_id: log.user_id.map(|id| id.to_string()),
                ip_address: log.ip_address,
                user_agent: log.user_agent,
                request_body: log.request_body,
                response_body: log.response_body,
                error_message: log.error_message,
            })
            .collect();

        let total_pages = (total + params.limit - 1) / params.limit;

        Ok(PaginatedResponse {
            data,
            pagination: PaginationMeta {
                page: params.page,
                limit: params.limit,
                total,
                total_pages,
            },
        })
    }

    /// Get paginated users with filtering
    pub async fn get_users(
        db: &DatabaseConnection,
        params: UsersQueryParams,
    ) -> Result<PaginatedResponse<UserResponse>, AppError> {
        let offset = (params.page - 1) * params.limit;

        // Build query with filters
        let mut query = users::Entity::find();

        if let Some(search) = params.search {
            query = query.filter(users::Column::Email.contains(&search));
        }

        if let Some(is_admin) = params.is_admin {
            query = query.filter(users::Column::IsAdmin.eq(is_admin));
        }

        // Get total count
        let total = query.clone().count(db).await.map_err(|e| AppError {
            message: format!("Database error: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        // Get paginated results
        let users = query
            .order_by_desc(users::Column::CreatedAt)
            .offset(offset as u64)
            .limit(params.limit as u64)
            .all(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        let data = users
            .into_iter()
            .map(|user| UserResponse {
                id: user.id.to_string(),
                email: user.email,
                created_at: user.created_at.map(|t| t.to_rfc3339()),
                is_admin: user.is_admin,
            })
            .collect();

        let total_pages = (total + params.limit - 1) / params.limit;

        Ok(PaginatedResponse {
            data,
            pagination: PaginationMeta {
                page: params.page,
                limit: params.limit,
                total,
                total_pages,
            },
        })
    }

    /// Get specific user by ID using UserService
    pub async fn get_user(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<UserResponse, AppError> {
        let user = UserService::find_user_by_id(db, user_id)
            .await?
            .ok_or(AppError {
                message: "User not found".to_string(),
                status_code: StatusCode::NOT_FOUND,
            })?;

        // Get admin status from database
        let user_model = users::Entity::find_by_id(user_id)
            .one(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        let is_admin = user_model.map(|u| u.is_admin).flatten();

        Ok(UserResponse {
            id: user.id.to_string(),
            email: user.email,
            created_at: user.created_at.map(|t| t.to_rfc3339()),
            is_admin,
        })
    }

    /// Create a new user using UserService
    pub async fn create_user(
        db: &DatabaseConnection,
        request: CreateUserRequest,
    ) -> Result<UserResponse, AppError> {
        let user = UserService::create_user_with_admin(
            db,
            request.email,
            request.password,
            request.is_admin.unwrap_or(false),
        )
        .await?;

        Ok(UserResponse {
            id: user.id.to_string(),
            email: user.email,
            created_at: user.created_at.map(|t| t.to_rfc3339()),
            is_admin: Some(request.is_admin.unwrap_or(false)),
        })
    }

    /// Update a user using UserService
    pub async fn update_user(
        db: &DatabaseConnection,
        user_id: Uuid,
        request: UpdateUserRequest,
    ) -> Result<UserResponse, AppError> {
        let user = UserService::update_user(
            db,
            user_id,
            request.email,
            request.password,
            request.is_admin,
        )
        .await?;

        // Get admin status from database
        let user_model = users::Entity::find_by_id(user_id)
            .one(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        let is_admin = user_model.map(|u| u.is_admin).flatten();

        Ok(UserResponse {
            id: user.id.to_string(),
            email: user.email,
            created_at: user.created_at.map(|t| t.to_rfc3339()),
            is_admin,
        })
    }

    /// Delete a user using UserService
    pub async fn delete_user(
        db: &DatabaseConnection,
        user_id: Uuid,
        current_admin_id: Uuid,
    ) -> Result<(), AppError> {
        // Prevent admin from deleting themselves
        if user_id == current_admin_id {
            return Err(AppError {
                message: "Cannot delete your own account".to_string(),
                status_code: StatusCode::BAD_REQUEST,
            });
        }

        UserService::delete_user(db, user_id).await
    }

    /// Get list of database tables
    pub async fn get_database_tables(
        db: &DatabaseConnection,
    ) -> Result<Vec<DatabaseTableResponse>, AppError> {
        // For SQLite, we can query the sqlite_master table
        let tables = db
            .query_all(Statement::from_sql_and_values(
                db.get_database_backend(),
                r#"SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"#,
                vec![],
            ))
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        let mut result = Vec::new();
        for row in tables {
            let table_name: String = row.try_get("", "name").map_err(|_| AppError {
                message: "Failed to parse table name".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

            // Skip system tables
            if table_name.starts_with("sqlite_") || table_name.starts_with("_sqlx_") || table_name.starts_with("seaql_") {
                continue;
            }

            // Get record count for each table
            let count_result = db
                .query_one(Statement::from_sql_and_values(
                    db.get_database_backend(),
                    format!("SELECT COUNT(*) as count FROM \"{}\"", table_name),
                    vec![],
                ))
                .await
                .map_err(|e| AppError {
                    message: format!("Database error: {}", e),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?;

            let record_count: u64 = count_result
                .and_then(|row| {
                    // Try different ways to access the count column
                    let result = row.try_get::<i64>("", "count")
                        .map(|v| v as u64)
                        .or_else(|e| {
                            println!("Failed to get as i64: {:?}", e);
                            row.try_get::<u64>("", "count")
                        })
                        .or_else(|e| {
                            println!("Failed to get as u64: {:?}", e);
                            row.try_get::<i32>("", "count").map(|v| v as u64)
                        })
                        .or_else(|e| {
                            println!("Failed to get as i32: {:?}", e);
                            row.try_get::<u32>("", "count").map(|v| v as u64)
                        });
                    result.ok()
                })
                .unwrap_or(0);

            result.push(DatabaseTableResponse {
                name: table_name,
                record_count,
            });
        }

        Ok(result)
    }

    /// Get table records
    pub async fn get_table_records(
        db: &DatabaseConnection,
        table_name: String,
        params: TableRecordsQueryParams,
    ) -> Result<TableRecordResponse, AppError> {
        let offset = (params.page - 1) * params.limit;

        // Get column names
        let columns_result = db
            .query_all(Statement::from_sql_and_values(
                db.get_database_backend(),
                format!("PRAGMA table_info(\"{}\")", table_name),
                vec![],
            ))
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        let mut columns = Vec::new();
        for row in columns_result {
            let column_name: String = row.try_get("", "name").map_err(|_| AppError {
                message: "Failed to parse column name".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;
            columns.push(column_name);
        }

        // Get records
        let records_result = db
            .query_all(Statement::from_sql_and_values(
                db.get_database_backend(),
                format!(
                    "SELECT * FROM \"{}\" LIMIT {} OFFSET {}",
                    table_name, params.limit, offset
                ),
                vec![],
            ))
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        let mut records = Vec::new();
        for row in records_result {
            let mut record = Vec::new();
            for column in &columns {
                // Try to get the value as different types and convert to JSON
                let value = if let Ok(v) = row.try_get::<String>("", column) {
                    serde_json::Value::String(v)
                } else if let Ok(v) = row.try_get::<i64>("", column) {
                    serde_json::Value::Number(serde_json::Number::from(v))
                } else if let Ok(v) = row.try_get::<f64>("", column) {
                    if let Some(n) = serde_json::Number::from_f64(v) {
                        serde_json::Value::Number(n)
                    } else {
                        serde_json::Value::Null
                    }
                } else if let Ok(v) = row.try_get::<bool>("", column) {
                    serde_json::Value::Bool(v)
                } else if let Ok(v) = row.try_get::<Vec<u8>>("", column) {
                    // Convert blob to base64 string
                    serde_json::Value::String(base64::engine::general_purpose::STANDARD.encode(v))
                } else {
                    serde_json::Value::Null
                };
                record.push(value);
            }
            records.push(record);
        }

        Ok(TableRecordResponse { columns, records })
    }

    /// Get system health status
    pub async fn get_health_status() -> HealthResponse {
        HealthResponse {
            status: "OK".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}