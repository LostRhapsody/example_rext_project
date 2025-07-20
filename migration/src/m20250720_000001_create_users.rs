use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
      "m20250720_000001_create_users"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .col(
                      ColumnDef::new(Users::Id)
                        .uuid()
                        .not_null()
                        .primary_key()
                    )
                    .col(
                      ColumnDef::new(Users::Email)
                        .string()
                        .not_null()
                        .unique_key()
                    )
                    .col(
                      ColumnDef::new(Users::PasswordHash)
                        .string()
                        .not_null()
                    )
                    .col(
                      ColumnDef::new(Users::CreatedAt)
                        .timestamp_with_time_zone()
                        .default(Expr::current_timestamp())
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Email,
    PasswordHash,
    CreatedAt,
}