use sea_orm::*;
use sea_orm::sea_query::Expr;
use crate::models::entity::{posts, categories, tags, post_tags, comments};
use crate::models::entity::network_resources;
use crate::utils::{AppError, Pagination};
use regex::Regex;

pub async fn list_posts(
    db: &DatabaseConnection,
    page: u64,
    page_size: u64,
    status: Option<&str>,
    category_id: Option<i32>,
    category_ids: Option<&[i32]>,
    tag_id: Option<i32>,
    tag_search: Option<&str>,
    author_id: Option<i32>,
    exclude_category_ids: Option<&[i32]>,
    article_type: Option<&str>,
    article_types: Option<&[String]>,
    article_status: Option<&str>,
    article_statuses: Option<&[String]>,
    is_pinned: Option<bool>,
    include_subcategories: Option<bool>,
    search: Option<&str>,
) -> Result<(Vec<posts::Model>, Pagination), AppError> {
    let mut condition = Condition::all()
        .add(posts::Column::DeletedAt.is_null());

    if let Some(s) = status {
        condition = condition.add(posts::Column::Status.eq(s));
    }
    // Merge category_id + category_ids
    let merged_category_ids: Vec<i32> = {
        let mut ids: Vec<i32> = category_ids.map(|v| v.to_vec()).unwrap_or_default();
        if let Some(cid) = category_id {
            if !ids.contains(&cid) { ids.push(cid); }
        }
        ids
    };
    if !merged_category_ids.is_empty() {
        let mut all_ids = merged_category_ids.clone();
        if include_subcategories.unwrap_or(false) {
            let children = categories::Entity::find()
                .filter(categories::Column::ParentId.is_in(merged_category_ids.clone()))
                .filter(categories::Column::DeletedAt.is_null())
                .all(db)
                .await?;
            all_ids.extend(children.iter().map(|c| c.id));
        }
        condition = condition.add(posts::Column::CategoryId.is_in(all_ids));
    }
    if let Some(query) = search {
        if !query.is_empty() {
            condition = condition.add(posts::Column::Title.like(&format!("%{}%", query)));
        }
    }
    if let Some(tid) = tag_id {
        let subquery = post_tags::Entity::find()
            .select_only()
            .column(post_tags::Column::PostId)
            .filter(post_tags::Column::TagId.eq(tid))
            .into_query();
        condition = condition.add(Expr::col(posts::Column::Id).in_subquery(subquery));
    }
    // Tag fuzzy search: find posts that have any tag whose name matches
    if let Some(tag_name) = tag_search {
        if !tag_name.is_empty() {
            let tag_ids_query = tags::Entity::find()
                .select_only()
                .column(tags::Column::Id)
                .filter(tags::Column::Name.like(&format!("%{}%", tag_name)))
                .into_query();
            let pt_subquery = post_tags::Entity::find()
                .select_only()
                .column(post_tags::Column::PostId)
                .filter(Expr::col(post_tags::Column::TagId).in_subquery(tag_ids_query))
                .into_query();
            condition = condition.add(Expr::col(posts::Column::Id).in_subquery(pt_subquery));
        }
    }
    if let Some(aid) = author_id {
        condition = condition.add(posts::Column::UserId.eq(aid));
    }
    if let Some(ids) = exclude_category_ids {
        if !ids.is_empty() {
            condition = condition.add(posts::Column::CategoryId.is_not_in(ids.iter().copied()));
        }
    }
    // Merge article_type + article_types
    if let Some(types) = article_types {
        if !types.is_empty() {
            let mut all = types.to_vec();
            if let Some(at) = article_type {
                if !all.contains(&at.to_string()) { all.push(at.to_string()); }
            }
            condition = condition.add(posts::Column::ArticleType.is_in(all));
        }
    } else if let Some(at) = article_type {
        condition = condition.add(posts::Column::ArticleType.eq(at));
    }
    // Merge article_status + article_statuses
    if let Some(statuses) = article_statuses {
        if !statuses.is_empty() {
            let mut all = statuses.to_vec();
            if let Some(as_) = article_status {
                if !all.contains(&as_.to_string()) { all.push(as_.to_string()); }
            }
            condition = condition.add(posts::Column::ArticleStatus.is_in(all));
        }
    } else if let Some(as_) = article_status {
        condition = condition.add(posts::Column::ArticleStatus.eq(as_));
    }
    if let Some(pinned) = is_pinned {
        condition = condition.add(posts::Column::IsPinned.eq(pinned));
    }

    let total = posts::Entity::find()
        .filter(condition.clone())
        .count(db)
        .await?;

    // Exclude content/content_html from list queries — these are large TEXT fields
    // that live in SQLite overflow pages. Only get_post/get_post_by_slug should fetch them.
    let items: Vec<posts::Model> = {
        let mut query = posts::Entity::find()
            .select_only()
            .column(posts::Column::Id)
            .column(posts::Column::UserId)
            .column(posts::Column::CategoryId)
            .column(posts::Column::Title)
            .column(posts::Column::Slug)
            .column(posts::Column::Summary)
            .column(posts::Column::CoverImage)
            .column(posts::Column::CoverImageUrl)
            .column(posts::Column::CoverImageFilename)
            .column(posts::Column::CoverNetworkId)
            .column(posts::Column::Status)
            .column(posts::Column::PostType)
            .column(posts::Column::IsPinned)
            .column(posts::Column::AllowComment)
            .column(posts::Column::SortOrder)
            .column(posts::Column::ViewCount)
            .column(posts::Column::LikeCount)
            .column(posts::Column::CommentCount)
            .column(posts::Column::ArticleType)
            .column(posts::Column::ArticleStatus)
            .column(posts::Column::PublishedAt)
            .column(posts::Column::DeletedAt)
            .column(posts::Column::CreatedAt)
            .column(posts::Column::UpdatedAt)
            .filter(condition);
        if is_pinned == Some(true) {
            query = query.order_by_asc(posts::Column::SortOrder);
        } else {
            // 已发布文章按发布时间排序，草稿(published_at=NULL)回退到创建时间
            query = query
                .order_by_desc(Expr::cust("COALESCE(published_at, created_at)"))
                .order_by_desc(posts::Column::Id);
        }
        query
            .offset(Some((page - 1) * page_size))
            .limit(Some(page_size))
            .into_json()
            .all(db)
            .await?
            .into_iter()
            .map(|v| serde_json::from_value(v).unwrap_or_default())
            .collect()
    };

    Ok((items, Pagination::new(total, page, page_size)))
}

