use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserSessions::Table)
                    .col(ColumnDef::new(UserSessions::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(UserSessions::UserId)
                            .uuid()
                            .not_null()
                    )
                    .col(ColumnDef::new(UserSessions::SessionToken).string().not_null().unique_key())
                    .col(ColumnDef::new(UserSessions::UserAgent).text().null())
                    .col(
                        ColumnDef::new(UserSessions::IpAddress)
                            .string()
                            .null()
                    )
                    .col(
                        ColumnDef::new(UserSessions::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                    )
                    .col(
                        ColumnDef::new(UserSessions::LastActivity)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                    )
                    .col(
                        ColumnDef::new(UserSessions::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(UserSessions::IsActive)
                            .boolean()
                            .default(true)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_sessions_user_id")
                            .from(UserSessions::Table, UserSessions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for performance
        manager
            .create_index(
                Index::create()
                    .name("idx_user_sessions_user_id")
                    .table(UserSessions::Table)
                    .col(UserSessions::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_sessions_token")
                    .table(UserSessions::Table)
                    .col(UserSessions::SessionToken)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_sessions_active")
                    .table(UserSessions::Table)
                    .col(UserSessions::IsActive)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_sessions_expires")
                    .table(UserSessions::Table)
                    .col(UserSessions::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        // Composite index for efficient active session queries
        manager
            .create_index(
                Index::create()
                    .name("idx_user_sessions_user_active")
                    .table(UserSessions::Table)
                    .col(UserSessions::UserId)
                    .col(UserSessions::IsActive)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserSessions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserSessions {
    Table,
    Id,
    UserId,
    SessionToken,
    UserAgent,
    IpAddress,
    CreatedAt,
    LastActivity,
    ExpiresAt,
    IsActive,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}