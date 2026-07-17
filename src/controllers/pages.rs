use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
};
use crate::utils::{AppState, AppError};
use serde_json::json;
use sea_orm::*;
use std::collections::HashSet;
use std::error::Error as _;
use crate::models::entity::{
    article_statuses, article_types, categories, posts, tags, users,
};

const DEFAULT_OG_IMAGE: &[u8] = include_bytes!("../../assets/default-og.png");

async fn get_setting(db: &sea_orm::DatabaseConnection, key: &str) -> Option<String> {
    crate::models::entity::settings::Entity::find()
        .filter(crate::models::entity::settings::Column::Key.eq(key))
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|s| s.value)
}

fn display_site_title(raw_title: &str) -> String {
    let title = raw_title.replace('-', "");
    let title = title.trim();
    if title.is_empty() {
        "Mark-Share-X_用AI学AI".to_string()
    } else {
        title.to_string()
    }
}

fn schema_datetime(value: chrono::NaiveDateTime) -> String {
    use chrono::{SecondsFormat, TimeZone};

    chrono::Local
        .from_local_datetime(&value)
        .earliest()
        .map(|datetime| datetime.to_rfc3339_opts(SecondsFormat::Secs, false))
        .unwrap_or_else(|| value.format("%Y-%m-%dT%H:%M:%S").to_string())
}

fn absolute_url(value: &str, base_url: &str) -> String {
    let value = value.trim();
    if value.starts_with("http://") || value.starts_with("https://") {
        value.to_string()
    } else if value.starts_with('/') {
        format!("{base_url}{value}")
    } else {
        format!("{base_url}/{value}")
    }
}

fn build_social_image_url(cover_image: Option<&str>, base_url: &str) -> String {
    match cover_image.map(str::trim).filter(|value| !value.is_empty()) {
        Some(url) => absolute_url(url, base_url),
        None => format!("{base_url}/default-og.png"),
    }
}

