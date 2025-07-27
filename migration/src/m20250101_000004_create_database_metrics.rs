use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DatabaseMetrics::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DatabaseMetrics::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DatabaseMetrics::QueryHash).string().not_null())
                    .col(ColumnDef::new(DatabaseMetrics::QueryType).string().not_null())
                    .col(ColumnDef::new(DatabaseMetrics::TableName).string().null())
                    .col(ColumnDef::new(DatabaseMetrics::ExecutionTimeMs).big_integer().not_null())
                    .col(ColumnDef::new(DatabaseMetrics::RowsAffected).big_integer().null())
                    .col(ColumnDef::new(DatabaseMetrics::ErrorMessage).text().null())
                    .col(ColumnDef::new(DatabaseMetrics::Timestamp).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(DatabaseMetrics::CreatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        // Create indexes for efficient querying
        manager
            .create_index(
                Index::create()
                    .name("idx_database_metrics_timestamp")
                    .table(DatabaseMetrics::Table)
                    .col(DatabaseMetrics::Timestamp)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_database_metrics_query_type")
                    .table(DatabaseMetrics::Table)
                    .col(DatabaseMetrics::QueryType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_database_metrics_table_name")
                    .table(DatabaseMetrics::Table)
                    .col(DatabaseMetrics::TableName)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DatabaseMetrics::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum DatabaseMetrics {
    Table,
    Id,
    QueryHash,
    QueryType,
    TableName,
    ExecutionTimeMs,
    RowsAffected,
    ErrorMessage,
    Timestamp,
    CreatedAt,
}