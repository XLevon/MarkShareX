use crate::models::entity::{categories, files, posts, settings};
use crate::utils::AppError;
use md5::{Digest, Md5};
use sea_orm::*;

fn upload_mutation_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

pub struct UploadMutationSession {
    _guard: tokio::sync::MutexGuard<'static, ()>,
}

pub async fn begin_upload_mutation() -> UploadMutationSession {
    UploadMutationSession {
        _guard: upload_mutation_lock().lock().await,
    }
}

/// Validate that a filename is exactly one safe path component.
fn validate_filename_component(name: &str) -> Result<(), AppError> {
    use std::path::{Component, Path};

    if name.trim().is_empty()
        || name == "."
        || name == ".."
        || name.contains(['/', '\\', ':'])
        || name.chars().any(char::is_control)
        || Path::new(name).is_absolute()
    {
        return Err(AppError::BadRequest("文件名不安全".to_string()));
    }

    let mut components = Path::new(name).components();
    if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
        return Err(AppError::BadRequest("文件名不安全".to_string()));
    }

    let stem = Path::new(name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if stem.is_empty() || stem == "." || stem == ".." {
        return Err(AppError::BadRequest("文件名不安全".to_string()));
    }

    Ok(())
}

/// Sanitize a validated filename for URL safety.
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

pub(crate) fn prepare_upload_filename(
    original_name: &str,
    rename_to: Option<&str>,
) -> Result<String, AppError> {
    validate_filename_component(original_name)?;
    let extension = std::path::Path::new(original_name)
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::BadRequest("文件缺少有效扩展名".to_string()))?;

    let raw_filename = if let Some(rename) = rename_to {
        validate_filename_component(rename)?;
        if std::path::Path::new(rename).extension().is_some() {
            rename.to_string()
        } else {
            format!("{rename}.{extension}")
        }
    } else {
        original_name.to_string()
    };

    let filename = sanitize_filename(&raw_filename);
    validate_filename_component(&filename)?;
    Ok(filename)
}

pub fn infer_mime_by_extension(filename: &str) -> Option<&'static str> {
    let ext = std::path::Path::new(filename)
        .extension()?
        .to_str()?
        .to_ascii_lowercase();
    Some(match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "md" | "markdown" => "text/markdown",
        "txt" => "text/plain",
        "zip" => "application/zip",
        _ => return None,
    })
}

fn detect_content_mime(data: &[u8]) -> Option<&'static str> {
    if data.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("image/png");
    }
    if data.starts_with(b"\xff\xd8\xff") {
        return Some("image/jpeg");
    }
    if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
        return Some("image/gif");
    }
    if data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    if data.starts_with(b"%PDF-") {
        return Some("application/pdf");
    }
    if data.starts_with(b"PK\x03\x04")
        || data.starts_with(b"PK\x05\x06")
        || data.starts_with(b"PK\x07\x08")
    {
        return Some("application/zip");
    }

    let text = std::str::from_utf8(data).ok()?;
    if text
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return None;
    }
    let lower = text
        .trim_start_matches('\u{feff}')
        .trim_start()
        .to_ascii_lowercase();
    if (lower.starts_with("<svg") || lower.starts_with("<?xml"))
        && lower
            .chars()
            .take(1024)
            .collect::<String>()
            .contains("<svg")
    {
        return Some("image/svg+xml");
    }
    Some("text/plain")
}

fn validate_image_structure(data: &[u8], format: image::ImageFormat) -> image::ImageResult<()> {
    let mut reader = image::ImageReader::with_format(std::io::Cursor::new(data), format);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    reader.decode().map(|_| ())
}

