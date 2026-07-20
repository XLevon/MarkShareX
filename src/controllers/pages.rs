use crate::models::entity::{
    article_statuses, article_types, categories, post_tags, posts, tags, users,
};
use crate::utils::{AppError, AppState};
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
};
use sea_orm::*;
use serde_json::json;
use std::collections::HashSet;
use std::error::Error as _;

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
        "MarkShareX_用AI学AI".to_string()
    } else {
        title.to_string()
    }
}

const META_TITLE_MAX_WIDTH: usize = 60;
const META_DESCRIPTION_MAX_WIDTH: usize = 160;

fn seo_char_width(value: char) -> usize {
    match value as u32 {
        0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF => 0,
        0x1100..=0x115F
        | 0x2329..=0x232A
        | 0x2E80..=0xA4CF
        | 0xAC00..=0xD7A3
        | 0xF900..=0xFAFF
        | 0xFE10..=0xFE19
        | 0xFE30..=0xFE6F
        | 0xFF00..=0xFF60
        | 0xFFE0..=0xFFE6
        | 0x1F300..=0x1FAFF
        | 0x20000..=0x3FFFD => 2,
        _ => 1,
    }
}

fn seo_display_width(value: &str) -> usize {
    value.chars().map(seo_char_width).sum()
}

fn compact_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_by_display_width(value: &str, max_width: usize, prefer_natural_break: bool) -> String {
    let compact = compact_whitespace(value);
    if seo_display_width(&compact) <= max_width {
        return compact;
    }

    let ellipsis_width = seo_char_width('…');
    let content_limit = max_width.saturating_sub(ellipsis_width);
    let mut width = 0;
    let mut hard_end = 0;
    let mut natural_break = None;

    for (index, character) in compact.char_indices() {
        let character_width = seo_char_width(character);
        if width + character_width > content_limit {
            break;
        }

        width += character_width;
        hard_end = index + character.len_utf8();
        if prefer_natural_break
            && matches!(
                character,
                '。' | '！' | '？' | '；' | '，' | '、' | ',' | ';'
            )
            && width * 5 >= max_width * 3
        {
            natural_break = Some((hard_end, character));
        }
    }

    let (end, ending) = natural_break
        .map(|(end, character)| (end, Some(character)))
        .unwrap_or((hard_end, None));
    let truncated = compact[..end].trim_end();

    if matches!(ending, Some('。' | '！' | '？')) {
        truncated.to_string()
    } else {
        format!("{}…", truncated)
    }
}

fn build_meta_title(post_title: &str, site_title: &str) -> String {
    let post_title = compact_whitespace(post_title);
    let site_title = compact_whitespace(site_title);

    if post_title.is_empty() {
        return truncate_by_display_width(&site_title, META_TITLE_MAX_WIDTH, false);
    }
    if site_title.is_empty() {
        return truncate_by_display_width(&post_title, META_TITLE_MAX_WIDTH, false);
    }

    let combined = format!("{} - {}", post_title, site_title);
    if seo_display_width(&combined) <= META_TITLE_MAX_WIDTH {
        combined
    } else {
        truncate_by_display_width(&post_title, META_TITLE_MAX_WIDTH, false)
    }
}