pub async fn post_detail(
    State(state): State<AppState>,
    Path(param): Path<String>,
    headers: HeaderMap,
) -> Result<Html<String>, AppError> {
    // Support both numeric ID and slug: pure digits → ID lookup, otherwise → slug lookup
    let post = if param.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(id) = param.parse::<i32>() {
            crate::services::posts::get_post(&state.db, id).await?
        } else {
            crate::services::posts::get_post_by_slug(&state.db, &param).await?
        }
    } else {
        crate::services::posts::get_post_by_slug(&state.db, &param).await?
    };

    let post_id = post.id;
    // Real view count from read_logs
    let view_count: i64 = state.db.query_one(
        sea_orm::Statement::from_string(
            state.db.get_database_backend(),
            format!("SELECT COUNT(*) FROM read_logs WHERE post_id = {}", post_id),
        ),
    ).await.ok().flatten().and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0);

    let category = if let Some(category_id) = post.category_id {
        categories::Entity::find_by_id(category_id)
            .one(&state.db)
            .await?
    } else {
        None
    };
    let category_name = category.as_ref().map(|value| value.name.clone());
    let tags = crate::services::posts::get_post_tags(&state.db, post.id).await?;

    let raw_site_title = get_setting(&state.db, "site_title")
        .await
        .unwrap_or_else(|| "Mark-Share-X_用AI学AI".to_string());
    let site_title = display_site_title(&raw_site_title);
    let site_description = get_setting(&state.db, "site_description").await.unwrap_or_default();
    let configured_site_logo = get_setting(&state.db, "site_logo")
        .await
        .filter(|value| !value.trim().is_empty());
    let guest_copy_enabled = get_setting(&state.db, "guest_copy_enabled")
        .await
        .as_deref()
        != Some("false");
    let author = users::Entity::find_by_id(post.user_id).one(&state.db).await?;
    let author_name = author
        .as_ref()
        .and_then(|value| value.display_name.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| author.as_ref().map(|value| value.username.clone()))
        .unwrap_or_else(|| site_title.clone());

    let mut cover_image = crate::controllers::network_resources::resolve_post_cover(
        &state.db,
        post.cover_network_id,
        post.cover_image_url.as_deref(),
        post.cover_image_filename.as_deref(),
        post.cover_image.as_deref(),
    ).await;

    if cover_image.is_none() {
        if let Some(category) = category.as_ref() {
            cover_image = crate::controllers::network_resources::resolve_cover_url(
                &state.db,
                category.network_resource_id,
                category.image_url.as_deref(),
                category.image_filename.as_deref(),
            ).await;
        }
    }

    // Derive base_url from request Host header (for OG meta tags)
    let scheme = if headers.get("x-forwarded-proto").and_then(|v| v.to_str().ok()) == Some("https") {
        "https"
    } else {
        "http"
    };
    let raw_host = headers.get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or(&state.config.server.host)
        .to_string();
    // Strip port from host header if present (Host: localhost:5023 → localhost)
    let host_only = raw_host.split(':').next().unwrap_or(&raw_host);
    let base_url = if host_only == "localhost" || host_only.starts_with("127.") || host_only.starts_with("192.168.") {
        format!("{}://{}:{}", scheme, host_only, state.config.server.port)
    } else {
        format!("{}://{}", scheme, raw_host)  // keep port in Host header for proxied requests
    };

    // Make the real article/category cover absolute for templates and social metadata.
    if let Some(ref ci) = cover_image {
        if !ci.starts_with("http://") && !ci.starts_with("https://") {
            cover_image = Some(build_social_image_url(Some(ci), &base_url));
        }
    }
    let social_image = build_social_image_url(cover_image.as_deref(), &base_url);
    let publisher_logo = configured_site_logo
        .as_deref()
        .map(|value| absolute_url(value, &base_url))
        .unwrap_or_else(|| format!("{base_url}/favicon.png"));
    let author_url = if author.is_some() {
        format!("{base_url}/author/{}", post.user_id)
    } else {
        base_url.clone()
    };
    let date_published = schema_datetime(post.published_at.unwrap_or(post.created_at));
    let date_modified = schema_datetime(post.updated_at);

    // Build or resolve content_html
    let needs_render = post.content_html.as_deref().map_or(true, |s| {
        s.is_empty() || (s.contains("<img") && !s.contains("referrerpolicy"))
    });
    let raw_html = if needs_render {
        // content_html is missing/empty/outdated — render from content markdown
        crate::services::posts::render_markdown(&state.db, &post.content.unwrap_or_default()).await
    } else {
        post.content_html.unwrap_or_default()
    };

    // Resolve nr:{id} → real URLs
    let mut resolved_html = crate::controllers::network_resources::resolve_nr_in_content(
        &state.db,
        &raw_html,
    ).await;

    // Resolve ./uploads/ → /uploads/ (root-relative) for images
    resolved_html = resolved_html.replace("./uploads/", "/uploads/");

    // Keep the article title as the page's only H1 without changing stored content.
    resolved_html = normalize_article_headings(&resolved_html, &post.title);

    // Add heading IDs for anchor link support (comrak doesn't auto-generate them)
    resolved_html = add_heading_ids(&resolved_html);

    let article_url = format!("{}/post/{}", base_url, post.slug);
    let meta_description = build_meta_description(
        post.summary.as_deref(),
        &resolved_html,
        &site_description,
        &post.title,
    );

    // Query adjacent posts for prev/next navigation
    let (adjacent_prev, adjacent_next) = crate::services::posts::get_adjacent_posts(&state.db, post_id)
        .await
        .unwrap_or_default();

    // Query related posts: same category, excluding current, limit 5
    use crate::models::entity::posts as posts_entity;
    use sea_orm::{QueryFilter, QueryOrder, ColumnTrait, EntityTrait};
    let related_posts: Vec<(String, String)> = if let Some(cid) = post.category_id {
        posts_entity::Entity::find()
            .filter(posts_entity::Column::CategoryId.eq(cid))
            .filter(posts_entity::Column::Id.ne(post_id))
            .filter(posts_entity::Column::Status.eq("published"))
            .filter(posts_entity::Column::DeletedAt.is_null())
            .order_by_desc(posts_entity::Column::PublishedAt)
            .limit(5)
            .all(&state.db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|p| (p.title, p.slug))
            .collect()
    } else {
        Vec::new()
    };

    let mut ctx = tera::Context::new();
    ctx.insert("site_title", &site_title);
    ctx.insert("site_description", &site_description);
    ctx.insert("guest_copy_enabled", &guest_copy_enabled);
    ctx.insert("base_url", &base_url);
    ctx.insert("article_url", &article_url);
    ctx.insert("meta_description", &meta_description);
    ctx.insert("social_image", &social_image);
    ctx.insert("date_published", &date_published);
    ctx.insert("date_modified", &date_modified);
    ctx.insert("author_name", &author_name);
    ctx.insert("author_url", &author_url);
    ctx.insert("publisher_logo", &publisher_logo);
    ctx.insert("post", &json!({
        "title": post.title,
        "slug": post.slug,
        "content_html": resolved_html,
        "summary": post.summary,
        "cover_image": cover_image,
        "view_count": view_count,
        "published_at": post.published_at.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default(),
        "created_at": post.created_at.format("%Y-%m-%d").to_string(),
        "category_name": category_name,
        "tags": tags.iter().map(|t| t.name.clone()).collect::<Vec<_>>(),
    }));
    ctx.insert("adjacent_prev", &adjacent_prev.map(|(_, title, slug)| json!({"title": title, "slug": slug})));
    ctx.insert("adjacent_next", &adjacent_next.map(|(_, title, slug)| json!({"title": title, "slug": slug})));
    ctx.insert("related_posts", &related_posts.iter().map(|(title, slug)| json!({"title": title, "slug": slug})).collect::<Vec<_>>());
    let tag_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
    ctx.insert("tag_names", &tag_names);

    let html = state
        .tera
        .render("post.html", &ctx)
        .map_err(|e| {
            let mut err_chain = format!("{}", e);
            let mut source = e.source();
            while let Some(s) = source {
                err_chain.push_str(&format!("\n  caused by: {}", s));
                source = s.source();
            }
            AppError::Internal(anyhow::anyhow!("Template error: {}", err_chain))
        })?;

    Ok(Html(html))
}