fn validate_file_structure(detected: &str, data: &[u8]) -> Result<(), AppError> {
    let invalid = || AppError::BadRequest(format!("文件内容不是有效的 {detected}"));
    match detected {
        "image/png" => validate_image_structure(data, image::ImageFormat::Png)
            .map(|_| ())
            .map_err(|_| invalid()),
        "image/jpeg" => validate_image_structure(data, image::ImageFormat::Jpeg)
            .map(|_| ())
            .map_err(|_| invalid()),
        "image/gif" => validate_image_structure(data, image::ImageFormat::Gif)
            .map(|_| ())
            .map_err(|_| invalid()),
        "image/webp" => validate_image_structure(data, image::ImageFormat::WebP)
            .map(|_| ())
            .map_err(|_| invalid()),
        "application/pdf" => lopdf::Document::load_mem(data)
            .map(|_| ())
            .map_err(|_| invalid()),
        "application/zip" => zip::ZipArchive::new(std::io::Cursor::new(data))
            .map(|_| ())
            .map_err(|_| invalid()),
        "text/plain" | "image/svg+xml" => Ok(()),
        _ => Err(AppError::BadRequest(format!(
            "该文件类型尚未启用安全结构验证: {detected}"
        ))),
    }
}

pub fn validate_upload_content(
    filename: &str,
    declared_mime: &str,
    data: &[u8],
    allowed_types: &[String],
) -> Result<String, AppError> {
    validate_filename_component(filename)?;
    let expected = infer_mime_by_extension(filename)
        .ok_or_else(|| AppError::BadRequest("无法识别文件扩展名".to_string()))?;
    let detected = detect_content_mime(data)
        .ok_or_else(|| AppError::BadRequest("无法识别文件真实类型".to_string()))?;
    validate_file_structure(detected, data)?;
    if detected == "image/svg+xml" {
        return Err(AppError::BadRequest(
            "安全策略禁止上传 SVG 主动内容；请转换为 PNG、JPEG 或 WebP".to_string(),
        ));
    }

    let detected = match (expected, detected) {
        ("text/markdown", "text/plain") => "text/markdown",
        _ => detected,
    };
    if expected != detected {
        return Err(AppError::BadRequest(format!(
            "文件扩展名与真实类型不一致: {expected} != {detected}"
        )));
    }

    let declared = declared_mime
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    let declared = if declared == "image/svg" {
        "image/svg+xml".to_string()
    } else {
        declared
    };
    if declared != detected {
        return Err(AppError::BadRequest(format!(
            "声明类型与真实类型不一致: {declared} != {detected}"
        )));
    }
    if !allowed_types.iter().any(|allowed| allowed == detected) {
        return Err(AppError::BadRequest(format!(
            "不支持的文件类型: {detected}"
        )));
    }

    Ok(detected.to_string())
}

pub(crate) fn preflight_upload(
    original_name: &str,
    declared_content_type: &str,
    data: &[u8],
    allowed_types: &[String],
    max_file_size: u64,
    rename_to: Option<&str>,
) -> Result<(String, String), AppError> {
    if data.len() as u64 > max_file_size {
        return Err(AppError::BadRequest(format!(
            "文件过大: {} bytes (最大 {} bytes)",
            data.len(),
            max_file_size
        )));
    }
    let filename = prepare_upload_filename(original_name, rename_to)?;
    let detected = validate_upload_content(&filename, declared_content_type, data, allowed_types)?;
    Ok((filename, detected))
}

/// Calculate MD5 hash of data
pub fn calculate_md5(data: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.input(data);
    let result = hasher.result();
    format!("{:x}", result)
}

/// Get a storage path only for a validated single-component filename.
pub fn get_storage_path(filename: &str, upload_dir: &str) -> Result<std::path::PathBuf, AppError> {
    validate_filename_component(filename)?;
    Ok(std::path::Path::new(upload_dir).join(filename))
}

pub fn read_storage_file(filename: &str, upload_dir: &str) -> Result<Vec<u8>, AppError> {
    use std::io::Read;

    let storage_path = get_storage_path(filename, upload_dir)?;
    let canonical_root = std::fs::canonicalize(upload_dir)?;
    let metadata = std::fs::symlink_metadata(&storage_path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(AppError::BadRequest(
            "存储文件不是安全的普通文件".to_string(),
        ));
    }
    let canonical_path = std::fs::canonicalize(&storage_path)?;
    if !canonical_path.starts_with(&canonical_root) {
        return Err(AppError::BadRequest("存储文件超出上传目录".to_string()));
    }

    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let mut file = options.open(&storage_path)?;
    if !file.metadata()?.is_file() {
        return Err(AppError::BadRequest(
            "存储文件不是安全的普通文件".to_string(),
        ));
    }
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}