fn compact_site_meta_description(value: &str, fallback: &str) -> String {
    let first_paragraph = value
        .split("\n\n")
        .map(str::trim)
        .find(|paragraph| !paragraph.is_empty())
        .unwrap_or(fallback);
    let compact = first_paragraph
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let compact = compact.trim();
    if compact.is_empty() {
        fallback.chars().take(120).collect()
    } else {
        compact.chars().take(120).collect()
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
    super::posts::authorize_post_read(None, &post)?;

    let post_id = post.id;
    // Real view count from read_logs
    let view_count: i64 = state
        .db
        .query_one(sea_orm::Statement::from_string(
            state.db.get_database_backend(),
            format!("SELECT COUNT(*) FROM read_logs WHERE post_id = {}", post_id),
        ))
        .await
        .ok()
        .flatten()
        .and_then(|r| r.try_get_by_index::<i64>(0).ok())
        .unwrap_or(0);

    let category = if let Some(category_id) = post.category_id {
        categories::Entity::find_by_id(category_id)
            .one(&state.db)
            .await?
    } else {
        None
    };
    let category_name = category.as_ref().map(|value| value.name.clone());
    let category_slug = category.as_ref().map(|value| value.slug.clone());
    let tags = crate::services::posts::get_post_tags(&state.db, post.id).await?;

    let raw_site_title = get_setting(&state.db, "site_title")
        .await
        .unwrap_or_else(|| "Mark-Share-X_用AI学AI".to_string());
    let site_title = display_site_title(&raw_site_title);
    let site_description = get_setting(&state.db, "site_description")
        .await
        .unwrap_or_default();
    let configured_site_logo = get_setting(&state.db, "site_logo")
        .await
        .filter(|value| !value.trim().is_empty());
    let guest_copy_enabled = get_setting(&state.db, "guest_copy_enabled")
        .await
        .as_deref()
        != Some("false");
    let author = users::Entity::find_by_id(post.user_id)
        .one(&state.db)
        .await?;
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
    )
    .await;

    if cover_image.is_none() {
        if let Some(category) = category.as_ref() {
            cover_image = crate::controllers::network_resources::resolve_cover_url(
                &state.db,
                category.network_resource_id,
                category.image_url.as_deref(),
                category.image_filename.as_deref(),
            )
            .await;
        }
    }

    // Derive base_url from request Host header (for OG meta tags)
    let scheme = if headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        == Some("https")
    {
        "https"
    } else {
        "http"
    };
    let raw_host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or(&state.config.server.host)
        .to_string();
    // Strip port from host header if present (Host: localhost:5023 → localhost)
    let host_only = raw_host.split(':').next().unwrap_or(&raw_host);
    let base_url = if host_only == "localhost"
        || host_only.starts_with("127.")
        || host_only.starts_with("192.168.")
    {
        format!("{}://{}:{}", scheme, host_only, state.config.server.port)
    } else {
        format!("{}://{}", scheme, raw_host) // keep port in Host header for proxied requests
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
    let mut resolved_html =
        crate::controllers::network_resources::resolve_nr_in_content(&state.db, &raw_html).await;

    // Resolve ./uploads/ → /uploads/ (root-relative) for images
    resolved_html = resolved_html.replace("./uploads/", "/uploads/");

    // Keep the article title as the page's only H1 without changing stored content.
    resolved_html = normalize_article_headings(&resolved_html, &post.title);

    // Add heading IDs for anchor link support (comrak doesn't auto-generate them)
    resolved_html = add_heading_ids(&resolved_html);

    let article_url = format!("{}/post/{}", base_url, post.slug);
    let meta_title = build_meta_title(&post.title, &site_title);
    let meta_description = build_meta_description(
        post.summary.as_deref(),
        &resolved_html,
        &site_description,
        &post.title,
    );

    // Query adjacent posts for prev/next navigation
    let (adjacent_prev, adjacent_next) =
        crate::services::posts::get_adjacent_posts(&state.db, post_id)
            .await
            .unwrap_or_default();

    // Query related posts: same category, excluding current, limit 5
    use crate::models::entity::posts as posts_entity;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
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
    ctx.insert("meta_title", &meta_title);
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
        "category_slug": category_slug,
        "tags": tags.iter().map(|tag| json!({
            "name": tag.name,
            "slug": tag.slug,
        })).collect::<Vec<_>>(),
    }));
    ctx.insert(
        "adjacent_prev",
        &adjacent_prev.map(|(_, title, slug)| json!({"title": title, "slug": slug})),
    );
    ctx.insert(
        "adjacent_next",
        &adjacent_next.map(|(_, title, slug)| json!({"title": title, "slug": slug})),
    );
    ctx.insert(
        "related_posts",
        &related_posts
            .iter()
            .map(|(title, slug)| json!({"title": title, "slug": slug}))
            .collect::<Vec<_>>(),
    );
    let tag_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
    ctx.insert("tag_names", &tag_names);
    let tag_links = tags
        .iter()
        .map(|tag| {
            json!({
                "name": tag.name,
                "slug": tag.slug,
            })
        })
        .collect::<Vec<_>>();
    ctx.insert("tag_links", &tag_links);

    let html = state.tera.render("post.html", &ctx).map_err(|e| {
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
    static H1_TAG_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)<(/?)h1(\s[^>]*)?>").expect("valid H1 tag regex"));

    let mut normalized = html.to_string();
    if let Some(captures) = LEADING_H1_RE.captures(&normalized) {
        let heading_html = captures.get(1).map_or("", |value| value.as_str());
        let heading_text = decode_heading_entities(&HTML_TAG_RE.replace_all(heading_html, ""));
        let heading_text = heading_text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let title_text = article_title
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

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
        format!(
            "<h{level} id=\"{id}\"{attrs}>{text}</h{level}>",
            level = level,
            id = id,
            attrs = attrs,
            text = text
        )
    })
    .to_string()
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

    let base = if base.is_empty() {
        "heading".to_string()
    } else {
        base
    };

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
    site_title: String,
    social_image: String,
    schema_type: &'static str,
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
    let title_start = shell
        .find("<title>")
        .ok_or("SPA shell is missing <title>")?;
    let title_end = shell[title_start..]
        .find("</title>")
        .map(|offset| title_start + offset + "</title>".len())
        .ok_or("SPA shell is missing </title>")?;

    let title = escape_html(&page.title);
    let description = escape_html(&page.description);
    let canonical_url = escape_html(&page.canonical_url);
    let social_image = escape_html(&page.social_image);
    let metadata = format!(
        "<title>{title}</title>\n    <meta name=\"description\" content=\"{description}\">\n    <link rel=\"canonical\" href=\"{canonical_url}\">\n    <meta property=\"og:type\" content=\"website\">\n    <meta property=\"og:title\" content=\"{title}\">\n    <meta property=\"og:description\" content=\"{description}\">\n    <meta property=\"og:url\" content=\"{canonical_url}\">\n    <meta property=\"og:image\" content=\"{social_image}\">\n    <meta name=\"twitter:card\" content=\"summary_large_image\">\n    <meta name=\"twitter:title\" content=\"{title}\">\n    <meta name=\"twitter:description\" content=\"{description}\">\n    <meta name=\"twitter:image\" content=\"{social_image}\">"
    );

    let schema = if page.schema_type == "WebSite" {
        json!({
            "@context": "https://schema.org",
            "@type": "WebSite",
            "name": page.site_title,
            "url": page.canonical_url,
            "description": page.description,
            "image": page.social_image,
            "potentialAction": {
                "@type": "SearchAction",
                "target": format!("{}/search?q={{search_term_string}}", page.canonical_url.trim_end_matches('/')),
                "query-input": "required name=search_term_string"
            }
        })
    } else {
        json!({
            "@context": "https://schema.org",
            "@type": page.schema_type,
            "name": page.heading,
            "url": page.canonical_url,
            "description": page.description,
            "image": page.social_image,
            "isPartOf": {
                "@type": "WebSite",
                "name": page.site_title,
                "url": page.canonical_url.split('/').take(3).collect::<Vec<_>>().join("/")
            }
        })
    };
    let schema_json = serde_json::to_string(&schema)
        .map_err(|_| "failed to serialize SEO JSON-LD")?
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026");
    let metadata =
        format!("{metadata}\n    <script type=\"application/ld+json\">{schema_json}</script>");

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
    let site_meta_description = get_setting(&state.db, "site_meta_description")
        .await
        .filter(|value| !value.trim().is_empty())
        .map(|value| compact_site_meta_description(&value, &site_title))
        .unwrap_or_else(|| compact_site_meta_description(&site_description, &site_title));
    let base_url = derive_base_url(
        &headers,
        &state.config.server.host,
        state.config.server.port,
    );
    let canonical_url = format!("{}{}", base_url, uri.path());
    let segments: Vec<&str> = uri.path().trim_matches('/').split('/').collect();

    let (heading, description, content_html) = match uri.path() {
        "/" => {
            let posts = load_seo_posts(&state, 12, SeoPostFilter::default()).await?;
            (
                site_title.clone(),
                site_description.clone(),
                render_post_list(&posts),
            )
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
        "/changelog" => (
            "更新日志".to_string(),
            format!("查看 {} 的版本更新、功能改进与问题修复记录。", site_title),
            "<section aria-label=\"更新日志\"><p>版本更新、功能改进与问题修复记录。</p></section>"
                .to_string(),
        ),
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
                (
                    article_type.display_name,
                    description,
                    render_post_list(&posts),
                )
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
                let description =
                    format!("浏览状态为“{}”的技术文章。", article_status.display_name);
                (
                    article_status.display_name,
                    description,
                    render_post_list(&posts),
                )
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
        description: if uri.path() == "/" {
            site_meta_description
        } else {
            description.clone()
        },
        canonical_url,
        site_title,
        social_image: format!("{base_url}/default-og.png"),
        schema_type: if uri.path() == "/" {
            "WebSite"
        } else {
            "CollectionPage"
        },
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
    static TAG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<[^>]+>").unwrap());
    static ENTITY_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"&(?:amp|lt|gt|quot|apos|nbsp|#\d+|#x[0-9a-fA-F]+);").unwrap()
    });

    let without_noise = NON_CONTENT_RE.replace_all(content_html, " ");
    let with_boundaries = BLOCK_END_RE.replace_all(&without_noise, " ");
    let without_tags = TAG_RE.replace_all(&with_boundaries, "");
    let decoded = ENTITY_RE.replace_all(&without_tags, |caps: &regex::Captures| match &caps[0] {
        "&amp;" => "&".to_string(),
        "&lt;" => "<".to_string(),
        "&gt;" => ">".to_string(),
        "&quot;" => "\"".to_string(),
        "&apos;" => "'".to_string(),
        "&nbsp;" => " ".to_string(),
        entity if entity.starts_with("&#x") => {
            u32::from_str_radix(&entity[3..entity.len() - 1], 16)
                .ok()
                .and_then(char::from_u32)
                .map(|c| c.to_string())
                .unwrap_or_default()
        }
        entity if entity.starts_with("&#") => entity[2..entity.len() - 1]
            .parse::<u32>()
            .ok()
            .and_then(char::from_u32)
            .map(|c| c.to_string())
            .unwrap_or_default(),
        _ => String::new(),
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

    text.chars()
        .take(160)
        .collect::<String>()
        .trim()
        .to_string()
}

fn build_meta_description(
    summary: Option<&str>,
    content_html: &str,
    site_description: &str,
    title: &str,
) -> String {
    if let Some(summary) = summary.map(str::trim).filter(|value| !value.is_empty()) {
        return truncate_by_display_width(summary, META_DESCRIPTION_MAX_WIDTH, true);
    }

    let extracted = extract_meta_text(content_html, title);
    if !extracted.is_empty() {
        return truncate_by_display_width(&extracted, META_DESCRIPTION_MAX_WIDTH, true);
    }

    let title = title.trim();
    let site_description = compact_site_meta_description(site_description, "");
    if !title.is_empty() {
        let fallback = if site_description.is_empty() {
            title.to_string()
        } else {
            format!("{}：{}", title, site_description)
        };
        return truncate_by_display_width(&fallback, META_DESCRIPTION_MAX_WIDTH, true);
    }

    truncate_by_display_width(&site_description, META_DESCRIPTION_MAX_WIDTH, true)
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

fn spa_robots_directive(path: &str) -> Option<&'static str> {
    if path == "/search" {
        Some("noindex, follow")
    } else if matches!(path, "/login" | "/register" | "/apply" | "/guestbook")
        || path == "/admin"
        || path.starts_with("/admin/")
    {
        Some("noindex, nofollow")
    } else {
        None
    }
}

fn inject_robots_meta(shell: &str, directive: &str) -> String {
    let meta = format!("    <meta name=\"robots\" content=\"{}\">\n", directive);
    shell.replacen("</head>", &format!("{meta}</head>"), 1)
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

    match tokio::fs::read_to_string("static/frontend/index.html").await {
        Ok(shell) => {
            let directive = spa_robots_directive(uri.path());
            let body = directive
                .map(|value| inject_robots_meta(&shell, value))
                .unwrap_or(shell);
            let mut response = (
                status,
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                body,
            )
                .into_response();
            if let Some(value) = directive {
                response
                    .headers_mut()
                    .insert("X-Robots-Tag", value.parse().unwrap());
            }
            response
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "前端页面文件不存在").into_response(),
    }
}

/// GET /robots.txt — guide search engine crawlers
pub async fn robots_txt(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, String), AppError> {
    let base_url = derive_base_url(
        &headers,
        &state.config.server.host,
        state.config.server.port,
    );
    let content = format!(
        "User-agent: *\nAllow: /\nDisallow: /admin/\nDisallow: /api/\nSitemap: {}/sitemap.xml\n",
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

    let base_url = derive_base_url(
        &headers,
        &state.config.server.host,
        state.config.server.port,
    );

    // Published, non-deleted posts are the source of truth for every sitemap entry.
    let published_posts = posts::Entity::find()
        .filter(posts::Column::Status.eq("published"))
        .filter(posts::Column::DeletedAt.is_null())
        .order_by_desc(posts::Column::PublishedAt)
        .all(&state.db)
        .await
        .unwrap_or_default();

    let published_post_ids = published_posts
        .iter()
        .map(|post| post.id)
        .collect::<HashSet<_>>();
    let published_author_ids = published_posts
        .iter()
        .map(|post| post.user_id)
        .collect::<HashSet<_>>();
    let used_article_types = published_posts
        .iter()
        .map(|post| post.article_type.clone())
        .filter(|value| !value.trim().is_empty())
        .collect::<HashSet<_>>();
    let used_article_statuses = published_posts
        .iter()
        .map(|post| post.article_status.clone())
        .filter(|value| !value.trim().is_empty())
        .collect::<HashSet<_>>();

    let category_models = categories::Entity::find()
        .filter(categories::Column::IsVisible.eq(true))
        .filter(categories::Column::DeletedAt.is_null())
        .order_by_asc(categories::Column::SortOrder)
        .all(&state.db)
        .await
        .unwrap_or_default();
    let mut published_category_ids = published_posts
        .iter()
        .filter_map(|post| post.category_id)
        .collect::<HashSet<_>>();
    // Parent category pages aggregate child posts, so include every used ancestor too.
    loop {
        let mut changed = false;
        for category in &category_models {
            if published_category_ids.contains(&category.id) {
                if let Some(parent_id) = category.parent_id {
                    changed |= published_category_ids.insert(parent_id);
                }
            }
        }
        if !changed {
            break;
        }
    }

    let used_tag_ids = post_tags::Entity::find()
        .all(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|link| published_post_ids.contains(&link.post_id))
        .map(|link| link.tag_id)
        .collect::<HashSet<_>>();
    let tag_models = tags::Entity::find()
        .filter(tags::Column::DeletedAt.is_null())
        .order_by_asc(tags::Column::Name)
        .all(&state.db)
        .await
        .unwrap_or_default();
    let author_models = users::Entity::find()
        .filter(users::Column::IsActive.eq(true))
        .filter(users::Column::DeletedAt.is_null())
        .all(&state.db)
        .await
        .unwrap_or_default();
    let type_models = article_types::Entity::find()
        .filter(article_types::Column::IsActive.eq(true))
        .order_by_asc(article_types::Column::SortOrder)
        .all(&state.db)
        .await
        .unwrap_or_default();
    let status_models = article_statuses::Entity::find()
        .filter(article_statuses::Column::IsActive.eq(true))
        .order_by_asc(article_statuses::Column::SortOrder)
        .all(&state.db)
        .await
        .unwrap_or_default();

    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
"#,
    );

    let mut push_url = |path: &str, priority: &str, lastmod: Option<String>| {
        xml.push_str(&format!(
            "  <url><loc>{}{}</loc>",
            base_url,
            escape_html(path)
        ));
        if let Some(lastmod) = lastmod {
            xml.push_str(&format!("<lastmod>{}</lastmod>", lastmod));
        }
        xml.push_str(&format!("<priority>{}</priority></url>\n", priority));
    };

    push_url("/", "1.0", None);
    for path in [
        "/knowledge-base",
        "/categories",
        "/tags",
        "/authors",
        "/pinned",
        "/types",
        "/statuses",
        "/changelog",
    ] {
        push_url(path, "0.7", None);
    }

    for post in &published_posts {
        let lastmod = post.updated_at.format("%Y-%m-%d").to_string();
        push_url(&format!("/post/{}", post.slug), "0.8", Some(lastmod));
    }
    for category in &category_models {
        if published_category_ids.contains(&category.id) {
            push_url(&format!("/category/{}", category.slug), "0.6", None);
        }
    }
    for tag in &tag_models {
        if used_tag_ids.contains(&tag.id) {
            push_url(&format!("/tag/{}", tag.slug), "0.5", None);
        }
    }
    for author in &author_models {
        if published_author_ids.contains(&author.id) {
            push_url(&format!("/author/{}", author.id), "0.6", None);
        }
    }
    for article_type in &type_models {
        if used_article_types.contains(&article_type.code) {
            push_url(&format!("/type/{}", article_type.code), "0.5", None);
        }
    }
    for article_status in &status_models {
        if used_article_statuses.contains(&article_status.code) {
            push_url(&format!("/status/{}", article_status.code), "0.5", None);
        }
    }

    xml.push_str("</urlset>\n");

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert("Content-Type", "application/xml".parse().unwrap());
    Ok((resp_headers, xml))
}

/// Helper: derive base URL from request Host header
fn derive_base_url(headers: &HeaderMap, config_host: &str, config_port: u16) -> String {
    let scheme = if headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        == Some("https")
    {
        "https"
    } else {
        "http"
    };
    let raw_host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or(config_host)
        .to_string();
    let host_only = raw_host.split(':').next().unwrap_or(&raw_host);
    if host_only == "localhost"
        || host_only.starts_with("127.")
        || host_only.starts_with("192.168.")
    {
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
        build_meta_description, build_meta_title, build_social_image_url,
        compact_site_meta_description, display_site_title, inject_robots_meta,
        normalize_article_headings, render_spa_seo_shell, seo_display_width, spa_robots_directive,
        SeoPage,
    };

    #[test]
    fn site_title_display_removes_every_hyphen() {
        assert_eq!(display_site_title("Mark-Share-X"), "MarkShareX");
        assert_eq!(display_site_title("Mark---Share--X"), "MarkShareX");
        assert_eq!(display_site_title("-Mark-Share-X-"), "MarkShareX");
        assert_eq!(display_site_title("----"), "MarkShareX_用AI学AI");
    }

    #[test]
    fn social_image_prefers_cover_and_falls_back_to_embedded_default() {
        assert_eq!(
            build_social_image_url(
                Some("https://cdn.example.com/cover.jpg"),
                "https://www.xlevon.cn"
            ),
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
            site_title: "MarkShareX".to_string(),
            social_image: "https://www.xlevon.cn/default-og.png".to_string(),
            schema_type: "CollectionPage",
            heading: "知识库".to_string(),
            intro: "浏览技术文章".to_string(),
            content_html: "<ul><li><a href=\"/post/rust\">Rust 教程</a></li></ul>".to_string(),
        };

        let html = render_spa_seo_shell(shell, &page).expect("SPA shell should render");

        assert!(html.contains("<title>知识库 - MarkShareX</title>"));
        assert!(html.contains("<meta name=\"description\" content=\"浏览技术文章\">"));
        assert!(
            html.contains("<link rel=\"canonical\" href=\"https://www.xlevon.cn/knowledge-base\">")
        );
        assert!(html.contains(
            "<meta property=\"og:image\" content=\"https://www.xlevon.cn/default-og.png\">"
        ));
        assert!(html.contains("<meta name=\"twitter:card\" content=\"summary_large_image\">"));
        assert!(html.contains("\"@type\":\"CollectionPage\""));
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
    fn meta_title_keeps_site_name_when_the_combined_title_fits() {
        assert_eq!(
            build_meta_title("Rust 入门", "MarkShareX_用AI学AI"),
            "Rust 入门 - MarkShareX_用AI学AI"
        );
    }

    #[test]
    fn meta_title_drops_site_name_before_truncating_the_article_title() {
        assert_eq!(
            build_meta_title(
                "Rust 错误处理深度解构：Result 与 ? 操作符",
                "MarkShareX_用AI学AI",
            ),
            "Rust 错误处理深度解构：Result 与 ? 操作符"
        );
    }

    #[test]
    fn meta_title_truncates_an_overlong_article_title_by_display_width() {
        let result = build_meta_title(
            "深入理解 Rust 所有权生命周期错误处理异步编程与生产环境最佳实践完整指南",
            "MarkShareX_用AI学AI",
        );

        assert!(result.ends_with('…'));
        assert!(!result.contains("MarkShareX"));
        assert!(seo_display_width(&result) <= 60);
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
    fn meta_description_truncates_mixed_language_text_at_a_natural_break() {
        let result = build_meta_description(
            Some("深入解析 Rust 错误处理体系的核心：Result 枚举类型与 ? 传播操作符。从类型系统设计出发，涵盖错误转换、自定义错误类型、thiserror/anyhow 库的配合使用，以及生产环境中的最佳实践与常见陷阱。"),
            "",
            "站点描述",
            "文章标题",
        );

        assert_eq!(
            result,
            "深入解析 Rust 错误处理体系的核心：Result 枚举类型与 ? 传播操作符。从类型系统设计出发，涵盖错误转换、自定义错误类型、thiserror/anyhow 库的配合使用，…"
        );
        assert!(seo_display_width(&result) <= 160);
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

    #[test]
    fn empty_article_content_uses_unique_title_based_description() {
        let result = build_meta_description(None, "", "站点描述", "文章标题");
        assert_eq!(result, "文章标题：站点描述");
    }

    #[test]
    fn site_meta_description_uses_first_paragraph_and_excludes_later_credentials() {
        let result = compact_site_meta_description(
            "简洁的站点定位。\n继续说明。\n\n账号：admin@example.com/password",
            "fallback",
        );
        assert_eq!(result, "简洁的站点定位。 继续说明。");
        assert!(!result.contains("password"));
    }

    #[test]
    fn private_spa_routes_get_matching_noindex_directives() {
        assert_eq!(spa_robots_directive("/search"), Some("noindex, follow"));
        assert_eq!(spa_robots_directive("/login"), Some("noindex, nofollow"));
        assert_eq!(
            spa_robots_directive("/admin/news"),
            Some("noindex, nofollow")
        );
        assert_eq!(
            spa_robots_directive("/guestbook"),
            Some("noindex, nofollow")
        );
        let shell = "<html><head></head><body></body></html>";
        assert!(inject_robots_meta(shell, "noindex, follow")
            .contains("<meta name=\"robots\" content=\"noindex, follow\">"));
    }

    #[test]
    fn ssr_article_template_uses_runtime_meta_title_without_changing_the_h1() {
        let template = include_str!("../../templates/default/post.html");

        assert!(template.contains("{% block title %}{{ meta_title }}{% endblock %}"));
        assert!(template.contains("<meta property=\"og:title\" content=\"{{ meta_title }}\">"));
        assert!(template.contains("<meta name=\"twitter:title\" content=\"{{ meta_title }}\">"));
        assert!(template.contains("<h1>{{ post.title }}</h1>"));
    }

    #[test]
    fn ssr_article_taxonomy_links_use_slugs() {
        let template = include_str!("../../templates/default/post.html");
        assert!(template.contains("/category/{{ post.category_slug }}"));
        assert!(template.contains("/tag/{{ tag.slug }}"));
        assert!(!template.contains("/category/{{ post.category_name }}"));
        assert!(!template.contains("/tag/{{ tag }}"));
    }
}