pub async fn get_post(db: &DatabaseConnection, id: i32) -> Result<posts::Model, AppError> {
    let post = posts::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(AppError::NotFound("文章不存在".to_string()))?;
    if post.deleted_at.is_some() {
        return Err(AppError::NotFound("文章不存在".to_string()));
    }
    Ok(post)
}

pub async fn get_post_by_slug(db: &DatabaseConnection, slug: &str) -> Result<posts::Model, AppError> {
    let post = posts::Entity::find()
        .filter(posts::Column::Slug.eq(slug))
        .filter(posts::Column::DeletedAt.is_null())
        .one(db)
        .await?
        .ok_or(AppError::NotFound("文章不存在".to_string()))?;
    Ok(post)
}

pub async fn delete_post(db: &DatabaseConnection, id: i32) -> Result<(), AppError> {
    // 1. 删除文章标签关联
    post_tags::Entity::delete_many()
        .filter(post_tags::Column::PostId.eq(id))
        .exec(db)
        .await?;
    
    // 2. 删除点赞
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!("DELETE FROM likes WHERE post_id = {}", id)
    );
    db.execute(stmt).await?;
    
    // 3. 删除文章评论
    comments::Entity::delete_many()
        .filter(comments::Column::PostId.eq(id))
        .exec(db)
        .await?;
    
    // 4. 删除文章阅读日志
    let stmt = Statement::from_string(
        db.get_database_backend(),
        format!("DELETE FROM read_logs WHERE post_id = {}", id)
    );
    db.execute(stmt).await?;
    
    // 5. 硬删除文章
    posts::Entity::delete_by_id(id)
        .exec(db)
        .await?;
    
    Ok(())
}

