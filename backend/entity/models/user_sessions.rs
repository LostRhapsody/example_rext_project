//! `SeaORM` Entity for user_sessions table

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[sea_orm(table_name = "user_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[schema(value_type = String)]
    pub id: Uuid,
    #[schema(value_type = String)]
    pub user_id: Uuid,
    pub session_token: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    #[schema(value_type = String)]
    pub created_at: Option<DateTimeWithTimeZone>,
    #[schema(value_type = String)]
    pub last_activity: Option<DateTimeWithTimeZone>,
    #[schema(value_type = String)]
    pub expires_at: DateTimeWithTimeZone,
    pub is_active: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    Users,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
