use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "database_metrics")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub query_hash: String,
    pub query_type: String,
    pub table_name: Option<String>,
    pub execution_time_ms: i64,
    pub rows_affected: Option<i64>,
    pub error_message: Option<String>,
    pub timestamp: DateTimeWithTimeZone,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
