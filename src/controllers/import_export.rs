use axum::{extract::{State, Json}, response::IntoResponse, http::HeaderMap};
use serde::{Deserialize, Serialize};
use crate::utils::{AppState, AppError, ApiResponse};
use crate::middleware::auth::AuthUser;
use crate::models::entity::{posts, users, categories, tags, post_tags, files};
use crate::models::entity::network_resources;
use sea_orm::*;

use std::collections::HashMap;
use regex::Regex;
use std::io::Write;
use base64::{Engine as _, engine::general_purpose::STANDARD};

// ============== 导出功能 ==============

#[derive(Deserialize)]
pub struct ExportRequest {
    pub post_ids: Option<Vec<i32>>,
}

/// 构建文章的Front Matter
fn build_front_matter(post: &posts::Model, category_name: Option<&String>, tag_names: &[String], cover_url: Option<&str>) -> String {
    let mut lines = vec!["---".to_string()];
    
    lines.push(format!("title: \"{}\"", escape_yaml_string(&post.title)));
    
    if let Some(published_at) = post.published_at {
        lines.push(format!("date: \"{}\"", published_at.format("%Y-%m-%d %H:%M:%S")));
    } else {
        lines.push(format!("date: \"{}\"", post.created_at.format("%Y-%m-%d %H:%M:%S")));
    }
    
    lines.push(format!("status: \"{}\"", post.status));
    lines.push(format!("draft: {}", post.status == "draft"));
    
    if let Some(name) = category_name {
        lines.push(format!("category: \"{}\"", escape_yaml_string(name)));
    }
    
    if !tag_names.is_empty() {
        lines.push("tags:".to_string());
        for tag in tag_names {
            lines.push(format!("  - \"{}\"", escape_yaml_string(tag)));
        }
    }
    
    if let Some(summary) = &post.summary {
        lines.push(format!("summary: \"{}\"", escape_yaml_string(summary)));
    }
    
    lines.push(format!("slug: \"{}\"", post.slug));
    if let Some(cover) = cover_url {
        lines.push(format!("cover_url: \"{}\"", escape_yaml_string(cover)));
    }
    lines.push(format!("author_id: {}", post.user_id));
    
    lines.push("---".to_string());
    lines.join("\n")
}

fn escape_yaml_string(s: &str) -> String {
    s.replace("\\", "\\\\")
     .replace("\"", "\\\"")
     .replace("\n", "\\n")
     .replace("\r", "\\r")
}

fn extract_image_urls(content: &str) -> Vec<String> {
    let re = Regex::new(r#"!\[.*?\]\(([^)]+)\)"#).unwrap();
    let mut urls = Vec::new();
    for cap in re.captures_iter(content) {
        if let Some(url) = cap.get(1) {
            let url_str = url.as_str();
            if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
                // Relative path (e.g. ./uploads/xxx) → local file
                urls.push(url_str.to_string());
            } else if url_str.contains("/uploads/") {
                // Self-hosted absolute URL → treat as local, extract filename
                let filename = url_str.split('/').last().unwrap_or(url_str);
                urls.push(filename.to_string());
            }
        }
    }
    urls
}

fn get_filename_from_url(url: &str) -> String {
    url.split('/').last().unwrap_or(url).to_string()
}
/// POST /api/v1/export/posts — Export posts as ZIP

