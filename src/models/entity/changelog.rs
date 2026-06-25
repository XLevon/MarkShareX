use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "changelog")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    /// 版本号，如 "v0.2.0"；草稿时为空字符串
    pub version: String,
    /// Markdown 格式的更新说明，支持多行
    #[sea_orm(column_type = "Text")]
    pub content: String,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
