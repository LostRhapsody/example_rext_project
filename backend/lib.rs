pub mod entity {
  pub mod models {
    pub mod roles;
  }
}

// Re-export the entity module
pub use entity::models::roles;