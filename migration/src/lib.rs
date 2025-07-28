pub use sea_orm_migration::prelude::*;

mod m20250101_000002_create_audit_logs;
mod m20250101_000003_add_admin_role_to_users;
mod m20250720_000001_create_users;
mod m20250726_190121_user_last_login_field;
mod m20250101_000004_create_database_metrics;
mod m20250727_055016_user_roles;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250720_000001_create_users::Migration),
            Box::new(m20250101_000002_create_audit_logs::Migration),
            Box::new(m20250101_000003_add_admin_role_to_users::Migration),
            Box::new(m20250101_000004_create_database_metrics::Migration),
            Box::new(m20250726_190121_user_last_login_field::Migration),
            Box::new(m20250727_055016_user_roles::Migration),
        ]
    }
}