fn storage_transaction_dir(upload_dir: &str) -> Result<std::path::PathBuf, AppError> {
    let path = std::path::Path::new(upload_dir).join(".transactions");
    match std::fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err(AppError::BadRequest("上传事务目录不是安全目录".to_string()));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir(&path)?;
            let metadata = std::fs::symlink_metadata(&path)?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(AppError::BadRequest("上传事务目录不是安全目录".to_string()));
            }
        }
        Err(error) => return Err(error.into()),
    }
    Ok(path)
}

fn restore_storage_backups(
    backups: &[(std::path::PathBuf, std::path::PathBuf)],
) -> Result<(), AppError> {
    let mut failures = Vec::new();
    for (backup, original) in backups.iter().rev() {
        if let Err(error) = std::fs::rename(backup, original) {
            failures.push(format!(
                "{} -> {}: {error}",
                backup.display(),
                original.display()
            ));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(AppError::Internal(anyhow::anyhow!(
            "文件恢复失败: {}",
            failures.join("; ")
        )))
    }
}

fn preserve_primary_with_compensation(
    primary: &dyn std::fmt::Display,
    compensation: Result<(), AppError>,
) -> Result<(), AppError> {
    compensation.map_err(|compensation_error| {
        AppError::Internal(anyhow::anyhow!(
            "主操作失败: {primary}; 补偿失败: {compensation_error}"
        ))
    })
}

fn log_cleanup_failure(path: &std::path::Path, error: std::io::Error) {
    tracing::error!(path = %path.display(), error = %error, "清理文件事务残留失败");
}

fn remove_owned_transaction_file(path: &std::path::Path) {
    match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => log_cleanup_failure(path, error),
    }
}

fn quarantine_uncommitted_storage_file(
    path: &std::path::Path,
    upload_dir: &str,
    primary_error: &dyn std::fmt::Display,
) -> Result<(), AppError> {
    match std::fs::remove_file(path) {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(remove_error) => {
            let transaction_dir = storage_transaction_dir(upload_dir).map_err(|directory_error| {
                AppError::Internal(anyhow::anyhow!(
                    "{primary_error}; 删除未提交文件失败: {remove_error}; 创建安全隔离目录失败: {directory_error}"
                ))
            })?;
            let quarantined =
                transaction_dir.join(format!("uncommitted-{}.failed", uuid::Uuid::new_v4()));
            std::fs::rename(path, &quarantined).map_err(|rename_error| {
                AppError::Internal(anyhow::anyhow!(
                    "{primary_error}; 删除未提交文件失败: {remove_error}; 隔离未提交文件失败: {rename_error}"
                ))
            })?;
            remove_owned_transaction_file(&quarantined);
            Ok(())
        }
    }
}

fn reserve_unique_storage_file(
    filename: &str,
    upload_dir: &str,
    occupied_database_names: &std::collections::HashSet<String>,
) -> Result<(String, std::path::PathBuf, std::fs::File), AppError> {
    use std::io::ErrorKind;

    let path = std::path::Path::new(filename);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| AppError::BadRequest("文件名不安全".to_string()))?;
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .ok_or_else(|| AppError::BadRequest("文件缺少有效扩展名".to_string()))?;

    for counter in 0u64.. {
        let candidate = if counter == 0 {
            filename.to_string()
        } else {
            format!("{stem}-{counter}.{extension}")
        };
        if occupied_database_names.contains(&candidate) {
            continue;
        }
        let storage_path = get_storage_path(&candidate, upload_dir)?;
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&storage_path)
        {
            Ok(file) => return Ok((candidate, storage_path, file)),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    unreachable!("u64 filename suffix space exhausted")
}

