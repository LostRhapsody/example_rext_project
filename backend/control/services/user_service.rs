use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use sea_orm::*;

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
            )
        }))
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