fn decode_heading_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

/// Normalize article body headings without changing stored article content.
/// A leading H1 that repeats the article title is removed; any remaining body H1
/// is demoted to H2 so the page-level article title remains the only H1.
pub(crate) fn normalize_article_headings(html: &str, article_title: &str) -> String {
    use regex::Regex;
    use std::sync::LazyLock;

    static LEADING_H1_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?is)^\s*<h1(?:\s[^>]*)?>(.*?)</h1>").expect("valid leading H1 regex")
    });
    static HTML_TAG_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?is)<[^>]+>").expect("valid HTML tag regex"));
    static H1_TAG_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)<(/?)h1(\s[^>]*)?>").expect("valid H1 tag regex")
    });

    let mut normalized = html.to_string();
    if let Some(captures) = LEADING_H1_RE.captures(&normalized) {
        let heading_html = captures.get(1).map_or("", |value| value.as_str());
        let heading_text = decode_heading_entities(&HTML_TAG_RE.replace_all(heading_html, ""));
        let heading_text = heading_text.split_whitespace().collect::<Vec<_>>().join(" ");
        let title_text = article_title.split_whitespace().collect::<Vec<_>>().join(" ");

        if heading_text == title_text {
            if let Some(full_match) = captures.get(0) {
                normalized.replace_range(full_match.range(), "");
            }
        }
    }

    H1_TAG_RE
        .replace_all(&normalized, |captures: &regex::Captures| {
            let slash = captures.get(1).map_or("", |value| value.as_str());
            let attrs = captures.get(2).map_or("", |value| value.as_str());
            format!("<{slash}h2{attrs}>")
        })
        .to_string()
}

/// Add unique IDs to <h1>–<h3> tags that don't already have an id attribute.
/// Generates IDs from the heading text content, supports Chinese characters.
fn add_heading_ids(html: &str) -> String {
    use regex::Regex;
    
    let re = Regex::new(r"<h([1-3])(\s[^>]*)?>([^<]*)</h[1-3]>").unwrap();
    let mut used_ids = HashSet::new();
    
    re.replace_all(html, |caps: &regex::Captures| {
        let level = caps.get(1).unwrap().as_str();
        let attrs = caps.get(2).map_or("", |m| m.as_str());
        let text = caps.get(3).unwrap().as_str();
        
        // Skip if already has an id
        if attrs.contains("id=") {
            return caps.get(0).unwrap().as_str().to_string();
        }
        
        let id = make_heading_id(text, &mut used_ids);
        format!("<h{level} id=\"{id}\"{attrs}>{text}</h{level}>", level = level, id = id, attrs = attrs, text = text)
    }).to_string()
}

/// Generate a URL-safe ID from heading text (supports Chinese)
fn make_heading_id(text: &str, used: &mut HashSet<String>) -> String {
    let base = text
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || ('\u{4e00}'..='\u{9fff}').contains(&c) {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    
    let base = if base.is_empty() { "heading".to_string() } else { base };
    
    if used.insert(base.clone()) {
        return base;
    }
    let mut i = 2;
    loop {
        let candidate = format!("{}-{}", base, i);
        if used.insert(candidate.clone()) {
            return candidate;
        }
        i += 1;
    }
}

#[derive(Debug, Clone)]
struct SeoPage {
    title: String,
    description: String,
    canonical_url: String,
    heading: String,
    intro: String,
    content_html: String,
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn render_spa_seo_shell(shell: &str, page: &SeoPage) -> Result<String, &'static str> {
    let title_start = shell.find("<title>").ok_or("SPA shell is missing <title>")?;
    let title_end = shell[title_start..]
        .find("</title>")
        .map(|offset| title_start + offset + "</title>".len())
        .ok_or("SPA shell is missing </title>")?;

    let title = escape_html(&page.title);
    let description = escape_html(&page.description);
    let canonical_url = escape_html(&page.canonical_url);
    let metadata = format!(
        "<title>{title}</title>\n    <meta name=\"description\" content=\"{description}\">\n    <link rel=\"canonical\" href=\"{canonical_url}\">\n    <meta property=\"og:type\" content=\"website\">\n    <meta property=\"og:title\" content=\"{title}\">\n    <meta property=\"og:description\" content=\"{description}\">\n    <meta property=\"og:url\" content=\"{canonical_url}\">"
    );

    let mut html = String::with_capacity(shell.len() + page.content_html.len() + 512);
    html.push_str(&shell[..title_start]);
    html.push_str(&metadata);
    html.push_str(&shell[title_end..]);

    let app_marker = "<div id=\"app\"></div>";
    if !html.contains(app_marker) {
        return Err("SPA shell is missing #app mount point");
    }

    let prerendered = format!(
        "<div id=\"app\"><main class=\"seo-prerender\"><header><h1>{}</h1><p>{}</p></header>{}</main></div>",
        escape_html(&page.heading),
        escape_html(&page.intro),
        page.content_html,
    );

    Ok(html.replacen(app_marker, &prerendered, 1))
}

fn render_post_list(posts: &[posts::Model]) -> String {
    if posts.is_empty() {
        return "<p>暂无已发布文章。</p>".to_string();
    }

    let items = posts
        .iter()
        .map(|post| {
            let published_at = post
                .published_at
                .unwrap_or(post.created_at)
                .format("%Y-%m-%d")
                .to_string();
            let summary = post
                .summary
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| format!("<p>{}</p>", escape_html(value)))
                .unwrap_or_default();
            format!(
                "<article><h2><a href=\"/post/{}\">{}</a></h2><time datetime=\"{}\">{}</time>{}</article>",
                escape_html(&post.slug),
                escape_html(&post.title),
                published_at,
                published_at,
                summary,
            )
        })
        .collect::<String>();

    format!("<section aria-label=\"文章列表\">{items}</section>")
}

