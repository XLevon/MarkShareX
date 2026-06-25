use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, Default)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub category_id: Option<i32>,
    pub title: String,
    #[sea_orm(unique)]
    pub slug: String,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub content_html: Option<String>,
    pub cover_image: Option<String>,          // 🔒 旧字段，历史兼容
    pub cover_image_url: Option<String>,      // 🆕 nr:ID 或外部 URL
    pub cover_image_filename: Option<String>, // 🆕 本地文件名
    pub cover_network_id: Option<i32>,
    pub status: String,
    pub post_type: String,
    pub is_pinned: bool,
    pub allow_comment: bool,
    pub sort_order: i32,
    pub view_count: i32,
    pub like_count: i32,
    pub comment_count: i32,
    pub article_type: String,
    pub article_status: String,
    pub published_at: Option<chrono::NaiveDateTime>,
    pub deleted_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