/// Get file URL from a validated filename. Invalid historical rows never become traversal URLs.
pub fn get_file_url(filename: &str) -> String {
    if validate_filename_component(filename).is_err() {
        return "/uploads/invalid-file".to_string();
    }
    format!("/uploads/{filename}")
}

pub struct UploadOutcome {
    pub file: files::Model,
    pub created: bool,
}

pub async fn upload_file(
    db: &DatabaseConnection,
    user_id: i32,
    original_name: &str,
    declared_content_type: &str,
    data: &[u8],
    upload_dir: &str,
    allowed_types: &[String],
    max_file_size: u64,
    rename_to: Option<&str>,
    overwrite: bool,
) -> Result<files::Model, AppError> {
    upload_file_with_outcome(
        db,
        user_id,
        original_name,
        declared_content_type,
        data,
        upload_dir,
        allowed_types,
        max_file_size,
        rename_to,
        overwrite,
    )
    .await
    .map(|outcome| outcome.file)
}

pub async fn upload_file_with_outcome(
    db: &DatabaseConnection,
    user_id: i32,
    original_name: &str,
    declared_content_type: &str,
    data: &[u8],
    upload_dir: &str,
    allowed_types: &[String],
    max_file_size: u64,
    rename_to: Option<&str>,
    overwrite: bool,
) -> Result<UploadOutcome, AppError> {
    let _mutation_guard = upload_mutation_lock().lock().await;
    upload_file_with_outcome_locked(
        db,
        user_id,
        original_name,
        declared_content_type,
        data,
        upload_dir,
        allowed_types,
        max_file_size,
        rename_to,
        overwrite,
    )
    .await
}

pub async fn upload_file_in_session(
    _session: &UploadMutationSession,
    db: &DatabaseConnection,
    user_id: i32,
    original_name: &str,
    declared_content_type: &str,
    data: &[u8],
    upload_dir: &str,
    allowed_types: &[String],
    max_file_size: u64,
    rename_to: Option<&str>,
    overwrite: bool,
) -> Result<UploadOutcome, AppError> {
    upload_file_with_outcome_locked(
        db,
        user_id,
        original_name,
        declared_content_type,
        data,
        upload_dir,
        allowed_types,
        max_file_size,
        rename_to,
        overwrite,
    )
    .await
}

