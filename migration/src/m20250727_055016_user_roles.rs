use sea_orm_migration::{prelude::*, schema::*};
use backend::roles;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create roles table
        manager
            .create_table(
                Table::create()
                    .table(Roles::Table)
                    .if_not_exists()
                    .col(pk_auto(Roles::Id))
                    .col(string(Roles::Name).not_null().unique_key())
                    .col(string(Roles::Description))
                    .col(string(Roles::Permissions).not_null()) // JSON string of permissions
                    .col(
                        timestamp_with_time_zone(Roles::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Roles::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Add role_id column to users table
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::RoleId)
                            .integer()
                            .null()
                    )
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraint
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_users_role_id")
                    .from(Users::Table, Users::RoleId)
                    .to(Roles::Table, Roles::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        // Insert default roles
        let default_roles = vec![
            Roles::ActiveModel {
                name: Set("admin"),
                description: Set("Full system access"),
                permissions: Set("[\"*\"]"),
            },
            Roles::ActiveModel {
                name: Set("moderator"),
                description: Set("User management and content moderation"),
                permissions: Set("[\"users:read\", \"users:write\", \"logs:read\"]"),
            },
            Roles::ActiveModel {
                name: Set("user"),
                description: Set("Basic user access"),
                permissions: Set("[\"profile:read\", \"profile:write\"]"),
            },
        ];

        default_roles.iter().for_each(|role| {
            Roles::Entity::insert(role).exec(manager.get_connection()).await.unwrap();
        });

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop foreign key constraint
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_users_role_id")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        // Drop role_id column from users table
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::RoleId)
                    .to_owned(),
            )
            .await?;

        // Drop roles table
        manager
            .drop_table(Table::drop().table(Roles::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Roles {
    Table,
    Id,
    Name,
    Description,
    Permissions,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    RoleId,
}
