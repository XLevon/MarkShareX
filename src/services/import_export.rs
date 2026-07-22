//! Import service — business logic for Markdown import post creation.
//!
//! Extracted from `controllers/import_export.rs` to keep the controller
//! focused on HTTP protocol concerns (request parsing, response building).

use crate::models::entity::{categories, post_tags, posts, tags};
use crate::utils::{AppError, AppState};
use sea_orm::*;

pub struct ImportPostError {
    pub error: AppError,
    pub rollback_files: bool,
    pub persisted_post_id: Option<i32>,
}

impl From<AppError> for ImportPostError {
    fn from(error: AppError) -> Self {
        Self {
            error,
            rollback_files: true,
            persisted_post_id: None,
        }
    }
}

impl From<DbErr> for ImportPostError {
    fn from(error: DbErr) -> Self {
        AppError::DbError(error).into()
    }
}

async fn compensate_committed_import(
    db: &DatabaseConnection,
    post_id: i32,
    created_category_id: Option<i32>,
    created_tag_ids: &[i32],
) -> Result<(), AppError> {
    let transaction = db.begin().await?;
    post_tags::Entity::delete_many()
        .filter(post_tags::Column::PostId.eq(post_id))
        .exec(&transaction)
        .await?;
    let deleted = posts::Entity::delete_by_id(post_id)
        .exec(&transaction)
        .await?;
    if deleted.rows_affected != 1 {
        transaction.rollback().await?;
        return Err(AppError::Internal(anyhow::anyhow!(
            "导入索引失败后的数据库补偿未找到文章 {post_id}"
        )));
    }

    for tag_id in created_tag_ids {
        let references = post_tags::Entity::find()
            .filter(post_tags::Column::TagId.eq(*tag_id))
            .count(&transaction)
            .await?;
        if references == 0 {
            tags::Entity::delete_by_id(*tag_id)
                .exec(&transaction)
                .await?;
        }
    }
    if let Some(category_id) = created_category_id {
        let references = posts::Entity::find()
            .filter(posts::Column::CategoryId.eq(category_id))
            .count(&transaction)
            .await?;
        if references == 0 {
            categories::Entity::delete_by_id(category_id)
                .exec(&transaction)
                .await?;
        }
    }
    transaction.commit().await?;
    Ok(())
}

async fn available_import_slug<C>(connection: &C, base: &str) -> Result<String, DbErr>
where
    C: ConnectionTrait,
{
    let mut slug = base.to_string();
    let mut counter = 2;
    while posts::Entity::find()
        .filter(posts::Column::Slug.eq(&slug))
        .one(connection)
        .await?
        .is_some()
    {
        slug = format!("{base}-{counter}");
        counter += 1;
    }
    Ok(slug)
}