#[utoipa::path(
    post,
    path = "/api/v1/export/posts",
    responses((status = 200, description = "成功")),
    tag = "Import/Export"
)]
pub async fn export_posts(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ExportRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut query = posts::Entity::find()
        .filter(posts::Column::DeletedAt.is_null());
    
    if !auth.is_privileged() {
        query = query.filter(posts::Column::UserId.eq(auth.user_id));
    }
    
    if let Some(post_ids) = req.post_ids {
        query = query.filter(posts::Column::Id.is_in(post_ids));
    }
    
    let posts_list = query.all(&state.db).await?;
    
    if posts_list.is_empty() {
        return Err(AppError::BadRequest("没有找到可导出的文章".to_string()));
    }
    
    let category_ids: Vec<i32> = posts_list.iter().filter_map(|p| p.category_id).collect();
    let categories_map: HashMap<i32, String> = if !category_ids.is_empty() {
        categories::Entity::find()
            .filter(categories::Column::Id.is_in(category_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|c| (c.id, c.name))
            .collect()
    } else {
        HashMap::new()
    };
    
    let post_ids_str: Vec<String> = posts_list.iter().map(|p| p.id.to_string()).collect();
    let tags_map: HashMap<i32, Vec<String>> = if !post_ids_str.is_empty() {
        let sql = format!(
            "SELECT pt.post_id, t.name FROM post_tags pt JOIN tags t ON pt.tag_id = t.id WHERE pt.post_id IN ({}) ORDER BY t.name",
            post_ids_str.join(",")
        );
        let rows = state.db.query_all(Statement::from_string(state.db.get_database_backend(), sql)).await?;
        let mut map: HashMap<i32, Vec<String>> = HashMap::new();
        for row in rows {
            if let (Ok(post_id), Ok(name)) = (row.try_get_by_index::<i32>(0), row.try_get_by_index::<String>(1)) {
                map.entry(post_id).or_default().push(name);
            }
        }
        map
    } else {
        HashMap::new()
    };
    
    let user_ids: Vec<i32> = posts_list.iter().map(|p| p.user_id).collect();
    let users_map: HashMap<i32, String> = if !user_ids.is_empty() {
        users::Entity::find()
            .filter(users::Column::Id.is_in(user_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|u| (u.id, u.display_name.unwrap_or(u.username)))
            .collect()
    } else {
        HashMap::new()
    };
    
    
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let zip_buf = std::io::Cursor::new(Vec::new());
    
    let mut zip_writer = zip::ZipWriter::new(zip_buf);
    let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    
    for post in &posts_list {
        let category_name = post.category_id.and_then(|cid| categories_map.get(&cid));
        let empty_tags = vec![];
        let tag_names = tags_map.get(&post.id).unwrap_or(&empty_tags);
        
        let unknown_name = "unknown".to_string();
        let author_name = users_map.get(&post.user_id).unwrap_or(&unknown_name);
        let date_str = post.created_at.format("%Y%m%d").to_string();
        let safe_title = sanitize_filename(&post.title);
        let dir_name = format!("{}_{}_{}", author_name, date_str, safe_title);
        
        let content = post.content.as_deref().unwrap_or("");
        
        // 🔴 解析内容中的 nr:{id} 引用 → 实际 URL（导出时还原为外链）
        let nr_re = Regex::new(r"!\[([^\]]*)\]\(nr:(\d+)\)").unwrap();
        let mut nr_resolved_content = content.to_string();
        if nr_re.is_match(content) {
            for cap in nr_re.captures_iter(content) {
                let nr_id: i32 = cap.get(2).unwrap().as_str().parse().unwrap_or(0);
                if nr_id > 0 {
                    if let Some(nr) = network_resources::Entity::find_by_id(nr_id).one(&state.db).await? {
                        let full_match = cap.get(0).unwrap().as_str();
                        let alt = cap.get(1).unwrap().as_str();
                        let replacement = format!("![{}]({})", alt, nr.url);
                        nr_resolved_content = nr_resolved_content.replace(full_match, &replacement);
                    }
                }
            }
        }
        let content = nr_resolved_content.as_str();
        
        let image_urls = extract_image_urls(content);
        
        let mut image_files: Vec<(String, Vec<u8>)> = Vec::new();
        
        for url in &image_urls {
            let filename = get_filename_from_url(url);
            
            if let Some(file) = files::Entity::find()
                .filter(files::Column::Filename.like(format!("%{}", filename).as_str()))
                .filter(files::Column::DeletedAt.is_null())
                .one(&state.db)
                .await?
            {
                let storage_path = crate::services::files::get_storage_path(&file.filename, &state.config.storage.upload_dir);
                if let Ok(data) = std::fs::read(&storage_path) {
                    // Content already uses ./uploads/filename — no replacement needed
                    image_files.push((file.filename.clone(), data));
                }
            }
        }
        
        if let Some(cover_url) = &post.cover_image {
            if !cover_url.is_empty() {
                let cover_filename = get_filename_from_url(cover_url);
                if let Some(file) = files::Entity::find()
                    .filter(files::Column::Filename.like(format!("%{}", cover_filename).as_str()))
                    .filter(files::Column::DeletedAt.is_null())
                    .one(&state.db)
                    .await?
                {
                    let storage_path = crate::services::files::get_storage_path(&file.filename, &state.config.storage.upload_dir);
                    if let Ok(data) = std::fs::read(&storage_path) {
                        image_files.push((file.filename.clone(), data));
                    }
                }
            }
        }
        
        // 🔴 解析封面中的 nr:{id} 引用 → 实际 URL（导出时还原为外链）
        let resolved_cover = if let Some(cover) = post.cover_image.as_deref() {
            if let Some(id_str) = cover.strip_prefix("nr:") {
                if let Ok(nr_id) = id_str.parse::<i32>() {
                    if let Ok(Some(nr)) = network_resources::Entity::find_by_id(nr_id).one(&state.db).await {
                        Some(nr.url)
                    } else {
                        Some(cover.to_string())
                    }
                } else {
                    Some(cover.to_string())
                }
            } else {
                Some(cover.to_string())
            }
        } else {
            None
        };

        let front_matter = build_front_matter(post, category_name, tag_names, resolved_cover.as_deref());
        // 将内容中的完整URL替换为相对路径，确保导出后可再次导入
        let normalized_content = content
            .replace("https://www.xlevon.cn/uploads/", "./uploads/")
            .replace("http://www.xlevon.cn/uploads/", "./uploads/");
        let full_content = format!("{}\n\n{}", front_matter, normalized_content);
        
        let md_path = format!("{}/index.md", dir_name);
        zip_writer.start_file(md_path, options)?;
        zip_writer.write(full_content.as_bytes())?;
        
        for (filename, data) in image_files {
            let img_path = format!("{}/uploads/{}", dir_name, filename);
            zip_writer.start_file(img_path, options)?;
            zip_writer.write(&data)?;
        }
    }
    
    let file_data = zip_writer.finish()?.into_inner();
    
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/zip".parse().unwrap());
    headers.insert("Content-Disposition", format!("attachment; filename=\"marksharex_export_{}.zip\"", timestamp).parse().unwrap());
    headers.insert("Content-Length", file_data.len().to_string().parse().unwrap());
    
    Ok((headers, file_data))
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            _ => c,
        })
        .collect()
}