fn render_link_list(items: &[(String, String, Option<String>)], label: &str) -> String {
    if items.is_empty() {
        return format!("<p>暂无{}。</p>", escape_html(label));
    }

    let links = items
        .iter()
        .map(|(href, name, description)| {
            let description = description
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| format!("<p>{}</p>", escape_html(value)))
                .unwrap_or_default();
            format!(
                "<li><a href=\"{}\">{}</a>{}</li>",
                escape_html(href),
                escape_html(name),
                description,
            )
        })
        .collect::<String>();

    format!(
        "<nav aria-label=\"{}\"><ul>{}</ul></nav>",
        escape_html(label),
        links,
    )
}

#[derive(Default)]
struct SeoPostFilter<'a> {
    category_id: Option<i32>,
    tag_id: Option<i32>,
    author_id: Option<i32>,
    article_type: Option<&'a str>,
    article_status: Option<&'a str>,
    pinned: Option<bool>,
}

async fn load_seo_posts(
    state: &AppState,
    limit: u64,
    filter: SeoPostFilter<'_>,
) -> Result<Vec<posts::Model>, AppError> {
    let hidden_ids = crate::controllers::categories::get_hidden_category_ids(&state.db).await?;
    let exclude_ids = (!hidden_ids.is_empty()).then_some(hidden_ids.as_slice());
    let (items, _) = crate::services::posts::list_posts(
        &state.db,
        1,
        limit,
        Some("published"),
        filter.category_id,
        None,
        filter.tag_id,
        None,
        filter.author_id,
        exclude_ids,
        filter.article_type,
        None,
        filter.article_status,
        None,
        filter.pinned,
        filter.category_id.map(|_| true),
        None,
    )
    .await?;
    Ok(items)
}

