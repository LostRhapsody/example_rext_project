use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Domain model for a user
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub created_at: Option<DateTime<Utc>>,
}

impl User {
    /// Creates a new user domain model
    pub fn new(
        id: Uuid,
        email: String,
        password_hash: String,
        created_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            email,
            password_hash,
            created_at,
        }
    }

    /// Creates a new user for registration
    pub fn create_new(email: String, password_hash: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            password_hash,
            created_at: Some(Utc::now()),
        }
    }
}

/// Domain model for user registration
#[derive(Debug)]
pub struct UserRegistration {
    pub email: String,
    pub password: String,
}

impl UserRegistration {
    pub fn new(email: String, password: String) -> Self {
        Self { email, password }
    }
}

/// Domain model for user login
#[derive(Debug)]
pub struct UserLogin {
    pub email: String,
    pub password: String,
}

impl UserLogin {
    pub fn new(email: String, password: String) -> Self {
        Self { email, password }
    }
}
