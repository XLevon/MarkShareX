use sea_orm::*;
use crate::models::entity::{files, posts, categories, settings};
use crate::utils::AppError;
use md5::{Md5, Digest};

/// Sanitize filename for URL safety — replace spaces and brackets with hyphens
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            ' ' | '(' | ')' | '[' | ']' | '{' | '}' | '#' | '%' | '&' | '?' => '-',
            _ => c,
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Calculate MD5 hash of data
pub fn calculate_md5(data: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.input(data);
    let result = hasher.result();
    format!("{:x}", result)
}

/// Get storage path from filename and upload_dir
pub fn get_storage_path(filename: &str, upload_dir: &str) -> String {
    std::path::Path::new(upload_dir).join(filename).to_string_lossy().to_string()
}

/// Get file URL from filename (relative, browser resolves based on current origin)
pub fn get_file_url(filename: &str) -> String {
    format!("/uploads/{}", filename)
}

pub async fn upload_file(
    db: &DatabaseConnection,
    user_id: i32,
    original_name: &str,
    content_type: &str,
    data: &[u8],
    upload_dir: &str,
    rename_to: Option<&str>,
    overwrite: bool,
) -> Result<files::Model, AppError> {
    // Calculate MD5 hash for deduplication
    let md5_hash = calculate_md5(data);
    
    // Check if file with same MD5 already exists
    if let Some(existing_file) = files::Entity::find()
        .filter(
            Condition::all()
                .add(files::Column::Md5Hash.eq(&md5_hash))
                .add(files::Column::DeletedAt.is_null()),
        )
        .one(db)
        .await?
    {
        return Ok(existing_file);
    }

    let ext = std::path::Path::new(original_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");

    // Determine filename
    let raw_filename = if let Some(name) = rename_to {
        if std::path::Path::new(name).extension().is_some() {
            name.to_string()
        } else {
            format!("{}.{}", name, ext)
        }
    } else {
        original_name.to_string()
    };

    // Sanitize: replace spaces + problematic chars for URL safety
    let filename = sanitize_filename(&raw_filename);

    // Check for duplicate if not overwriting — generate new filename with suffix
    let mut final_filename = filename.clone();
    if !overwrite {
        let stem = std::path::Path::new(&filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&filename);
        
        let mut counter = 1;
        while files::Entity::find()
            .filter(
                Condition::all()
                    .add(files::Column::Filename.eq(&final_filename))
                    .add(files::Column::DeletedAt.is_null()),
            )
            .one(db)
            .await?
            .is_some()
        {
            final_filename = format!("{}-{}.{}", stem, counter, ext);
            counter += 1;
        }
    }

    // If overwriting, soft-delete existing
    if overwrite {
        if let Some(existing) = files::Entity::find()
            .filter(
                Condition::all()
                    .add(files::Column::Filename.eq(&final_filename))
                    .add(files::Column::DeletedAt.is_null()),
            )
            .one(db)
            .await?
        {
            let mut active: files::ActiveModel = existing.into();
            let now = crate::utils::now_local();
            active.deleted_at = Set(Some(now));
            active.updated_at = Set(now);
            active.update(db).await?;
        }
    }

    // Ensure upload directory exists
    std::fs::create_dir_all(upload_dir)?;

    let storage_path = get_storage_path(&final_filename, upload_dir);
    std::fs::write(&storage_path, data)?;

    let now = crate::utils::now_local();

    // 不再存储 storage_path 和 url，运行时动态计算
    let file_model = files::ActiveModel {
        user_id: Set(user_id),
        filename: Set(final_filename),
        original_name: Set(original_name.to_string()),
        mime_type: Set(content_type.to_string()),
        size: Set(data.len() as i64),
        storage_path: Set(None),
        url: Set(None),
        md5_hash: Set(Some(md5_hash)),
        deleted_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let result = file_model.insert(db).await?;
    Ok(result)
}

pub async fn list_files(
    db: &DatabaseConnection,
    page: u64,
    page_size: u64,
) -> Result<(Vec<files::Model>, u64), AppError> {
    let condition = Condition::all().add(files::Column::DeletedAt.is_null());

    let total = files::Entity::find()
        .filter(condition.clone())
        .count(db)
        .await?;

    let items = files::Entity::find()
        .filter(condition)
        .order_by_desc(files::Column::CreatedAt)
        .offset(Some((page - 1) * page_size))
        .limit(Some(page_size))
        .all(db)
        .await?;

    Ok((items, total))
}

pub async fn list_files_by_user(
    db: &DatabaseConnection,
    user_id: i32,
    page: u64,
    page_size: u64,
) -> Result<(Vec<files::Model>, u64), AppError> {
    let condition = Condition::all()
        .add(files::Column::DeletedAt.is_null())
        .add(files::Column::UserId.eq(user_id));

    let total = files::Entity::find()
        .filter(condition.clone())
        .count(db)
        .await?;

    let items = files::Entity::find()
        .filter(condition)
        .order_by_desc(files::Column::CreatedAt)
        .offset(Some((page - 1) * page_size))
        .limit(Some(page_size))
        .all(db)
        .await?;

    Ok((items, total))
}

pub async fn delete_file(
    db: &DatabaseConnection,
    id: i32,
    upload_dir: &str,
) -> Result<(), AppError> {
    let file = files::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(AppError::NotFound("文件不存在".to_string()))?;

    // Remove physical file from disk (ignore errors if already gone)
    let storage_path = get_storage_path(&file.filename, upload_dir);
    let path = std::path::Path::new(&storage_path);
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }

    // Hard-delete DB record
    files::Entity::delete_by_id(id)
        .exec(db)
        .await?;

    Ok(())
}

pub async fn batch_delete_files(
    db: &DatabaseConnection,
    ids: &[i32],
    upload_dir: &str,
) -> Result<usize, AppError> {
    let mut deleted = 0;
    for &id in ids {
        let file = files::Entity::find_by_id(id)
            .one(db)
            .await?;
        if let Some(file) = file {
            let storage_path = get_storage_path(&file.filename, upload_dir);
            let _ = std::fs::remove_file(std::path::Path::new(&storage_path));
            files::Entity::delete_by_id(id).exec(db).await?;
            deleted += 1;
        }
    }
    Ok(deleted)
}

/// 检查图片是否被任何地方引用
/// 检查范围：文章内容、文章封面、分类图片、网站Logo
pub async fn is_file_referenced(db: &DatabaseConnection, filename: &str) -> Result<bool, AppError> {
    // 1. 检查是否被文章内容引用
    let content_ref_count = posts::Entity::find()
        .filter(
            Condition::all()
                .add(posts::Column::Content.like(format!("%{}%", filename)))
                .add(posts::Column::DeletedAt.is_null()),
        )
        .count(db)
        .await?;
    
    if content_ref_count > 0 {
        return Ok(true);
    }

    // 2. 检查是否被文章封面引用
    let cover_ref_count = posts::Entity::find()
        .filter(
            Condition::all()
                .add(posts::Column::CoverImage.like(format!("%{}%", filename)))
                .add(posts::Column::DeletedAt.is_null()),
        )
        .count(db)
        .await?;
    
    if cover_ref_count > 0 {
        return Ok(true);
    }

    // 3. 检查是否被分类图片引用（image_url 或 image_filename）
    let category_ref_count = categories::Entity::find()
        .filter(
            Condition::all()
                .add(
                    Condition::any()
                        .add(categories::Column::ImageUrl.like(format!("%{}%", filename)))
                        .add(categories::Column::ImageFilename.like(format!("%{}%", filename)))
                )
                .add(categories::Column::DeletedAt.is_null()),
        )
        .count(db)
        .await?;
    
    if category_ref_count > 0 {
        return Ok(true);
    }

    // 4. 检查是否被网站Logo引用
    if let Some(setting) = settings::Entity::find_by_id("site_logo")
        .one(db)
        .await?
    {
        if setting.value.contains(filename) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// 获取所有未被引用的文件
pub async fn get_unreferenced_files(db: &DatabaseConnection) -> Result<Vec<files::Model>, AppError> {
    let all_files = files::Entity::find()
        .filter(files::Column::DeletedAt.is_null())
        .all(db)
        .await?;

    let mut unreferenced = Vec::new();
    for file in all_files {
        let referenced = is_file_referenced(db, &file.filename).await?;
        if !referenced {
            unreferenced.push(file);
        }
    }

    Ok(unreferenced)
}