pub async fn aggregate_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Html<String>, AppError> {
    let raw_site_title = get_setting(&state.db, "site_title")
        .await
        .unwrap_or_else(|| "Mark-Share-X_用AI学AI".to_string());
    let site_title = display_site_title(&raw_site_title);
    let site_description = get_setting(&state.db, "site_description")
        .await
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("浏览 {} 的最新技术文章与知识内容。", site_title));
    let base_url = derive_base_url(&headers, &state.config.server.host, state.config.server.port);
    let canonical_url = format!("{}{}", base_url, uri.path());
    let segments: Vec<&str> = uri.path().trim_matches('/').split('/').collect();

    let (heading, description, content_html) = match uri.path() {
        "/" => {
            let posts = load_seo_posts(&state, 12, SeoPostFilter::default()).await?;
            (site_title.clone(), site_description.clone(), render_post_list(&posts))
        }
        "/knowledge-base" => {
            let posts = load_seo_posts(&state, 50, SeoPostFilter::default()).await?;
            (
                "知识库".to_string(),
                format!("浏览 {} 发布的技术文章、实践教程与知识内容。", site_title),
                render_post_list(&posts),
            )
        }
        "/categories" => {
            let models = categories::Entity::find()
                .filter(categories::Column::IsVisible.eq(true))
                .filter(categories::Column::DeletedAt.is_null())
                .order_by_asc(categories::Column::SortOrder)
                .order_by_asc(categories::Column::Name)
                .all(&state.db)
                .await?;
            let items = models
                .into_iter()
                .map(|item| {
                    (
                        format!("/category/{}", item.slug),
                        item.name,
                        item.description,
                    )
                })
                .collect::<Vec<_>>();
            (
                "文章分类".to_string(),
                format!("按分类浏览 {} 的技术文章与知识内容。", site_title),
                render_link_list(&items, "文章分类"),
            )
        }
        "/tags" => {
            let models = tags::Entity::find()
                .filter(tags::Column::DeletedAt.is_null())
                .order_by_asc(tags::Column::Name)
                .all(&state.db)
                .await?;
            let items = models
                .into_iter()
                .map(|item| (format!("/tag/{}", item.slug), item.name, None))
                .collect::<Vec<_>>();
            (
                "文章标签".to_string(),
                format!("通过标签发现 {} 的相关技术文章。", site_title),
                render_link_list(&items, "文章标签"),
            )
        }
        "/authors" => {
            let rows = state
                .db
                .query_all(Statement::from_string(
                    state.db.get_database_backend(),
                    "SELECT u.id, COALESCE(NULLIF(u.display_name, ''), u.username), u.bio, COUNT(p.id) FROM users u JOIN posts p ON p.user_id = u.id WHERE p.status = 'published' AND p.deleted_at IS NULL AND u.deleted_at IS NULL GROUP BY u.id, u.display_name, u.username, u.bio ORDER BY COUNT(p.id) DESC".to_string(),
                ))
                .await?;
            let items = rows
                .into_iter()
                .filter_map(|row| {
                    let id = row.try_get_by_index::<i32>(0).ok()?;
                    let name = row.try_get_by_index::<String>(1).ok()?;
                    let bio = row.try_get_by_index::<Option<String>>(2).ok().flatten();
                    Some((format!("/author/{id}"), name, bio))
                })
                .collect::<Vec<_>>();
            (
                "作者".to_string(),
                format!("浏览 {} 的作者及其发布的文章。", site_title),
                render_link_list(&items, "作者"),
            )
        }
        "/types" => {
            let models = article_types::Entity::find()
                .filter(article_types::Column::IsActive.eq(true))
                .order_by_asc(article_types::Column::SortOrder)
                .all(&state.db)
                .await?;
            let items = models
                .into_iter()
                .filter(|item| !item.code.trim().is_empty() && !item.display_name.trim().is_empty())
                .map(|item| (format!("/type/{}", item.code), item.display_name, None))
                .collect::<Vec<_>>();
            (
                "文章类型".to_string(),
                "按教程、实践、参考资料等类型浏览文章。".to_string(),
                render_link_list(&items, "文章类型"),
            )
        }
        "/statuses" => {
            let models = article_statuses::Entity::find()
                .filter(article_statuses::Column::IsActive.eq(true))
                .order_by_asc(article_statuses::Column::SortOrder)
                .all(&state.db)
                .await?;
            let items = models
                .into_iter()
                .filter(|item| !item.code.trim().is_empty() && !item.display_name.trim().is_empty())
                .map(|item| (format!("/status/{}", item.code), item.display_name, None))
                .collect::<Vec<_>>();
            (
                "文章状态".to_string(),
                "按文章维护状态浏览当前有效内容。".to_string(),
                render_link_list(&items, "文章状态"),
            )
        }
        "/pinned" => {
            let posts = load_seo_posts(
                &state,
                50,
                SeoPostFilter {
                    pinned: Some(true),
                    ..Default::default()
                },
            )
            .await?;
            (
                "推荐文章".to_string(),
                format!("浏览 {} 精选和推荐的技术文章。", site_title),
                render_post_list(&posts),
            )
        }
        _ => match segments.as_slice() {
            ["category", slug] if !slug.is_empty() => {
                let category = categories::Entity::find()
                    .filter(categories::Column::Slug.eq(*slug))
                    .filter(categories::Column::IsVisible.eq(true))
                    .filter(categories::Column::DeletedAt.is_null())
                    .one(&state.db)
                    .await?
                    .ok_or_else(|| AppError::NotFound("分类不存在".to_string()))?;
                let posts = load_seo_posts(
                    &state,
                    50,
                    SeoPostFilter {
                        category_id: Some(category.id),
                        ..Default::default()
                    },
                )
                .await?;
                let description = category
                    .description
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| format!("浏览“{}”分类下的技术文章。", category.name));
                (category.name, description, render_post_list(&posts))
            }
            ["tag", slug] if !slug.is_empty() => {
                let tag = tags::Entity::find()
                    .filter(tags::Column::Slug.eq(*slug))
                    .filter(tags::Column::DeletedAt.is_null())
                    .one(&state.db)
                    .await?
                    .ok_or_else(|| AppError::NotFound("标签不存在".to_string()))?;
                let posts = load_seo_posts(
                    &state,
                    50,
                    SeoPostFilter {
                        tag_id: Some(tag.id),
                        ..Default::default()
                    },
                )
                .await?;
                let description = format!("浏览带有“{}”标签的技术文章。", tag.name);
                (tag.name, description, render_post_list(&posts))
            }
            ["author", id] => {
                let id = id
                    .parse::<i32>()
                    .map_err(|_| AppError::NotFound("作者不存在".to_string()))?;
                let author = users::Entity::find_by_id(id)
                    .filter(users::Column::DeletedAt.is_null())
                    .one(&state.db)
                    .await?
                    .ok_or_else(|| AppError::NotFound("作者不存在".to_string()))?;
                let posts = load_seo_posts(
                    &state,
                    50,
                    SeoPostFilter {
                        author_id: Some(id),
                        ..Default::default()
                    },
                )
                .await?;
                let name = author.display_name.unwrap_or(author.username);
                let description = author
                    .bio
                    .map(|value| value.trim().to_string())
                    .filter(|value| value.chars().count() >= 10)
                    .unwrap_or_else(|| format!("浏览 {} 发布的技术文章。", name));
                (name, description, render_post_list(&posts))
            }
            ["type", code] if !code.is_empty() => {
                let article_type = article_types::Entity::find()
                    .filter(article_types::Column::Code.eq(*code))
                    .filter(article_types::Column::IsActive.eq(true))
                    .one(&state.db)
                    .await?
                    .ok_or_else(|| AppError::NotFound("文章类型不存在".to_string()))?;
                if article_type.display_name.trim().is_empty() {
                    return Err(AppError::NotFound("文章类型不存在".to_string()));
                }
                let posts = load_seo_posts(
                    &state,
                    50,
                    SeoPostFilter {
                        article_type: Some(&article_type.code),
                        ..Default::default()
                    },
                )
                .await?;
                let description = format!("浏览“{}”类型的技术文章。", article_type.display_name);
                (article_type.display_name, description, render_post_list(&posts))
            }
            ["status", code] if !code.is_empty() => {
                let article_status = article_statuses::Entity::find()
                    .filter(article_statuses::Column::Code.eq(*code))
                    .filter(article_statuses::Column::IsActive.eq(true))
                    .one(&state.db)
                    .await?
                    .ok_or_else(|| AppError::NotFound("文章状态不存在".to_string()))?;
                if article_status.display_name.trim().is_empty() {
                    return Err(AppError::NotFound("文章状态不存在".to_string()));
                }
                let posts = load_seo_posts(
                    &state,
                    50,
                    SeoPostFilter {
                        article_status: Some(&article_status.code),
                        ..Default::default()
                    },
                )
                .await?;
                let description = format!("浏览状态为“{}”的技术文章。", article_status.display_name);
                (article_status.display_name, description, render_post_list(&posts))
            }
            _ => return Err(AppError::NotFound("页面不存在".to_string())),
        },
    };

    let page_title = if uri.path() == "/" {
        site_title.clone()
    } else {
        format!("{} - {}", heading, site_title)
    };
    let page = SeoPage {
        title: page_title,
        description: description.clone(),
        canonical_url,
        heading,
        intro: description,
        content_html,
    };
    let shell = tokio::fs::read_to_string("static/frontend/index.html").await?;
    let html = render_spa_seo_shell(&shell, &page)
        .map_err(|message| AppError::Internal(anyhow::anyhow!(message)))?;
    Ok(Html(html))
}