async fn upload_file_with_outcome_locked(
    db: &DatabaseConnection,
    user_id: i32,
    original_name: &str,
    declared_content_type: &str,
    data: &[u8],
    upload_dir: &str,
    allowed_types: &[String],
    max_file_size: u64,
    rename_to: Option<&str>,
    overwrite: bool,
) -> Result<UploadOutcome, AppError> {
    let (filename, detected_content_type) = preflight_upload(
        original_name,
        declared_content_type,
        data,
        allowed_types,
        max_file_size,
        rename_to,
    )?;

    // Calculate MD5 only after preflight; callers serialize mutations with the shared lock.
    let md5_hash = calculate_md5(data);

    if let Some(existing_file) = files::Entity::find()
        .filter(
            Condition::all()
                .add(files::Column::Md5Hash.eq(&md5_hash))
                .add(files::Column::DeletedAt.is_null()),
        )
        .one(db)
        .await?
    {
        return Ok(UploadOutcome {
            file: existing_file,
            created: false,
        });
    }

    std::fs::create_dir_all(upload_dir)?;
    let now = crate::utils::now_local();
    let build_model = |final_filename: String| files::ActiveModel {
        user_id: Set(user_id),
        filename: Set(final_filename),
        original_name: Set(original_name.to_string()),
        mime_type: Set(detected_content_type.clone()),
        size: Set(data.len() as i64),
        storage_path: Set(None),
        url: Set(None),
        md5_hash: Set(Some(md5_hash.clone())),
        deleted_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    if !overwrite {
        use std::io::Write;

        let occupied_database_names = files::Entity::find()
            .filter(files::Column::DeletedAt.is_null())
            .select_only()
            .column(files::Column::Filename)
            .into_tuple::<String>()
            .all(db)
            .await?
            .into_iter()
            .collect::<std::collections::HashSet<_>>();
        let (final_filename, storage_path, mut reserved_file) =
            reserve_unique_storage_file(&filename, upload_dir, &occupied_database_names)?;
        if let Err(error) = reserved_file
            .write_all(data)
            .and_then(|_| reserved_file.sync_all())
        {
            drop(reserved_file);
            quarantine_uncommitted_storage_file(&storage_path, upload_dir, &error)?;
            return Err(error.into());
        }
        drop(reserved_file);

        return match build_model(final_filename).insert(db).await {
            Ok(model) => Ok(UploadOutcome {
                file: model,
                created: true,
            }),
            Err(error) => {
                quarantine_uncommitted_storage_file(&storage_path, upload_dir, &error)?;
                Err(error.into())
            }
        };
    }

    use std::io::Write;
    let storage_path = get_storage_path(&filename, upload_dir)?;
    let transaction_dir = storage_transaction_dir(upload_dir)?;
    let temp_path = transaction_dir.join(format!("upload-{}.tmp", uuid::Uuid::new_v4()));
    let mut temp_file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)?;
    if let Err(error) = temp_file.write_all(data).and_then(|_| temp_file.sync_all()) {
        drop(temp_file);
        remove_owned_transaction_file(&temp_path);
        return Err(error.into());
    }
    drop(temp_file);

    let transaction = match db.begin().await {
        Ok(transaction) => transaction,
        Err(error) => {
            remove_owned_transaction_file(&temp_path);
            return Err(error.into());
        }
    };
    let existing = match files::Entity::find()
        .filter(
            Condition::all()
                .add(files::Column::Filename.eq(&filename))
                .add(files::Column::DeletedAt.is_null()),
        )
        .one(&transaction)
        .await
    {
        Ok(existing) => existing,
        Err(error) => {
            if let Err(rollback_error) = transaction.rollback().await {
                tracing::error!(error = %rollback_error, "文件事务数据库回滚失败");
            }
            remove_owned_transaction_file(&temp_path);
            return Err(error.into());
        }
    };
    if let Some(existing) = existing {
        let mut active: files::ActiveModel = existing.into();
        active.deleted_at = Set(Some(now));
        active.updated_at = Set(now);
        if let Err(error) = active.update(&transaction).await {
            if let Err(rollback_error) = transaction.rollback().await {
                tracing::error!(error = %rollback_error, "文件事务数据库回滚失败");
            }
            remove_owned_transaction_file(&temp_path);
            return Err(error.into());
        }
    }
    let result = match build_model(filename).insert(&transaction).await {
        Ok(model) => model,
        Err(error) => {
            if let Err(rollback_error) = transaction.rollback().await {
                tracing::error!(error = %rollback_error, "文件事务数据库回滚失败");
            }
            remove_owned_transaction_file(&temp_path);
            return Err(error.into());
        }
    };

    let backup_path = transaction_dir.join(format!("upload-{}.backup", uuid::Uuid::new_v4()));
    let had_existing_target = match std::fs::symlink_metadata(&storage_path) {
        Ok(metadata) if metadata.is_dir() => {
            if let Err(rollback_error) = transaction.rollback().await {
                tracing::error!(error = %rollback_error, "文件事务数据库回滚失败");
            }
            remove_owned_transaction_file(&temp_path);
            return Err(AppError::BadRequest("目标文件是目录".to_string()));
        }
        Ok(_) => {
            if let Err(error) = std::fs::rename(&storage_path, &backup_path) {
                if let Err(rollback_error) = transaction.rollback().await {
                    tracing::error!(error = %rollback_error, "文件事务数据库回滚失败");
                }
                remove_owned_transaction_file(&temp_path);
                return Err(error.into());
            }
            true
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => {
            if let Err(rollback_error) = transaction.rollback().await {
                tracing::error!(error = %rollback_error, "文件事务数据库回滚失败");
            }
            remove_owned_transaction_file(&temp_path);
            return Err(error.into());
        }
    };

    if let Err(error) = std::fs::rename(&temp_path, &storage_path) {
        let restore_result = if had_existing_target {
            restore_storage_backups(&[(backup_path.clone(), storage_path.clone())])
        } else {
            Ok(())
        };
        if let Err(rollback_error) = transaction.rollback().await {
            tracing::error!(error = %rollback_error, "覆盖上传数据库回滚失败");
        }
        remove_owned_transaction_file(&temp_path);
        preserve_primary_with_compensation(&error, restore_result)?;
        return Err(error.into());
    }

    if let Err(error) = transaction.commit().await {
        let failed_path = transaction_dir.join(format!("upload-{}.failed", uuid::Uuid::new_v4()));
        if let Err(displace_error) = std::fs::rename(&storage_path, &failed_path) {
            return Err(AppError::Internal(anyhow::anyhow!(
                "数据库提交失败且无法隔离新文件: {error}; {displace_error}"
            )));
        }
        if had_existing_target {
            let restore_result =
                restore_storage_backups(&[(backup_path.clone(), storage_path.clone())]);
            preserve_primary_with_compensation(&error, restore_result)?;
        }
        if let Err(cleanup_error) = std::fs::remove_file(&failed_path) {
            log_cleanup_failure(&failed_path, cleanup_error);
        }
        return Err(error.into());
    }
    if had_existing_target {
        if let Err(error) = std::fs::remove_file(&backup_path) {
            log_cleanup_failure(&backup_path, error);
        }
    }
    Ok(UploadOutcome {
        file: result,
        created: true,
    })
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
    if batch_delete_files(db, &[id], upload_dir).await? == 0 {
        return Err(AppError::NotFound("文件不存在".to_string()));
    }
    Ok(())
}

pub async fn batch_delete_files(
    db: &DatabaseConnection,
    ids: &[i32],
    upload_dir: &str,
) -> Result<usize, AppError> {
    let _mutation_guard = upload_mutation_lock().lock().await;
    batch_delete_files_locked(db, ids, upload_dir).await
}

pub async fn batch_delete_files_in_session(
    _session: &UploadMutationSession,
    db: &DatabaseConnection,
    ids: &[i32],
    upload_dir: &str,
) -> Result<usize, AppError> {
    batch_delete_files_locked(db, ids, upload_dir).await
}

async fn batch_delete_files_locked(
    db: &DatabaseConnection,
    ids: &[i32],
    upload_dir: &str,
) -> Result<usize, AppError> {
    let records = files::Entity::find()
        .filter(files::Column::Id.is_in(ids.iter().copied()))
        .all(db)
        .await?;
    if records.is_empty() {
        return Ok(0);
    }

    // Validate every historical name before mutating either database or filesystem.
    let mut paths = Vec::with_capacity(records.len());
    for file in &records {
        paths.push(get_storage_path(&file.filename, upload_dir)?);
    }
    let transaction_dir = storage_transaction_dir(upload_dir)?;

    let transaction = db.begin().await?;
    files::Entity::delete_many()
        .filter(files::Column::Id.is_in(records.iter().map(|file| file.id)))
        .exec(&transaction)
        .await?;

    let mut backups: Vec<(std::path::PathBuf, std::path::PathBuf)> = Vec::new();
    for path in &paths {
        match std::fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_dir() => {
                let primary = "目标文件是目录";
                let restore_result = restore_storage_backups(&backups);
                if let Err(rollback_error) = transaction.rollback().await {
                    tracing::error!(error = %rollback_error, "文件事务数据库回滚失败");
                }
                preserve_primary_with_compensation(&primary, restore_result)?;
                return Err(AppError::BadRequest(primary.to_string()));
            }
            Ok(_) => {
                let backup =
                    transaction_dir.join(format!("delete-{}.backup", uuid::Uuid::new_v4()));
                if let Err(error) = std::fs::rename(path, &backup) {
                    let restore_result = restore_storage_backups(&backups);
                    if let Err(rollback_error) = transaction.rollback().await {
                        tracing::error!(error = %rollback_error, "文件事务数据库回滚失败");
                    }
                    preserve_primary_with_compensation(&error, restore_result)?;
                    return Err(error.into());
                }
                backups.push((backup, path.clone()));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                let restore_result = restore_storage_backups(&backups);
                if let Err(rollback_error) = transaction.rollback().await {
                    tracing::error!(error = %rollback_error, "文件事务数据库回滚失败");
                }
                preserve_primary_with_compensation(&error, restore_result)?;
                return Err(error.into());
            }
        }
    }

    if let Err(error) = transaction.commit().await {
        let restore_result = restore_storage_backups(&backups);
        preserve_primary_with_compensation(&error, restore_result)?;
        return Err(error.into());
    }
    for (backup, _) in backups {
        if let Err(error) = std::fs::remove_file(&backup) {
            log_cleanup_failure(&backup, error);
        }
    }
    Ok(records.len())
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
                        .add(categories::Column::ImageFilename.like(format!("%{}%", filename))),
                )
                .add(categories::Column::DeletedAt.is_null()),
        )
        .count(db)
        .await?;

    if category_ref_count > 0 {
        return Ok(true);
    }

    // 4. 检查是否被网站Logo引用
    if let Some(setting) = settings::Entity::find_by_id("site_logo").one(db).await? {
        if setting.value.contains(filename) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// 获取所有未被引用的文件
pub async fn get_unreferenced_files(
    db: &DatabaseConnection,
) -> Result<Vec<files::Model>, AppError> {
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

#[cfg(test)]
mod security_tests {
    use super::*;
    use lopdf::dictionary;
    use std::io::Write;

    fn valid_png() -> Vec<u8> {
        png_with_dimensions(1, 1)
    }

    fn png_with_dimensions(width: u32, height: u32) -> Vec<u8> {
        let image = image::RgbaImage::from_pixel(width, height, image::Rgba([1, 2, 3, 255]));
        let mut cursor = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut cursor, image::ImageFormat::Png)
            .unwrap();
        cursor.into_inner()
    }

    fn valid_pdf() -> Vec<u8> {
        let mut document = lopdf::Document::with_version("1.5");
        let pages = document.add_object(dictionary! {
            "Type" => "Pages",
            "Kids" => lopdf::Object::Array(Vec::new()),
            "Count" => 0,
        });
        let catalog = document.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages,
        });
        document.trailer.set("Root", catalog);
        let mut bytes = Vec::new();
        document.save_to(&mut bytes).unwrap();
        bytes
    }

    fn valid_zip() -> Vec<u8> {
        let cursor = std::io::Cursor::new(Vec::new());
        let mut writer = zip::ZipWriter::new(cursor);
        writer
            .start_file("entry.txt", zip::write::FileOptions::default())
            .unwrap();
        writer.write_all(b"safe").unwrap();
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn upload_filename_must_be_one_safe_path_component() {
        for name in [
            "../outside.png",
            "nested/../../outside.png",
            "/tmp/outside.png",
            r"..\outside.png",
            ".",
            "..",
            "",
            "bad\0name.png",
        ] {
            assert!(
                prepare_upload_filename(name, None).is_err(),
                "accepted {name:?}"
            );
        }

        for rename in [
            "../outside",
            "nested/outside",
            "/tmp/outside.png",
            r"..\outside.png",
            ".",
            "..",
            "",
        ] {
            assert!(
                prepare_upload_filename("safe.png", Some(rename)).is_err(),
                "accepted rename {rename:?}"
            );
        }
    }

    #[test]
    fn upload_filename_preserves_safe_unicode_and_inherits_extension() {
        assert_eq!(
            prepare_upload_filename("你好 图片.png", None).unwrap(),
            "你好-图片.png"
        );
        assert_eq!(
            prepare_upload_filename("photo.png", Some("renamed file")).unwrap(),
            "renamed-file.png"
        );
        assert_eq!(
            prepare_upload_filename("photo.png", Some("renamed.webp")).unwrap(),
            "renamed.webp"
        );
    }

    #[test]
    fn storage_path_rejects_unsafe_historical_filenames() {
        let root = tempfile::tempdir().unwrap();
        assert!(get_storage_path("safe.png", root.path().to_str().unwrap()).is_ok());
        assert_eq!(get_file_url("safe.png"), "/uploads/safe.png");
        for name in ["../victim", "/tmp/victim", r"..\victim", ".", "..", ""] {
            assert!(get_storage_path(name, root.path().to_str().unwrap()).is_err());
            assert_eq!(get_file_url(name), "/uploads/invalid-file");
        }
    }

    #[test]
    fn upload_content_requires_matching_signature_extension_and_declared_mime() {
        let png = valid_png();
        let allowed = vec!["image/png".to_string(), "image/svg+xml".to_string()];

        assert_eq!(
            validate_upload_content("safe.png", "image/png", &png, &allowed).unwrap(),
            "image/png"
        );
        assert!(validate_upload_content(
            "fake.png",
            "image/png",
            b"plain text pretending to be an image",
            &allowed
        )
        .is_err());
        assert!(validate_upload_content("wrong.svg", "image/svg+xml", &png, &allowed).is_err());
        assert!(validate_upload_content("safe.png", "text/plain", &png, &allowed).is_err());
    }

    #[test]
    fn upload_content_rejects_detected_types_outside_the_allowlist() {
        let png = valid_png();
        let allowed = vec!["image/jpeg".to_string()];
        assert!(validate_upload_content("safe.png", "image/png", &png, &allowed).is_err());
    }

    #[test]
    fn active_svg_uploads_are_fail_closed_even_when_configured() {
        let allowed = vec!["image/svg+xml".to_string()];
        for svg in [
            br#"<svg xmlns="http://www.w3.org/2000/svg"><circle r="10"/></svg>"#.as_slice(),
            br#"<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>"#
                .as_slice(),
        ] {
            assert!(validate_upload_content("image.svg", "image/svg+xml", svg, &allowed).is_err());
        }
    }

    #[test]
    fn non_overwrite_storage_names_are_reserved_atomically() {
        let root = tempfile::tempdir().unwrap();
        let occupied = std::collections::HashSet::new();
        let (first_name, first_path, _first_file) =
            reserve_unique_storage_file("same.png", root.path().to_str().unwrap(), &occupied)
                .unwrap();
        let (second_name, second_path, _second_file) =
            reserve_unique_storage_file("same.png", root.path().to_str().unwrap(), &occupied)
                .unwrap();
        assert_eq!(first_name, "same.png");
        assert_eq!(second_name, "same-1.png");
        assert_ne!(first_path, second_path);
        assert!(first_path.exists());
        assert!(second_path.exists());
    }

    #[test]
    fn truncated_magic_prefixes_are_not_valid_files() {
        assert!(validate_upload_content(
            "short.png",
            "image/png",
            b"\x89PNG\r\n\x1a\n",
            &["image/png".to_string()],
        )
        .is_err());
        assert!(validate_upload_content(
            "short.pdf",
            "application/pdf",
            b"%PDF-1.7\n",
            &["application/pdf".to_string()],
        )
        .is_err());
        assert!(validate_upload_content(
            "short.zip",
            "application/zip",
            b"PK\x03\x04\0\0\0\0",
            &["application/zip".to_string()],
        )
        .is_err());
    }

    #[test]
    fn oversized_image_dimensions_are_rejected_before_unbounded_decode() {
        let allowed = vec!["image/png".to_string()];
        let oversized = png_with_dimensions(8193, 1);
        assert!(validate_upload_content("wide.png", "image/png", &oversized, &allowed).is_err());
    }

    #[test]
    fn valid_document_and_archive_signatures_are_accepted() {
        let allowed = vec!["application/pdf".to_string(), "application/zip".to_string()];
        let pdf = valid_pdf();
        let zip = valid_zip();
        assert_eq!(
            validate_upload_content("doc.pdf", "application/pdf", &pdf, &allowed).unwrap(),
            "application/pdf"
        );
        assert_eq!(
            validate_upload_content("archive.zip", "application/zip", &zip, &allowed,).unwrap(),
            "application/zip"
        );
    }
}
