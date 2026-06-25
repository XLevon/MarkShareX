use axum::{extract::{State, Path}, response::{Html, Response}, http::{HeaderMap, header, StatusCode}};
use crate::utils::{AppState, AppError};
use serde_json::json;
use sea_orm::*;
use std::collections::HashSet;
use std::error::Error as _;
use crate::models::entity::posts;

async fn get_setting(db: &sea_orm::DatabaseConnection, key: &str) -> Option<String> {
    crate::models::entity::settings::Entity::find()
        .filter(crate::models::entity::settings::Column::Key.eq(key))
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|s| s.value)
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

    let category_name = crate::services::posts::get_category_name(&state.db, post.category_id).await;
    let tags = crate::services::posts::get_post_tags(&state.db, post.id).await?;

    let site_title = get_setting(&state.db, "site_title").await.unwrap_or_else(|| "MarkShareX".to_string());
    let site_description = get_setting(&state.db, "site_description").await.unwrap_or_default();

    let mut cover_image = crate::controllers::network_resources::resolve_post_cover(
        &state.db,
        post.cover_network_id,
        post.cover_image_url.as_deref(),
        post.cover_image_filename.as_deref(),
        post.cover_image.as_deref(),
    ).await;

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

    // Make cover_image absolute for OG meta tags (WeChat requires full URL)
    if let Some(ref ci) = cover_image {
        if !ci.starts_with("http://") && !ci.starts_with("https://") {
            let abs_url = format!("{}{}", base_url, ci);
            cover_image = Some(abs_url);
        }
    }

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

    // Add heading IDs for anchor link support (comrak doesn't auto-generate them)
    resolved_html = add_heading_ids(&resolved_html);

    let article_url = format!("{}/post/{}", base_url, post.slug);

    let mut ctx = tera::Context::new();
    ctx.insert("site_title", &site_title);
    ctx.insert("site_description", &site_description);
    ctx.insert("base_url", &base_url);
    ctx.insert("article_url", &article_url);
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

// ── SEO endpoints ──

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