/// Create a post from imported Markdown content inside a single DB
/// transaction.  Category lookup/creation, slug generation/dedup,
/// post insertion, and tag association all happen atomically.
pub async fn create_post_from_import(
    state: &AppState,
    user_id: i32,
    title: &str,
    content: &str,
    summary: Option<&str>,
    category_name: Option<&str>,
    status: &str,
    tags: &[String],
    explicit_slug: Option<&str>,
    cover_url: Option<&str>,
) -> Result<posts::Model, ImportPostError> {
    let content_html = Some(crate::services::posts::render_markdown(&state.db, content).await);
    let now = crate::utils::now_local();
    let published_at = if status == "published" {
        Some(now)
    } else {
        None
    };

    // Import only passes nr:{id} here, so this validates an existing resource without creating one.
    let cover_network_id = if let Some(url) = cover_url {
        if url.starts_with("nr:") {
            crate::controllers::network_resources::ensure_url(&state.db, url).await?
        } else {
            None
        }
    } else {
        None
    };
    let (cover_image_url, cover_image_filename) = match cover_url {
        Some(url)
            if url.starts_with("nr:")
                || url.starts_with("http://")
                || url.starts_with("https://") =>
        {
            (Some(url.to_string()), None)
        }
        Some(name) => (None, Some(name.to_string())),
        None => (None, None),
    };

    let transaction = state.db.begin().await?;

    let mut created_category_id = None;
    let category_id = if let Some(name) = category_name {
        if let Some(category) = categories::Entity::find()
            .filter(categories::Column::Name.eq(name))
            .one(&transaction)
            .await?
        {
            Some(category.id)
        } else {
            let category = categories::ActiveModel {
                name: Set(name.to_string()),
                slug: Set(crate::services::posts::generate_slug(name)),
                ..Default::default()
            }
            .insert(&transaction)
            .await?;
            created_category_id = Some(category.id);
            Some(category.id)
        }
    } else {
        None
    };

    // 有明确 slug：用它去重；无 slug：从 title 生成并自动去重
    let slug = if let Some(s) = explicit_slug {
        if posts::Entity::find()
            .filter(posts::Column::Slug.eq(s))
            .one(&transaction)
            .await?
            .is_some()
        {
            transaction.rollback().await?;
            return Err(AppError::BadRequest("文章已存在，跳过导入！".to_string()).into());
        }
        s.to_string()
    } else {
        let base = crate::services::posts::generate_slug_base(title);
        let base = if base.is_empty() { "post" } else { &base };
        available_import_slug(&transaction, base).await?
    };

    let post = posts::ActiveModel {
        user_id: Set(user_id),
        category_id: Set(category_id),
        title: Set(title.to_string()),
        slug: Set(slug),
        summary: Set(summary.map(|s| s.to_string())),
        content: Set(Some(content.to_string())),
        content_html: Set(content_html),
        cover_image: Set(cover_url.map(|s| s.to_string())),
        cover_image_url: Set(cover_image_url),
        cover_image_filename: Set(cover_image_filename),
        cover_network_id: Set(cover_network_id),
        status: Set(status.to_string()),
        post_type: Set("post".to_string()),
        is_pinned: Set(false),
        allow_comment: Set(true),
        sort_order: Set(0),
        view_count: Set(0),
        like_count: Set(0),
        comment_count: Set(0),
        published_at: Set(published_at),
        deleted_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&transaction)
    .await?;

    let mut created_tag_ids = Vec::new();
    for tag_name in tags {
        let tag_id = if let Some(tag) = tags::Entity::find()
            .filter(tags::Column::Name.eq(tag_name))
            .one(&transaction)
            .await?
        {
            tag.id
        } else {
            let slug = crate::services::posts::generate_slug(tag_name);
            let tag = tags::ActiveModel {
                name: Set(tag_name.to_string()),
                slug: Set(slug),
                user_id: Set(Some(user_id)),
                deleted_at: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .insert(&transaction)
            .await?;
            created_tag_ids.push(tag.id);
            tag.id
        };
        post_tags::ActiveModel {
            post_id: Set(post.id),
            tag_id: Set(tag_id),
        }
        .insert(&transaction)
        .await?;
    }

    transaction.commit().await?;
    if post.status == "published" {
        let result = state.search_engine.index_document(
            post.id as u64,
            &post.title,
            post.content.as_deref().unwrap_or_default(),
        );
        if let Err(index_error) = crate::services::search::ensure_search_index_consistency(
            &state.search_engine,
            &state.db,
            post.id,
            result,
        )
        .await
        {
            return match compensate_committed_import(
                &state.db,
                post.id,
                created_category_id,
                &created_tag_ids,
            )
            .await
            {
                Ok(()) => Err(ImportPostError {
                    error: index_error,
                    rollback_files: true,
                    persisted_post_id: None,
                }),
                Err(compensation_error) => Err(ImportPostError {
                    error: AppError::Internal(anyhow::anyhow!(
                        "文章 {} 的搜索索引失败，数据库补偿也失败: {}; 原始错误: {}",
                        post.id,
                        compensation_error,
                        index_error
                    )),
                    rollback_files: false,
                    persisted_post_id: Some(post.id),
                }),
            };
        }
    }
    Ok(post)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::entity::users;

    #[tokio::test]
    async fn generated_import_slug_skips_soft_deleted_global_collision() -> anyhow::Result<()> {
        let db = Database::connect("sqlite::memory:").await?;
        crate::models::run_migrations(&db).await?;
        let now = crate::utils::now_local();
        let user = users::ActiveModel {
            username: Set("slug-test-user".into()),
            email: Set("slug-test@example.com".into()),
            password_hash: Set("unused".into()),
            role: Set("author".into()),
            is_active: Set(true),
            status: Set("active".into()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await?;
        posts::ActiveModel {
            user_id: Set(user.id),
            title: Set("retained deleted post".into()),
            slug: Set("generated-base".into()),
            status: Set("draft".into()),
            post_type: Set("post".into()),
            is_pinned: Set(false),
            allow_comment: Set(true),
            sort_order: Set(0),
            view_count: Set(0),
            like_count: Set(0),
            comment_count: Set(0),
            article_type: Set("original".into()),
            article_status: Set("latest".into()),
            deleted_at: Set(Some(now)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await?;

        assert_eq!(
            available_import_slug(&db, "generated-base").await?,
            "generated-base-2"
        );
        Ok(())
    }
}
