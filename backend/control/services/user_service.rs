use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use sea_orm::*;
use sea_orm::prelude::Expr;
use uuid::Uuid;

use crate::domain::{user::*, validation::*};
use crate::entity::models::{prelude::*, *};
use crate::infrastructure::app_error::AppError;
use axum::http::StatusCode;

/// Service for user-related business operations
pub struct UserService;

impl UserService {
    /// Creates a new user in the database
    pub async fn create_user(
        db: &DatabaseConnection,
        registration: UserRegistration,
    ) -> Result<User, AppError> {
        // Validate input
        validate_registration_input(&registration.email, &registration.password)?;

        // Check if user already exists
        let existing_user: Option<users::Model> = Users::find()
            .filter(users::Column::Email.eq(registration.email.clone()))
            .one(db)
            .await
            .map_err(|_| AppError {
                message: "Database error".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        if existing_user.is_some() {
            return Err(AppError {
                message: "User already exists".to_string(),
                status_code: StatusCode::CONFLICT,
            });
        }

        // Hash password
        let password_hash = Self::hash_password(&registration.password)?;

        // Create user domain model
        let user = User::create_new(registration.email, password_hash);

        // Save to database
        let user_active_model = users::ActiveModel {
            id: Set(user.id),
            email: Set(user.email.clone()),
            password_hash: Set(user.password_hash.clone()),
            created_at: Set(user.created_at.map(|dt| dt.fixed_offset())),
            is_admin: Set(Some(false)), // Default to non-admin
            last_login: Set(None),
        };

        Users::insert(user_active_model)
            .exec(db)
            .await
            .map_err(|_| AppError {
                message: "Failed to create user".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        Ok(user)
    }

    /// Creates a new user with admin privileges (for admin service)
    pub async fn create_user_with_admin(
        db: &DatabaseConnection,
        email: String,
        password: String,
        is_admin: bool,
    ) -> Result<User, AppError> {
        // Validate input
        validate_registration_input(&email, &password)?;

        // Check if user already exists
        let existing_user: Option<users::Model> = Users::find()
            .filter(users::Column::Email.eq(&email))
            .one(db)
            .await
            .map_err(|_| AppError {
                message: "Database error".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        if existing_user.is_some() {
            return Err(AppError {
                message: "User already exists".to_string(),
                status_code: StatusCode::CONFLICT,
            });
        }

        // Hash password
        let password_hash = Self::hash_password(&password)?;

        // Create user domain model
        let user = User::create_new(email, password_hash);

        // Save to database
        let user_active_model = users::ActiveModel {
            id: Set(user.id),
            email: Set(user.email.clone()),
            password_hash: Set(user.password_hash.clone()),
            created_at: Set(user.created_at.map(|dt| dt.fixed_offset())),
            is_admin: Set(Some(is_admin)),
            last_login: Set(None),
        };

        Users::insert(user_active_model)
            .exec(db)
            .await
            .map_err(|_| AppError {
                message: "Failed to create user".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        Ok(user)
    }

    /// Updates a user's last login timestamp (non-blocking)
    pub async fn update_last_login(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let now = chrono::Utc::now();

        // Use a non-blocking update operation
        let update_result = Users::update_many()
            .col_expr(users::Column::LastLogin, Expr::value(now.fixed_offset()))
            .filter(users::Column::Id.eq(user_id))
            .exec(db)
            .await;

        // We don't want to fail the login if this update fails
        if let Err(e) = update_result {
            // Log the error but don't return it to avoid blocking login
            eprintln!("Failed to update last_login for user {}: {:?}", user_id, e);
        }

        Ok(())
    }

    /// Finds a user by email
    pub async fn find_user_by_email(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<Option<User>, AppError> {
        let user_model: Option<users::Model> = Users::find()
            .filter(users::Column::Email.eq(email))
            .one(db)
            .await
            .map_err(|_| AppError {
                message: "Database error".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        Ok(user_model.map(|model| {
            User::new(
                model.id,
                model.email,
                model.password_hash,
                model.created_at.map(|dt| dt.to_utc()),
                model.last_login.map(|dt| dt.to_utc()),
            )
        }))
    }

    /// Finds a user by ID
    pub async fn find_user_by_id(
        db: &DatabaseConnection,
        user_id: uuid::Uuid,
    ) -> Result<Option<User>, AppError> {
        let user_model: Option<users::Model> =
            Users::find_by_id(user_id)
                .one(db)
                .await
                .map_err(|_| AppError {
                    message: "Database error".to_string(),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?;

        Ok(user_model.map(|model| {
            User::new(
                model.id,
                model.email,
                model.password_hash,
                model.created_at.map(|dt| dt.to_utc()),
                model.last_login.map(|dt| dt.to_utc()),
            )
        }))
    }

    /// Updates a user
    pub async fn update_user(
        db: &DatabaseConnection,
        user_id: Uuid,
        email: Option<String>,
        password: Option<String>,
        is_admin: Option<bool>,
    ) -> Result<User, AppError> {
        let user_model = Users::find_by_id(user_id)
            .one(db)
            .await
            .map_err(|_| AppError {
                message: "Database error".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?
            .ok_or(AppError {
                message: "User not found".to_string(),
                status_code: StatusCode::NOT_FOUND,
            })?;

        let mut user_active_model: users::ActiveModel = user_model.clone().into();

        // Update email if provided
        if let Some(new_email) = email {
            validate_email(&new_email)?;

            // Check if email is already taken by another user
            let existing_user = Users::find()
                .filter(users::Column::Email.eq(&new_email))
                .filter(users::Column::Id.ne(user_id))
                .one(db)
                .await
                .map_err(|_| AppError {
                    message: "Database error".to_string(),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?;

            if existing_user.is_some() {
                return Err(AppError {
                    message: "Email already taken".to_string(),
                    status_code: StatusCode::CONFLICT,
                });
            }

            user_active_model.email = Set(new_email);
        }

        // Update password if provided
        if let Some(new_password) = password {
            validate_password(&new_password)?;
            let password_hash = Self::hash_password(&new_password)?;
            user_active_model.password_hash = Set(password_hash);
        }

        // Update admin status if provided
        if let Some(admin_status) = is_admin {
            user_active_model.is_admin = Set(Some(admin_status));
        }

        let updated_user = user_active_model.update(db).await.map_err(|_| AppError {
            message: "Failed to update user".to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        Ok(User::new(
            updated_user.id,
            updated_user.email,
            updated_user.password_hash,
            updated_user.created_at.map(|dt| dt.to_utc()),
            updated_user.last_login.map(|dt| dt.to_utc()),
        ))
    }

    /// Deletes a user
    pub async fn delete_user(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let user_model = Users::find_by_id(user_id)
            .one(db)
            .await
            .map_err(|_| AppError {
                message: "Database error".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?
            .ok_or(AppError {
                message: "User not found".to_string(),
                status_code: StatusCode::NOT_FOUND,
            })?;

        let user_active_model: users::ActiveModel = user_model.into();
        user_active_model.delete(db).await.map_err(|_| AppError {
            message: "Failed to delete user".to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        Ok(())
    }

    /// Verifies a user's password
    pub fn verify_password(user: &User, password: &str) -> Result<bool, AppError> {
        let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|_| AppError {
            message: "Invalid password hash".to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        let argon2 = Argon2::default();
        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Hashes a password using Argon2
    fn hash_password(password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut rand_core::OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| AppError {
                message: "Failed to hash password".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })?
            .to_string();

        Ok(password_hash)
    }
}