pub async fn get_category_name(db: &DatabaseConnection, category_id: Option<i32>) -> Option<String> {
    if let Some(cid) = category_id {
        categories::Entity::find_by_id(cid)
            .one(db)
            .await
            .ok()
            .flatten()
            .map(|c| c.name)
    } else {
        None
    }
}

pub async fn get_post_tags(db: &DatabaseConnection, post_id: i32) -> Result<Vec<tags::Model>, AppError> {
    let tag_ids: Vec<i32> = post_tags::Entity::find()
        .filter(post_tags::Column::PostId.eq(post_id))
        .all(db)
        .await?
        .into_iter()
        .map(|pt| pt.tag_id)
        .collect();

    if tag_ids.is_empty() {
        return Ok(Vec::new());
    }

    let tags = tags::Entity::find()
        .filter(tags::Column::Id.is_in(tag_ids))
        .all(db)
        .await?;

    Ok(tags)
}

pub async fn set_post_tags(db: &DatabaseConnection, post_id: i32, tag_ids: &[i32]) -> Result<(), AppError> {
    use sea_orm::ConnectionTrait;

    // Delete existing tags
    post_tags::Entity::delete_many()
        .filter(post_tags::Column::PostId.eq(post_id))
        .exec(db)
        .await?;

    // SQLite: INSERT OR IGNORE 不忽略 FK 约束（SQLite 文档明确），需要临时关闭 FK 检查
    // post_id 和 tag_id 均已在本事务中创建/确认存在，关闭 FK 是安全的
    db.execute(sea_orm::Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "PRAGMA foreign_keys = OFF",
    )).await?;

    for &tid in tag_ids {
        db.execute(sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "INSERT INTO post_tags (post_id, tag_id) VALUES (?, ?)",
            [post_id.into(), tid.into()],
        ))
        .await
        .map_err(|e| {
            tracing::error!("set_post_tags: insert error for post={} tag={}: {:?}", post_id, tid, e);
            AppError::DbError(e)
        })?;
    }

    db.execute(sea_orm::Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "PRAGMA foreign_keys = ON",
    )).await?;

    Ok(())
}

/// Resolve tag names to tag IDs. Creates new tags for names that don't exist.
pub async fn resolve_tag_names(db: &DatabaseConnection, tag_names: &[String]) -> Result<Vec<i32>, AppError> {
    let now = crate::utils::now_local();
    let mut tag_ids = Vec::new();

    for name in tag_names {
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        // Find or create tag
        let tag = tags::Entity::find()
            .filter(tags::Column::Name.eq(name))
            .filter(tags::Column::DeletedAt.is_null())
            .one(db)
            .await?;

        let tag_id = if let Some(existing) = tag {
            existing.id
        } else {
            // Create new tag
            let slug = generate_slug(name);
            let model = tags::ActiveModel {
                name: Set(name.to_string()),
                slug: Set(slug),
                user_id: Set(None),  // 明确设置 NULL 避免 FK 约束问题
                deleted_at: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            };
            let created = model.insert(db).await
                .map_err(|e| {
                    tracing::error!("resolve_tag_names: failed to insert tag '{}': {:?}", name, e);
                    AppError::DbError(e)
                })?;
            created.id
        };
        tag_ids.push(tag_id);
    }

    Ok(tag_ids)
}

