//! Import service — business logic for Markdown import post creation.
//!
//! Extracted from `controllers/import_export.rs` to keep the controller
//! focused on HTTP protocol concerns (request parsing, response building).

use crate::models::entity::{categories, post_tags, posts, tags};
use crate::utils::{AppError, AppState};
use sea_orm::*;

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
) -> Result<posts::Model, AppError> {
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

    let category_id = if let Some(name) = category_name {
        if let Some(category) = categories::Entity::find()
            .filter(categories::Column::Name.eq(name))
            .one(&transaction)
            .await?
        {
            Some(category.id)
        } else {
            Some(
                categories::ActiveModel {
                    name: Set(name.to_string()),
                    slug: Set(crate::services::posts::generate_slug(name)),
                    ..Default::default()
                }
                .insert(&transaction)
                .await?
                .id,
            )
        }
    } else {
        None
    };

    // 有明确 slug：用它去重；无 slug：从 title 生成并自动去重
    let slug = if let Some(s) = explicit_slug {
        if posts::Entity::find()
            .filter(posts::Column::Slug.eq(s))
            .filter(posts::Column::DeletedAt.is_null())
            .one(&transaction)
            .await?
            .is_some()
        {
            transaction.rollback().await?;
            return Err(AppError::BadRequest("文章已存在，跳过导入！".to_string()));
        }
        s.to_string()
    } else {
        let base = crate::services::posts::generate_slug(title);
        let mut slug = base.clone();
        let mut counter = 2;
        while posts::Entity::find()
            .filter(posts::Column::Slug.eq(&slug))
            .filter(posts::Column::DeletedAt.is_null())
            .one(&transaction)
            .await?
            .is_some()
        {
            slug = format!("{}-{}", base, counter);
            counter += 1;
        }
        slug
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

    for tag_name in tags {
        let tag_id = if let Some(tag) = tags::Entity::find()
            .filter(tags::Column::Name.eq(tag_name))
            .one(&transaction)
            .await?
        {
            tag.id
        } else {
            let slug = crate::services::posts::generate_slug(tag_name);
            tags::ActiveModel {
                name: Set(tag_name.to_string()),
                slug: Set(slug),
                user_id: Set(Some(user_id)),
                deleted_at: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .insert(&transaction)
            .await?
            .id
        };
        post_tags::ActiveModel {
            post_id: Set(post.id),
            tag_id: Set(tag_id),
        }
        .insert(&transaction)
        .await?;
    }

    transaction.commit().await?;
    Ok(post)
}
