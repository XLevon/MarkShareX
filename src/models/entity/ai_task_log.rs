use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "ai_task_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub task_id: i32,
    pub status: String,
    #[sea_orm(column_type = "Text")]
    pub steps: String,
    #[sea_orm(column_type = "Text")]
    pub final_reply: String,
    pub error: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::ai_task::Entity",
        from = "Column::TaskId",
        to = "super::ai_task::Column::Id"
    )]
    AiTask,
}

impl Related<super::ai_task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AiTask.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