fn extract_meta_text(content_html: &str, title: &str) -> String {
    use regex::Regex;
    use std::sync::LazyLock;

    static NON_CONTENT_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?is)<(?:script|style|pre)[^>]*>.*?</(?:script|style|pre)>").unwrap()
    });
    static BLOCK_END_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)</(?:p|h[1-6]|li|blockquote|div|section|article)>|<br\s*/?>").unwrap()
    });
    static TAG_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?s)<[^>]+>").unwrap());
    static ENTITY_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"&(?:amp|lt|gt|quot|apos|nbsp|#\d+|#x[0-9a-fA-F]+);").unwrap()
    });

    let without_noise = NON_CONTENT_RE.replace_all(content_html, " ");
    let with_boundaries = BLOCK_END_RE.replace_all(&without_noise, " ");
    let without_tags = TAG_RE.replace_all(&with_boundaries, "");
    let decoded = ENTITY_RE.replace_all(&without_tags, |caps: &regex::Captures| {
        match &caps[0] {
            "&amp;" => "&".to_string(),
            "&lt;" => "<".to_string(),
            "&gt;" => ">".to_string(),
            "&quot;" => "\"".to_string(),
            "&apos;" => "'".to_string(),
            "&nbsp;" => " ".to_string(),
            entity if entity.starts_with("&#x") => u32::from_str_radix(&entity[3..entity.len() - 1], 16)
                .ok()
                .and_then(char::from_u32)
                .map(|c| c.to_string())
                .unwrap_or_default(),
            entity if entity.starts_with("&#") => entity[2..entity.len() - 1]
                .parse::<u32>()
                .ok()
                .and_then(char::from_u32)
                .map(|c| c.to_string())
                .unwrap_or_default(),
            _ => String::new(),
        }
    });

    let mut text = decoded.split_whitespace().collect::<Vec<_>>().join(" ");
    let title = title.trim();
    if !title.is_empty() {
        if let Some(rest) = text.strip_prefix(title) {
            text = rest
                .trim_start_matches(|c: char| c.is_whitespace() || ":：-—|".contains(c))
                .to_string();
        }
    }

    text.chars().take(160).collect::<String>().trim().to_string()
}

fn build_meta_description(
    summary: Option<&str>,
    content_html: &str,
    site_description: &str,
    title: &str,
) -> String {
    if let Some(summary) = summary.map(str::trim).filter(|value| !value.is_empty()) {
        return summary.chars().take(160).collect();
    }

    let extracted = extract_meta_text(content_html, title);
    if !extracted.is_empty() {
        return extracted;
    }

    let site_description = site_description.trim();
    if !site_description.is_empty() {
        return site_description.chars().take(160).collect();
    }

    title.trim().chars().take(160).collect()
}

