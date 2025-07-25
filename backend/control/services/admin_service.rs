use argon2::{Argon2, PasswordHash, PasswordVerifier};
use sea_orm::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    bridge::types::admin::*,
    domain::{auth::*, user::*, validation::*},
    entity::models::{audit_logs, users},
    infrastructure::{app_error::AppError, jwt_claims::Claims},
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

        // Find user by email
        let user = users::Entity::find()
            .filter(users::Column::Email.eq(&login.email))
            .one(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?
            .ok_or(AppError {
                message: "Invalid credentials".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            })?;

        // Check if user is admin
        if !user.is_admin.unwrap_or(false) {
            return Err(AppError {
                message: "Access denied: Admin privileges required".to_string(),
                status_code: StatusCode::FORBIDDEN,
            });
        }

        // Verify password
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|_| AppError {
                message: "Invalid password hash".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        let is_valid = Argon2::default()
            .verify_password(login.password.as_bytes(), &parsed_hash)
            .is_ok();

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
                total_pages: total_pages,
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
                total_pages: total_pages,
            },
        })
    }

    /// Get specific user by ID
    pub async fn get_user(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<UserResponse, AppError> {
        let user = users::Entity::find_by_id(user_id)
            .one(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?
            .ok_or(AppError {
                message: "User not found".to_string(),
                status_code: StatusCode::NOT_FOUND,
            })?;

        Ok(UserResponse {
            id: user.id.to_string(),
            email: user.email,
            created_at: user.created_at.map(|t| t.to_rfc3339()),
            is_admin: user.is_admin,
        })
    }

    /// Create a new user
    pub async fn create_user(
        db: &DatabaseConnection,
        request: CreateUserRequest,
    ) -> Result<UserResponse, AppError> {
        // Validate input
        validate_registration_input(&request.email, &request.password)?;

        // Check if user already exists
        let existing_user = users::Entity::find()
            .filter(users::Column::Email.eq(&request.email))
            .one(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        if existing_user.is_some() {
            return Err(AppError {
                message: "User already exists".to_string(),
                status_code: StatusCode::CONFLICT,
            });
        }

        // Hash password
        let salt = rand::random::<[u8; 16]>();
        let argon2 = Argon2::default();
        let password_hash = PasswordHash::generate(&argon2, request.password.as_bytes(), &salt)
            .map_err(|_| AppError {
                message: "Failed to hash password".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?
            .to_string();

        // Create user
        let user_id = Uuid::new_v4();
        let user = users::ActiveModel {
            id: Set(user_id),
            email: Set(request.email),
            password_hash: Set(password_hash),
            created_at: Set(Some(chrono::Utc::now().into())),
            is_admin: Set(request.is_admin.unwrap_or(false)),
        };

        let user = user.insert(db).await.map_err(|e| AppError {
            message: format!("Failed to create user: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        Ok(UserResponse {
            id: user.id.to_string(),
            email: user.email,
            created_at: user.created_at.map(|t| t.to_rfc3339()),
            is_admin: user.is_admin,
        })
    }

    /// Update a user
    pub async fn update_user(
        db: &DatabaseConnection,
        user_id: Uuid,
        request: UpdateUserRequest,
    ) -> Result<UserResponse, AppError> {
        let mut user = users::Entity::find_by_id(user_id)
            .one(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?
            .ok_or(AppError {
                message: "User not found".to_string(),
                status_code: StatusCode::NOT_FOUND,
            })?;

        let mut user_model: users::ActiveModel = user.clone().into();

        // Update email if provided
        if let Some(email) = request.email {
            validate_email(&email)?;

            // Check if email is already taken by another user
            let existing_user = users::Entity::find()
                .filter(users::Column::Email.eq(&email))
                .filter(users::Column::Id.ne(user_id))
                .one(db)
                .await
                .map_err(|e| AppError {
                    message: format!("Database error: {}", e),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?;

            if existing_user.is_some() {
                return Err(AppError {
                    message: "Email already taken".to_string(),
                    status_code: StatusCode::CONFLICT,
                });
            }

            user_model.email = Set(email);
        }

        // Update password if provided
        if let Some(password) = request.password {
            validate_password(&password)?;

            let salt = rand::random::<[u8; 16]>();
            let argon2 = Argon2::default();
            let password_hash = PasswordHash::generate(&argon2, password.as_bytes(), &salt)
                .map_err(|_| AppError {
                    message: "Failed to hash password".to_string(),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?
                .to_string();

            user_model.password_hash = Set(password_hash);
        }

        // Update admin status if provided
        if let Some(is_admin) = request.is_admin {
            user_model.is_admin = Set(is_admin);
        }

        let user = user_model.update(db).await.map_err(|e| AppError {
            message: format!("Failed to update user: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        Ok(UserResponse {
            id: user.id.to_string(),
            email: user.email,
            created_at: user.created_at.map(|t| t.to_rfc3339()),
            is_admin: user.is_admin,
        })
    }

    /// Delete a user
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

        let user = users::Entity::find_by_id(user_id)
            .one(db)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?
            .ok_or(AppError {
                message: "User not found".to_string(),
                status_code: StatusCode::NOT_FOUND,
            })?;

        let user_model: users::ActiveModel = user.into();
        user_model.delete(db).await.map_err(|e| AppError {
            message: format!("Failed to delete user: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        Ok(())
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

            // Get record count for each table
            let count_result = db
                .query_one(Statement::from_sql_and_values(
                    db.get_database_backend(),
                    format!("SELECT COUNT(*) as count FROM {}", table_name),
                    vec![],
                ))
                .await
                .map_err(|e| AppError {
                    message: format!("Database error: {}", e),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?;

            let record_count: u64 = count_result
                .and_then(|row| row.try_get("", "count").ok())
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
                format!("PRAGMA table_info({})", table_name),
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
                    "SELECT * FROM {} LIMIT {} OFFSET {}",
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
                let value: serde_json::Value = row.try_get("", column).unwrap_or(serde_json::Value::Null);
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