// ============== 导入功能 ==============

#[derive(Serialize)]
pub struct ImportResult {
    pub success: bool,
    pub message: String,
    pub imported_count: usize,
    pub skipped_count: usize,
    pub errors: Vec<String>,
}

#[derive(Deserialize)]
pub struct ImportRequest {
    pub items: Vec<ImportItem>,
}

#[derive(Deserialize)]
pub struct ImportImage {
    pub data: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct ImportItem {
    pub filename: String,
    pub content: String,
    pub images: Vec<ImportImage>,
}
/// POST /api/v1/import/posts — Import posts from ZIP

#[utoipa::path(
    post,
    path = "/api/v1/import/posts",
    responses((status = 200, description = "成功")),
    tag = "Import/Export"
)]
pub async fn import_markdown(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ImportRequest>,
) -> Result<Json<ApiResponse<ImportResult>>, AppError> {
    let mut results = ImportResult {
        success: true,
        message: "".to_string(),
        imported_count: 0,
        skipped_count: 0,
        errors: Vec::new(),
    };
    
    for item in &req.items {
        let (meta, tags, mut body) = parse_front_matter(&item.content);
        
        // Fallback: extract tags from body for CSDN-style exports without YAML front matter tags
        let tags = if tags.is_empty() {
            extract_tags_from_body(&body)
        } else {
            tags
        };
        
        // 处理图片上传
        let mut image_url_map: HashMap<String, String> = HashMap::new();
        for image in &item.images {
            let original_filename = &image.name;
            // 传入的可能带路径前缀（如 ./uploads/xxx.svg），上传时只用纯文件名
            let upload_name = std::path::Path::new(original_filename)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(original_filename);
            
            // 检查是否是base64格式的本地图片
            if image.data.starts_with("data:image/") {
                // 本地图片，需要上传处理
                if let Some((mime_part, data)) = image.data.split_once(",") {
                    if let Ok(image_bytes) = STANDARD.decode(data) {
                        // 解析MIME类型，去掉data:前缀和;base64后缀
                        let mime = mime_part
                            .strip_prefix("data:")
                            .unwrap_or(mime_part)
                            .split(';')
                            .next()
                            .unwrap_or("image/png");
                        
                        // 计算MD5哈希值
                        let md5_hash = crate::services::files::calculate_md5(&image_bytes);
                        
                        // 检查是否已存在相同MD5的文件
                        if let Some(existing_file) = files::Entity::find()
                            .filter(
                                sea_orm::Condition::all()
                                    .add(files::Column::Md5Hash.eq(&md5_hash))
                                    .add(files::Column::DeletedAt.is_null()),
                            )
                            .one(&state.db)
                            .await?
                        {
                            // 直接使用已存在的文件URL
                            let file_url = crate::services::files::get_file_url(&existing_file.filename);
                            image_url_map.insert(original_filename.clone(), file_url);
                        } else {
                            // 不存在相同MD5的文件，上传新文件
                            if let Ok(file) = crate::services::files::upload_file(
                                &state.db,
                                auth.user_id,
                                upload_name,
                                mime,
                                &image_bytes,
                                &state.config.storage.upload_dir,
                                None,
                                false, // 不覆盖，让服务处理重名情况
                            ).await {
                                let file_url = crate::services::files::get_file_url(&file.filename);
                                image_url_map.insert(original_filename.clone(), file_url);
                            }
                        }
                    }
                }
            } else if image.data.contains("/uploads/") {
                // 自托管图片 → 作为本地资源处理，提取文件名
                let relative = image.data.split("/uploads/").last().unwrap_or(&image.data);
                let relative = relative.trim_start_matches("./").trim_start_matches('/');
                let filename = std::path::Path::new(relative)
                    .file_name().and_then(|n| n.to_str()).unwrap_or(relative);
                // 检查文件是否已存在（按文件名）
                if let Some(existing) = files::Entity::find()
                    .filter(files::Column::Filename.eq(filename))
                    .filter(files::Column::DeletedAt.is_null())
                    .one(&state.db).await?
                {
                    let file_url = crate::services::files::get_file_url(&existing.filename);
                    image_url_map.insert(original_filename.clone(), file_url);
                } else {
                    // 本地文件不存在，保留原 URL
                    image_url_map.insert(original_filename.clone(), image.data.clone());
                }
            } else {
                // 外链图片，直接使用
                image_url_map.insert(original_filename.clone(), image.data.clone());
            }
        }
        
        // 替换文章中的图片引用
        for (old_filename, new_url) in &image_url_map {
            // 只替换相对路径的图片引用，不替换外链URL
            // 检查是否是相对路径（不包含http://或https://）
            if !old_filename.starts_with("http://") && !old_filename.starts_with("https://") {
                body = body.replace(old_filename, new_url);
            }
        }
        
        // 🔴 扫描外网 URL，比对 network_resources 表
        let external_re = Regex::new(r#"!\[[^\]]*\]\((https?://[^)]+)\)"#).unwrap();
        let mut nr_replacements: Vec<(String, String)> = Vec::new();
        for cap in external_re.captures_iter(&body) {
            let url = cap.get(1).unwrap().as_str();
            // 跳过本地上传 URL
            if url.contains("/uploads/") {
                continue;
            }
            // 查 network_resources 表
            if let Some(nr) = network_resources::Entity::find()
                .filter(network_resources::Column::Url.eq(url))
                .one(&state.db).await?
            {
                nr_replacements.push((url.to_string(), format!("nr:{}", nr.id)));
            }
        }
        for (url, nr_ref) in &nr_replacements {
            body = body.replace(url, nr_ref);
        }
        
        let category_id = if let Some(cat_name) = meta.get("category").or_else(|| meta.get("categories")) {
            if let Some(cat) = categories::Entity::find()
                .filter(categories::Column::Name.eq(cat_name))
                .one(&state.db)
                .await?
            {
                Some(cat.id)
            } else {
                let cat = categories::ActiveModel {
                    name: Set(cat_name.clone()),
                    slug: Set(crate::services::posts::generate_slug(cat_name)),
                    ..Default::default()
                }.insert(&state.db).await?;
                Some(cat.id)
            }
        } else {
            None
        };
        
        let title = meta.get("title").unwrap_or(&item.filename).clone();
        let summary = meta.get("summary").cloned();
        let status = meta.get("status").unwrap_or(&"draft".to_string()).clone();
        let explicit_slug = meta.get("slug").map(|s| s.as_str());
        let cover_url = meta.get("cover_url").map(|s| s.as_str());

        let result = create_post_from_import(
            &state,
            auth.user_id,
            &title,
            &body,
            summary.as_deref(),
            category_id,
            &status,
            &tags,
            explicit_slug,
            cover_url,
        ).await;
        
        match result {
            Ok(_) => results.imported_count += 1,
            Err(e) => {
                results.errors.push(format!("「{}」: {}", title, e));
                results.skipped_count += 1;
            }
        }
    }
    
    results.message = if results.errors.is_empty() {
        format!("成功导入 {} 篇文章", results.imported_count)
    } else {
        format!("导入完成，成功: {}, 失败: {}", results.imported_count, results.skipped_count)
    };
    
    results.success = results.errors.is_empty();
    
    Ok(Json(ApiResponse::new(results)))
}

fn parse_front_matter(content: &str) -> (HashMap<String, String>, Vec<String>, String) {
    // Normalize Windows line endings (\r\n → \n) for regex compatibility
    let content = content.replace("\r\n", "\n");
    let re = Regex::new(r#"^---\n([\s\S]*?)\n---\n?([\s\S]*)$"#).unwrap();
    let mut meta: HashMap<String, String> = HashMap::new();
    let mut tags: Vec<String> = Vec::new();
    
    if let Some(caps) = re.captures(&content) {
        let yaml_content = caps.get(1).unwrap().as_str();
        let body = caps.get(2).unwrap().as_str();
        
        // Build regexes once
        let list_item_re = Regex::new(r#"^\s*-\s*"?([^"]+)"?\s*$"#).unwrap();
        let key_value_re = Regex::new(r#"^(\w+):\s*"?([^"]+)"?\s*$"#).unwrap();
        
        let mut in_tags = false;
        for line in yaml_content.lines() {
            // Support both "tags:" and "tag:" (CSDN uses singular too)
            let is_tag_line = line.starts_with("tags:") || line.starts_with("tags：") 
                || line.starts_with("tag:") || line.starts_with("tag：");
            
            if is_tag_line {
                let val = line
                    .replacen("tags:", "", 1).replacen("tags：", "", 1)
                    .replacen("tag:", "", 1).replacen("tag：", "", 1);
                let val = val.trim().trim_matches('"').trim();
                if !val.is_empty() {
                    // Support bracket notation: [a, b, c]
                    let val = val.trim_start_matches('[').trim_end_matches(']').trim();
                    for t in val.split(',') {
                        let t = t.trim().trim_matches('"').trim_matches('\'').trim();
                        if !t.is_empty() { tags.push(t.to_string()); }
                    }
                } else {
                    in_tags = true;
                }
                continue;
            }
            
            if in_tags {
                if let Some(caps) = list_item_re.captures(line) {
                    tags.push(caps.get(1).unwrap().as_str().to_string());
                } else if !line.trim().is_empty() && !line.starts_with("-") {
                    in_tags = false;
                }
            } else if let Some(caps) = key_value_re.captures(line) {
                meta.insert(caps.get(1).unwrap().as_str().to_string(), caps.get(2).unwrap().as_str().to_string());
            }
        }
        
        (meta, tags, body.to_string())
    } else {
        (meta, tags, content.to_string())
    }
}

/// Extract tags from body text when YAML front matter doesn't contain them.
/// Handles CSDN-style tag lines like "**标签:** tag1, tag2" or "标签：tag1 tag2"
/// Also extracts 【TAG】 patterns from the body (CSDN title convention).
fn extract_tags_from_body(body: &str) -> Vec<String> {
    let mut tags = Vec::new();
    
    // 1. Extract 【TAG】 brackets from body (CSDN title convention, e.g. @[TOC](【ROS2】...))
    let bracket_re = Regex::new(r"【([^】]+)】").unwrap();
    for cap in bracket_re.captures_iter(body) {
        let tag = cap.get(1).unwrap().as_str().trim().to_string();
        if !tag.is_empty() && tag.len() < 50 && !tags.contains(&tag) {
            tags.push(tag);
        }
    }
    
    // 2. Tag lines in body: **标签:** tag1, tag2 / 标签：tag1 tag2
    let re = Regex::new(r"(?im)^\s*(?:\*\*)?标[签籤]s?\s*(?::|：)\s*(?:\*\*\s*)?(.+)$").unwrap();
    if let Some(caps) = re.captures(body) {
        let raw = caps.get(1).unwrap().as_str();
        for t in raw.split(&[',', '，', ' ', '　'][..]) {
            let t = t.trim().trim_matches('*').trim();
            if !t.is_empty() && t.len() < 50 && !tags.contains(&t.to_string()) {
                tags.push(t.to_string());
            }
        }
    }
    
    tags
}

async fn create_post_from_import(
    state: &AppState,
    user_id: i32,
    title: &str,
    content: &str,
    summary: Option<&str>,
    category_id: Option<i32>,
    status: &str,
    tags: &[String],
    explicit_slug: Option<&str>,
    cover_url: Option<&str>,
) -> Result<posts::Model, AppError> {
    // 有明确 slug：用它去重；无 slug：从 title 生成并自动去重
    let slug = if let Some(s) = explicit_slug {
        if posts::Entity::find()
            .filter(posts::Column::Slug.eq(s))
            .filter(posts::Column::DeletedAt.is_null())
            .one(&state.db)
            .await?
            .is_some()
        {
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
            .one(&state.db)
            .await?
            .is_some()
        {
            slug = format!("{}-{}", base, counter);
            counter += 1;
        }
        slug
    };
    let content_html = Some(crate::services::posts::render_markdown(&state.db, content).await);
    let now = crate::utils::now_local();
    let published_at = if status == "published" { Some(now) } else { None };

    // nr:{id} → 验证网络资源；普通外链 → 不入库
    let cover_network_id = if let Some(url) = cover_url {
        if url.starts_with("nr:") {
            super::network_resources::ensure_url(&state.db, url).await?
        } else {
            None
        }
    } else {
        None
    };
    // 拆分为新字段
    let (cover_image_url, cover_image_filename) = match cover_url {
        Some(url) if url.starts_with("nr:") || url.starts_with("http://") || url.starts_with("https://") => {
            (Some(url.to_string()), None)
        }
        Some(name) => (None, Some(name.to_string())),
        None => (None, None),
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
    }.insert(&state.db).await?;
    
    for tag_name in tags {
        let tag_id = if let Some(tag) = tags::Entity::find()
            .filter(tags::Column::Name.eq(tag_name))
            .one(&state.db)
            .await?
        {
            tag.id
        } else {
            // Create new tag
            let slug = crate::services::posts::generate_slug(tag_name);
            tags::ActiveModel {
                name: Set(tag_name.to_string()),
                slug: Set(slug),
                user_id: Set(Some(user_id)),
                deleted_at: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }.insert(&state.db).await?.id
        };
        let _ = post_tags::ActiveModel {
            post_id: Set(post.id),
            tag_id: Set(tag_id),
        }.insert(&state.db).await;
    }
    
    Ok(post)
}


