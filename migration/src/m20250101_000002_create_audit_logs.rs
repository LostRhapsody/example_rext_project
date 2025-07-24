use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250101_000002_create_audit_logs"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AuditLogs::Table)
                    .col(ColumnDef::new(AuditLogs::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(AuditLogs::Timestamp)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(AuditLogs::Method).string_len(10).not_null())
                    .col(ColumnDef::new(AuditLogs::Path).text().not_null())
                    .col(ColumnDef::new(AuditLogs::StatusCode).integer())
                    .col(ColumnDef::new(AuditLogs::ResponseTimeMs).integer())
                    .col(ColumnDef::new(AuditLogs::UserId).uuid())
                    .col(ColumnDef::new(AuditLogs::IpAddress).string_len(45))
                    .col(ColumnDef::new(AuditLogs::UserAgent).text())
                    .col(ColumnDef::new(AuditLogs::RequestBody).text())
                    .col(ColumnDef::new(AuditLogs::ResponseBody).text())
                    .col(ColumnDef::new(AuditLogs::ErrorMessage).text())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_audit_logs_user_id")
                            .from(AuditLogs::Table, AuditLogs::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuditLogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AuditLogs {
    Table,
    Id,
    Timestamp,
    Method,
    Path,
    StatusCode,
    ResponseTimeMs,
    UserId,
    IpAddress,
    UserAgent,
    RequestBody,
    ResponseBody,
    ErrorMessage,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
} 