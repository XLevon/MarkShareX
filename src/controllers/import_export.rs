use crate::middleware::auth::AuthUser;
use crate::models::entity::network_resources;
use crate::models::entity::{categories, files, posts, users};
use crate::utils::{ApiResponse, AppError, AppState};
use axum::{
    extract::{Json, State},
    http::HeaderMap,
    response::IntoResponse,
};
use sea_orm::*;
use serde::{Deserialize, Serialize};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use regex::Regex;
use std::collections::HashMap;
use std::io::Write;

// ============== 导出功能 ==============

#[derive(Deserialize)]
pub struct ExportRequest {
    pub post_ids: Option<Vec<i32>>,
}

/// 构建文章的Front Matter
fn build_front_matter(
    post: &posts::Model,
    category_name: Option<&String>,
    tag_names: &[String],
    cover_url: Option<&str>,
) -> String {
    let mut lines = vec!["---".to_string()];

    lines.push(format!("title: \"{}\"", escape_yaml_string(&post.title)));

    if let Some(published_at) = post.published_at {
        lines.push(format!(
            "date: \"{}\"",
            published_at.format("%Y-%m-%d %H:%M:%S")
        ));
    } else {
        lines.push(format!(
            "date: \"{}\"",
            post.created_at.format("%Y-%m-%d %H:%M:%S")
        ));
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
    let mut query = posts::Entity::find().filter(posts::Column::DeletedAt.is_null());

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
        let rows = state
            .db
            .query_all(Statement::from_string(state.db.get_database_backend(), sql))
            .await?;
        let mut map: HashMap<i32, Vec<String>> = HashMap::new();
        for row in rows {
            if let (Ok(post_id), Ok(name)) = (
                row.try_get_by_index::<i32>(0),
                row.try_get_by_index::<String>(1),
            ) {
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
    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

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
                    if let Some(nr) = network_resources::Entity::find_by_id(nr_id)
                        .one(&state.db)
                        .await?
                    {
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
                match crate::services::files::read_storage_file(
                    &file.filename,
                    &state.config.storage.upload_dir,
                ) {
                    Ok(data) => image_files.push((file.filename.clone(), data)),
                    Err(AppError::IoError(error))
                        if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
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
                    match crate::services::files::read_storage_file(
                        &file.filename,
                        &state.config.storage.upload_dir,
                    ) {
                        Ok(data) => image_files.push((file.filename.clone(), data)),
                        Err(AppError::IoError(error))
                            if error.kind() == std::io::ErrorKind::NotFound => {}
                        Err(error) => return Err(error),
                    }
                }
            }
        }

        // 🔴 解析封面中的 nr:{id} 引用 → 实际 URL（导出时还原为外链）
        let resolved_cover = if let Some(cover) = post.cover_image.as_deref() {
            if let Some(id_str) = cover.strip_prefix("nr:") {
                if let Ok(nr_id) = id_str.parse::<i32>() {
                    if let Ok(Some(nr)) = network_resources::Entity::find_by_id(nr_id)
                        .one(&state.db)
                        .await
                    {
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

        let front_matter =
            build_front_matter(post, category_name, tag_names, resolved_cover.as_deref());
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
    headers.insert(
        "Content-Disposition",
        format!(
            "attachment; filename=\"marksharex_export_{}.zip\"",
            timestamp
        )
        .parse()
        .unwrap(),
    );
    headers.insert(
        "Content-Length",
        file_data.len().to_string().parse().unwrap(),
    );

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
    pub persisted_with_errors: Vec<i32>,
    pub errors: Vec<String>,
}

async fn rollback_import_files(
    state: &AppState,
    mutation_session: &crate::services::files::UploadMutationSession,
    created_file_ids: &[i32],
) -> Result<(), AppError> {
    if created_file_ids.is_empty() {
        return Ok(());
    }
    let deleted = crate::services::files::batch_delete_files_in_session(
        mutation_session,
        &state.db,
        created_file_ids,
        &state.config.storage.upload_dir,
    )
    .await?;
    if deleted != created_file_ids.len() {
        return Err(AppError::Internal(anyhow::anyhow!(
            "导入文件补偿不完整: 期望删除 {} 个，实际删除 {} 个",
            created_file_ids.len(),
            deleted
        )));
    }
    Ok(())
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
    if !matches!(auth.role.as_str(), "author" | "sub_admin" | "admin") {
        return Err(AppError::Forbidden);
    }

    let mut results = ImportResult {
        success: true,
        message: "".to_string(),
        imported_count: 0,
        skipped_count: 0,
        persisted_with_errors: Vec::new(),
        errors: Vec::new(),
    };

    for item in &req.items {
        let item_error_start = results.errors.len();
        let (meta, tags, mut body) = parse_front_matter(&item.content);

        // Fallback: extract tags from body for CSDN-style exports without YAML front matter tags
        let tags = if tags.is_empty() {
            extract_tags_from_body(&body)
        } else {
            tags
        };

        // 处理图片上传：先对整篇文章的 data URL 做无副作用预检，全部通过后才落盘。
        let mut image_url_map: HashMap<String, String> = HashMap::new();
        let mut prepared_uploads: Vec<(String, String, Vec<u8>)> = Vec::new();
        for image in &item.images {
            if !image.data.starts_with("data:image/") {
                continue;
            }
            let original_filename = &image.name;
            let Some((mime_part, encoded)) = image.data.split_once(',') else {
                results
                    .errors
                    .push(format!("图片数据格式无效: {original_filename}"));
                continue;
            };
            let image_bytes = match STANDARD.decode(encoded) {
                Ok(bytes) => bytes,
                Err(error) => {
                    results
                        .errors
                        .push(format!("图片 Base64 解码失败 {original_filename}: {error}"));
                    continue;
                }
            };
            let mime = mime_part
                .strip_prefix("data:")
                .unwrap_or(mime_part)
                .split(';')
                .next()
                .unwrap_or("application/octet-stream")
                .to_string();

            match crate::services::files::preflight_upload(
                original_filename,
                &mime,
                &image_bytes,
                &state.config.storage.allowed_types,
                state.config.storage.max_file_size,
                None,
            ) {
                Ok(_) => prepared_uploads.push((original_filename.clone(), mime, image_bytes)),
                Err(error) => results
                    .errors
                    .push(format!("图片上传失败 {original_filename}: {error}")),
            }
        }

        if results.errors.len() > item_error_start {
            results.skipped_count += 1;
            continue;
        }

        let mutation_session = crate::services::files::begin_upload_mutation().await;
        let mut created_file_ids = Vec::new();
        for (original_filename, mime, image_bytes) in prepared_uploads {
            match crate::services::files::upload_file_in_session(
                &mutation_session,
                &state.db,
                auth.user_id,
                &original_filename,
                &mime,
                &image_bytes,
                &state.config.storage.upload_dir,
                &state.config.storage.allowed_types,
                state.config.storage.max_file_size,
                None,
                false,
            )
            .await
            {
                Ok(outcome) => {
                    if outcome.created {
                        created_file_ids.push(outcome.file.id);
                    }
                    let file_url = crate::services::files::get_file_url(&outcome.file.filename);
                    image_url_map.insert(original_filename, file_url);
                }
                Err(error) => results
                    .errors
                    .push(format!("图片上传失败 {original_filename}: {error}")),
            }
        }

        if results.errors.len() > item_error_start {
            rollback_import_files(&state, &mutation_session, &created_file_ids).await?;
            results.skipped_count += 1;
            continue;
        }

        for image in &item.images {
            let original_filename = &image.name;
            if image.data.starts_with("data:image/") {
                continue;
            }
            if image.data.contains("/uploads/") {
                // 自托管图片 → 作为本地资源处理，提取文件名
                let relative = image.data.split("/uploads/").last().unwrap_or(&image.data);
                let relative = relative.trim_start_matches("./").trim_start_matches('/');
                let filename = std::path::Path::new(relative)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(relative);
                // 检查文件是否已存在（按文件名）
                let existing = match files::Entity::find()
                    .filter(files::Column::Filename.eq(filename))
                    .filter(files::Column::DeletedAt.is_null())
                    .one(&state.db)
                    .await
                {
                    Ok(existing) => existing,
                    Err(error) => {
                        rollback_import_files(&state, &mutation_session, &created_file_ids).await?;
                        return Err(error.into());
                    }
                };
                if let Some(existing) = existing {
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
            let nr = match network_resources::Entity::find()
                .filter(network_resources::Column::Url.eq(url))
                .one(&state.db)
                .await
            {
                Ok(nr) => nr,
                Err(error) => {
                    rollback_import_files(&state, &mutation_session, &created_file_ids).await?;
                    return Err(error.into());
                }
            };
            if let Some(nr) = nr {
                nr_replacements.push((url.to_string(), format!("nr:{}", nr.id)));
            }
        }
        for (url, nr_ref) in &nr_replacements {
            body = body.replace(url, nr_ref);
        }

        let category_name = meta
            .get("category")
            .or_else(|| meta.get("categories"))
            .map(String::as_str);

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
            category_name,
            &status,
            &tags,
            explicit_slug,
            cover_url,
        )
        .await;

        match result {
            Ok(_) => results.imported_count += 1,
            Err(e) => {
                if e.rollback_files {
                    rollback_import_files(&state, &mutation_session, &created_file_ids).await?;
                }
                results.errors.push(format!("「{}」: {}", title, e.error));
                if let Some(post_id) = e.persisted_post_id {
                    results.imported_count += 1;
                    results.persisted_with_errors.push(post_id);
                } else {
                    results.skipped_count += 1;
                }
            }
        }
    }

    results.message = if results.errors.is_empty() {
        format!("成功导入 {} 篇文章", results.imported_count)
    } else {
        format!(
            "导入完成，已持久化: {}, 未导入: {}, 异常: {}",
            results.imported_count,
            results.skipped_count,
            results.errors.len()
        )
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
            let is_tag_line = line.starts_with("tags:")
                || line.starts_with("tags：")
                || line.starts_with("tag:")
                || line.starts_with("tag：");

            if is_tag_line {
                let val = line
                    .replacen("tags:", "", 1)
                    .replacen("tags：", "", 1)
                    .replacen("tag:", "", 1)
                    .replacen("tag：", "", 1);
                let val = val.trim().trim_matches('"').trim();
                if !val.is_empty() {
                    // Support bracket notation: [a, b, c]
                    let val = val.trim_start_matches('[').trim_end_matches(']').trim();
                    for t in val.split(',') {
                        let t = t.trim().trim_matches('"').trim_matches('\'').trim();
                        if !t.is_empty() {
                            tags.push(t.to_string());
                        }
                    }
                } else {
                    in_tags = true;
                }
                continue;
            }

            if in_tags {
                if let Some(caps) = list_item_re.captures(line) {
                    tags.push(caps.get(1).unwrap().as_str().to_string());
                    continue;
                }
                if line.trim().is_empty() {
                    continue;
                }
                in_tags = false;
            }
            if let Some(caps) = key_value_re.captures(line) {
                meta.insert(
                    caps.get(1).unwrap().as_str().to_string(),
                    caps.get(2).unwrap().as_str().to_string(),
                );
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
    category_name: Option<&str>,
    status: &str,
    tags: &[String],
    explicit_slug: Option<&str>,
    cover_url: Option<&str>,
) -> Result<posts::Model, crate::services::import_export::ImportPostError> {
    crate::services::import_export::create_post_from_import(
        state,
        user_id,
        title,
        content,
        summary,
        category_name,
        status,
        tags,
        explicit_slug,
        cover_url,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::parse_front_matter;

    #[test]
    fn front_matter_fields_after_multiline_tags_are_preserved() {
        let (meta, tags, body) = parse_front_matter(
            "---\ntitle: Ordered fields\ntags:\n  - rust\n  - sqlite\nstatus: published\nslug: ordered-fields\n---\nbody",
        );

        assert_eq!(tags, vec!["rust", "sqlite"]);
        assert_eq!(meta.get("status").map(String::as_str), Some("published"));
        assert_eq!(meta.get("slug").map(String::as_str), Some("ordered-fields"));
        assert_eq!(body, "body");
    }
}