// ── SEO endpoints ──

/// Return true for routes declared in the Vue router that are served by the SPA shell.
/// Keep this list in sync with frontend/src/router/index.ts.
fn is_known_spa_route(path: &str) -> bool {
    const EXACT_ROUTES: &[&str] = &[
        "/",
        "/knowledge-base",
        "/tags",
        "/types",
        "/statuses",
        "/categories",
        "/authors",
        "/search",
        "/login",
        "/register",
        "/apply",
        "/changelog",
        "/pinned",
        "/guestbook",
        "/admin",
        "/admin/setup",
        "/admin/dashboard",
        "/admin/posts",
        "/admin/posts/new",
        "/admin/categories",
        "/admin/tags",
        "/admin/files",
        "/admin/analytics/views",
        "/admin/analytics/comments",
        "/admin/likes",
        "/admin/import",
        "/admin/users",
        "/admin/settings",
        "/admin/guestbook",
        "/admin/news",
        "/admin/ai",
    ];

    if EXACT_ROUTES.contains(&path) {
        return true;
    }

    let segments: Vec<&str> = path.trim_matches('/').split('/').collect();
    matches!(
        segments.as_slice(),
        ["tag" | "type" | "status" | "category" | "author", value] if !value.is_empty()
    ) || matches!(
        segments.as_slice(),
        ["admin", "posts", value] if !value.is_empty()
    )
}

/// Fallback for routes that are not handled by the API, SEO pages, or static files.
/// Known Vue routes return 200. Unknown routes return the same SPA shell with a real
/// 404 status so Vue can render its NotFound view without creating soft 404s.
pub async fn spa_fallback(uri: Uri) -> Response {
    let status = if is_known_spa_route(uri.path()) {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    };

    match tokio::fs::read("static/frontend/index.html").await {
        Ok(body) => (
            status,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            body,
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "前端页面文件不存在",
        )
            .into_response(),
    }
}

/// GET /robots.txt — guide search engine crawlers
pub async fn robots_txt(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, String), AppError> {
    let base_url = derive_base_url(&headers, &state.config.server.host, state.config.server.port);
    let content = format!(
        "User-agent: *\nAllow: /\nSitemap: {}/sitemap.xml\n",
        base_url
    );
    let mut resp_headers = HeaderMap::new();
    resp_headers.insert("Content-Type", "text/plain".parse().unwrap());
    Ok((resp_headers, content))
}

/// GET /sitemap.xml — dynamic sitemap for search engines
pub async fn sitemap_xml(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, String), AppError> {
    use sea_orm::QueryOrder;

    let base_url = derive_base_url(&headers, &state.config.server.host, state.config.server.port);

    // Query published posts
    let posts = posts::Entity::find()
        .filter(posts::Column::Status.eq("published"))
        .order_by_desc(posts::Column::PublishedAt)
        .all(&state.db)
        .await
        .unwrap_or_default();

    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
"#,
    );

    // Homepage
    xml.push_str(&format!(
        "  <url><loc>{}/</loc><priority>1.0</priority></url>\n",
        base_url
    ));

    // Posts
    for post in &posts {
        let lastmod = post.published_at.unwrap_or(post.created_at)
            .format("%Y-%m-%d");
        xml.push_str(&format!(
            "  <url><loc>{}/post/{}</loc><lastmod>{}</lastmod><priority>0.8</priority></url>\n",
            base_url, post.slug, lastmod
        ));
    }

    xml.push_str("</urlset>\n");

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert("Content-Type", "application/xml".parse().unwrap());
    Ok((resp_headers, xml))
}

/// Helper: derive base URL from request Host header
fn derive_base_url(headers: &HeaderMap, config_host: &str, config_port: u16) -> String {
    let scheme = if headers.get("x-forwarded-proto").and_then(|v| v.to_str().ok()) == Some("https") {
        "https"
    } else {
        "http"
    };
    let raw_host = headers.get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or(config_host)
        .to_string();
    let host_only = raw_host.split(':').next().unwrap_or(&raw_host);
    if host_only == "localhost" || host_only.starts_with("127.") || host_only.starts_with("192.168.") {
        format!("{}://{}:{}", scheme, host_only, config_port)
    } else {
        format!("{}://{}", scheme, raw_host)
    }
}

/// Embedded default social sharing image for posts without a real cover.
pub async fn default_og_image() -> Response {
    (
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        DEFAULT_OG_IMAGE,
    )
        .into_response()
}

// ── Favicon: SVG 优先，缺失时 fallback 到 PNG；只要有一个文件存在就正常返回 ──
pub async fn favicon_svg() -> Response {
    match tokio::fs::read("static/frontend/favicon.svg").await {
        Ok(data) => Response::builder()
            .header(header::CONTENT_TYPE, "image/svg+xml")
            .body(axum::body::Body::from(data))
            .unwrap(),
        Err(_) => match tokio::fs::read("static/frontend/favicon.png").await {
            Ok(data) => Response::builder()
                .header(header::CONTENT_TYPE, "image/png")
                .body(axum::body::Body::from(data))
                .unwrap(),
            Err(_) => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(axum::body::Body::empty())
                .unwrap(),
        },
    }
}