/// Get previous and next published posts for navigation.
pub async fn get_adjacent_posts(
    db: &DatabaseConnection,
    post_id: i32,
) -> Result<(Option<(i32, String, String)>, Option<(i32, String, String)>), AppError> {
    // Get current post's published_at
    let current = posts::Entity::find_by_id(post_id)
        .filter(posts::Column::DeletedAt.is_null())
        .one(db)
        .await?
        .ok_or(AppError::NotFound("文章不存在".to_string()))?;

    let published_at = current.published_at.ok_or(AppError::NotFound("文章未发布".to_string()))?;

    // Previous: max published_at < current, order by published_at DESC
    let prev = posts::Entity::find()
        .filter(posts::Column::DeletedAt.is_null())
        .filter(posts::Column::Status.eq("published"))
        .filter(posts::Column::PublishedAt.lt(published_at))
        .order_by_desc(posts::Column::PublishedAt)
        .one(db)
        .await?;

    // Next: min published_at > current, order by published_at ASC
    let next = posts::Entity::find()
        .filter(posts::Column::DeletedAt.is_null())
        .filter(posts::Column::Status.eq("published"))
        .filter(posts::Column::PublishedAt.gt(published_at))
        .order_by_asc(posts::Column::PublishedAt)
        .one(db)
        .await?;

    Ok((
        prev.map(|p| (p.id, p.title, p.slug)),
        next.map(|p| (p.id, p.title, p.slug)),
    ))
}

pub async fn render_markdown(db: &DatabaseConnection, content: &str) -> String {
    // 预处理：./uploads/ 替换为 /uploads/（相对路径，浏览器自动适配当前域名）
    let content = content.replace("](./uploads/", "](/uploads/");

    // 🔴 预处理：解析 nr:{id} → 真实 URL（必须在 comrak/ammonia 之前，否则 ammonia 会洗掉 src="nr:1"）
    let nr_re = Regex::new(r"!\[([^\]]*)\]\(nr:(\d+)\)").unwrap();
    let mut resolved = content.clone();
    for cap in nr_re.captures_iter(&content) {
        let full = cap.get(0).unwrap().as_str();
        let alt = cap.get(1).unwrap().as_str();
        let id_str = cap.get(2).unwrap().as_str();
        if let Ok(id) = id_str.parse::<i32>() {
            if let Some(nr) = network_resources::Entity::find_by_id(id).one(db).await.ok().flatten() {
                resolved = resolved.replace(full, &format!("![{}]({})", alt, nr.url));
            }
        }
    }

    let mut options = comrak::Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.shortcodes = true;
    options.extension.superscript = true;
    let html = comrak::markdown_to_html(&resolved, &options);
    
    // Allow class attribute on code/pre for syntax highlighting
    let mut builder = ammonia::Builder::default();
    builder.add_tag_attributes("code", &["class"]);
    builder.add_tag_attributes("pre", &["class"]);
    // Allow referrerpolicy on img for anti-hotlinking
    builder.add_tag_attributes("img", &["src", "alt", "title", "width", "height", "loading", "referrerpolicy"]);
    let safe_html = builder.clean(&html).to_string();
    
    // Add referrerpolicy to all img tags (prevents hotlink blocking from sites like CSDN)
    let re = Regex::new(r"<img\s").unwrap();
    re.replace_all(&safe_html, r#"<img referrerpolicy="no-referrer" "#).to_string()
}

pub fn generate_slug(title: &str) -> String {
    use nanoid::nanoid;

    let slugified: String = title
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() || c == '_' || c == '-' {
                '-'
            } else {
                '-'
            }
        })
        .collect();

    // Remove consecutive dashes
    let mut result = String::new();
    let mut prev_dash = false;
    for c in slugified.chars() {
        if c == '-' {
            if !prev_dash { result.push(c); }
            prev_dash = true;
        } else {
            result.push(c);
            prev_dash = false;
        }
    }
    let base = result.trim_matches('-').to_string();

    // Truncate meaningful prefix, then append fixed-length Nano ID
    // for guaranteed uniqueness. Total length ≤ 40 + 1 + 10 = 51.
    let prefix = if base.len() > 40 {
        let mut p = base[..40].to_string();
        if let Some(i) = p.rfind('-') {
            if i > 20 { p.truncate(i); }  // word boundary
        }
        p.trim_matches('-').to_string()
    } else {
        base
    };

    if prefix.is_empty() {
        format!("post-{}", nanoid!(10))
    } else {
        format!("{}-{}", prefix, nanoid!(10))
    }
}
