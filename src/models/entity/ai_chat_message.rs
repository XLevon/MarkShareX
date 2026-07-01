use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "ai_chat_messages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub session_id: i32,
    pub role: String,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub tool_calls: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::ai_chat_session::Entity",
        from = "Column::SessionId",
        to = "super::ai_chat_session::Column::Id"
    )]
    Session,
}

impl Related<super::ai_chat_session::Entity> for Entity {
    fn to() -> RelationDef { Relation::Session.def() }
}

impl ActiveModelBehavior for ActiveModel {}