pub async fn favicon_png() -> Response {
    match tokio::fs::read("static/frontend/favicon.png").await {
        Ok(data) => Response::builder()
            .header(header::CONTENT_TYPE, "image/png")
            .body(axum::body::Body::from(data))
            .unwrap(),
        Err(_) => match tokio::fs::read("static/frontend/favicon.svg").await {
            Ok(data) => Response::builder()
                .header(header::CONTENT_TYPE, "image/svg+xml")
                .body(axum::body::Body::from(data))
                .unwrap(),
            Err(_) => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(axum::body::Body::empty())
                .unwrap(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_meta_description, build_social_image_url, display_site_title,
        normalize_article_headings, render_spa_seo_shell, SeoPage,
    };

    #[test]
    fn site_title_display_removes_every_hyphen() {
        assert_eq!(display_site_title("Mark-Share-X"), "MarkShareX");
        assert_eq!(display_site_title("Mark---Share--X"), "MarkShareX");
        assert_eq!(display_site_title("-Mark-Share-X-"), "MarkShareX");
        assert_eq!(display_site_title("----"), "Mark-Share-X_用AI学AI");
    }

    #[test]
    fn social_image_prefers_cover_and_falls_back_to_embedded_default() {
        assert_eq!(
            build_social_image_url(Some("https://cdn.example.com/cover.jpg"), "https://www.xlevon.cn"),
            "https://cdn.example.com/cover.jpg"
        );
        assert_eq!(
            build_social_image_url(Some("/uploads/cover.jpg"), "https://www.xlevon.cn"),
            "https://www.xlevon.cn/uploads/cover.jpg"
        );
        assert_eq!(
            build_social_image_url(None, "https://www.xlevon.cn"),
            "https://www.xlevon.cn/default-og.png"
        );
    }

    #[test]
    fn article_headings_remove_duplicate_title_and_demote_other_h1s() {
        let html = normalize_article_headings(
            "  <h1 class=\"title\">Rust &amp; 错误处理</h1><p>导语</p><h1 id=\"part\">进阶</h1><h2>细节</h2>",
            "Rust & 错误处理",
        );

        assert!(!html.contains("<h1"));
        assert!(!html.contains("Rust &amp; 错误处理</h1>"));
        assert!(html.contains("<h2 id=\"part\">进阶</h2>"));
        assert!(html.contains("<h2>细节</h2>"));
    }

    #[test]
    fn spa_shell_contains_unique_metadata_and_crawlable_content() {
        let shell = r#"<html><head><title>默认标题</title><script src="/assets/app.js"></script></head><body><div id="app"></div></body></html>"#;
        let page = SeoPage {
            title: "知识库 - MarkShareX".to_string(),
            description: "浏览技术文章".to_string(),
            canonical_url: "https://www.xlevon.cn/knowledge-base".to_string(),
            heading: "知识库".to_string(),
            intro: "浏览技术文章".to_string(),
            content_html: "<ul><li><a href=\"/post/rust\">Rust 教程</a></li></ul>".to_string(),
        };

        let html = render_spa_seo_shell(shell, &page).expect("SPA shell should render");

        assert!(html.contains("<title>知识库 - MarkShareX</title>"));
        assert!(html.contains("<meta name=\"description\" content=\"浏览技术文章\">"));
        assert!(html.contains("<link rel=\"canonical\" href=\"https://www.xlevon.cn/knowledge-base\">"));
        assert!(html.contains("<h1>知识库</h1>"));
        assert!(html.contains("<a href=\"/post/rust\">Rust 教程</a>"));
        assert!(html.contains("<script src=\"/assets/app.js\"></script>"));
    }

    #[test]
    fn ssr_article_template_applies_guest_copy_setting_to_direct_visits() {
        let template = include_str!("../../templates/default/post.html");

        assert!(template.contains("data-guest-copy-enabled=\"{{ guest_copy_enabled }}\""));
        assert!(template.contains("addEventListener('contextmenu'"));
        assert!(template.contains("addEventListener('copy'"));
        assert!(template.contains("marksharex_token"));
        assert!(!template.contains("addEventListener('keydown'"));
    }

    #[test]
    fn meta_description_prefers_trimmed_manual_summary() {
        let result = build_meta_description(
            Some("  人工编写的文章摘要  "),
            "<p>正文内容</p>",
            "站点描述",
            "文章标题",
        );

        assert_eq!(result, "人工编写的文章摘要");
    }

    #[test]
    fn blank_summary_is_generated_from_readable_article_text() {
        let result = build_meta_description(
            Some("  \n "),
            "<h1>文章标题</h1><p>这是<strong>正文重点</strong>。</p><pre><code>cargo build --release</code></pre><p>第二段 &amp; 内容。</p>",
            "站点描述",
            "文章标题",
        );

        assert_eq!(result, "这是正文重点。 第二段 & 内容。");
    }
}
