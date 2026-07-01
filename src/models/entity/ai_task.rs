use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "ai_tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub skill_id: i32,
    pub provider_id: Option<i32>,
    pub cron_expr: String,
    #[sea_orm(column_type = "Text")]
    pub params: String,
    pub enabled: bool,
    pub last_run_at: Option<chrono::NaiveDateTime>,
    pub run_count: i32,
    pub agent_config_id: Option<i32>,
    pub model_id: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::ai_skill::Entity",
        from = "Column::SkillId",
        to = "super::ai_skill::Column::Id"
    )]
    Skill,
    #[sea_orm(
        belongs_to = "super::ai_provider::Entity",
        from = "Column::ProviderId",
        to = "super::ai_provider::Column::Id"
    )]
    Provider,
}

impl Related<super::ai_skill::Entity> for Entity {
    fn to() -> RelationDef { Relation::Skill.def() }
}
impl Related<super::ai_provider::Entity> for Entity {
    fn to() -> RelationDef { Relation::Provider.def() }
}

impl ActiveModelBehavior for ActiveModel {}
