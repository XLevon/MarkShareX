mod common;

use axum_test::multipart::{MultipartForm, Part};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use lopdf::dictionary;
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Statement,
};
use serde_json::{json, Value};
use std::io::{Read, Write};

use common::TestApp;
use marksharex::models::entity::{categories, files, post_tags, posts, tags};

fn regular_file_count(path: &std::path::Path) -> std::io::Result<usize> {
    let mut count = 0;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            count += regular_file_count(&entry.path())?;
        } else if entry.file_type()?.is_file() {
            count += 1;
        }
    }
    Ok(count)
}

fn regular_files(path: &std::path::Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            files.extend(regular_files(&entry.path())?);
        } else if entry.file_type()?.is_file() {
            files.push(entry.path());
        }
    }
    Ok(files)
}

fn valid_png(red: u8) -> Vec<u8> {
    let image = image::RgbaImage::from_pixel(1, 1, image::Rgba([red, 0, 0, 255]));
    let mut cursor = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .expect("test PNG must encode");
    cursor.into_inner()
}

#[tokio::test]
async fn published_import_index_failure_rolls_back_database_relations_and_file_bytes(
) -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let author = app
        .create_user("import-index-failure-author", "author")
        .await?;
    let encoded = STANDARD.encode(valid_png(231));
    app.state.search_engine.poison_writer_for_test();

    let response = app
        .server
        .post("/api/v1/import/posts")
        .authorization_bearer(&author.token)
        .json(&json!({
            "items": [{
                "filename": "index-failure.md",
                "content": "---\ntitle: Index failure import\ncategory: index-failure-category\ntags:\n  - index-failure-tag\nstatus: published\n---\n![x](index-failure.png)",
                "images": [{
                    "name": "index-failure.png",
                    "data": format!("data:image/png;base64,{encoded}")
                }]
            }]
        }))
        .await;
    response.assert_status_ok();
    let body = response.json::<Value>();
    assert_eq!(body["data"]["imported_count"], 0);
    assert_eq!(body["data"]["skipped_count"], 1);

    assert_eq!(
        posts::Entity::find()
            .filter(posts::Column::Title.eq("Index failure import"))
            .count(&app.db)
            .await?,
        0
    );
    assert_eq!(
        categories::Entity::find()
            .filter(categories::Column::Name.eq("index-failure-category"))
            .count(&app.db)
            .await?,
        0
    );
    assert_eq!(
        tags::Entity::find()
            .filter(tags::Column::Name.eq("index-failure-tag"))
            .count(&app.db)
            .await?,
        0
    );
    assert_eq!(post_tags::Entity::find().count(&app.db).await?, 0);
    assert_eq!(files::Entity::find().count(&app.db).await?, 0);
    assert_eq!(
        regular_file_count(std::path::Path::new(&app.state.config.storage.upload_dir))?,
        0
    );
    Ok(())
}

#[tokio::test]
async fn compensation_failure_reports_persisted_post_and_preserves_relations_and_exact_file_bytes(
) -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let author = app
        .create_user("import-compensation-failure-author", "author")
        .await?;
    app.db
        .execute_unprepared(
            "CREATE TRIGGER fail_import_post_compensation
             BEFORE DELETE ON posts
             BEGIN
                 SELECT RAISE(FAIL, 'forced import compensation failure');
             END;",
        )
        .await?;
    let png = valid_png(177);
    let encoded = STANDARD.encode(&png);
    app.state.search_engine.poison_writer_for_test();

    let response = app
        .server
        .post("/api/v1/import/posts")
        .authorization_bearer(&author.token)
        .json(&json!({
            "items": [{
                "filename": "compensation-failure.md",
                "content": "---\ntitle: Persisted after compensation failure\ncategory: persisted-category\ntags:\n  - persisted-tag\nstatus: published\n---\n![x](persisted.png)",
                "images": [{
                    "name": "persisted.png",
                    "data": format!("data:image/png;base64,{encoded}")
                }]
            }]
        }))
        .await;
    response.assert_status_ok();
    let body = response.json::<Value>();
    assert_eq!(body["data"]["success"], false);
    assert_eq!(body["data"]["imported_count"], 1);
    assert_eq!(body["data"]["skipped_count"], 0);
    assert_eq!(
        body["data"]["persisted_with_errors"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(body["data"]["errors"][0]
        .as_str()
        .unwrap()
        .contains("数据库补偿也失败"));

    let post = posts::Entity::find()
        .filter(posts::Column::Title.eq("Persisted after compensation failure"))
        .one(&app.db)
        .await?
        .expect("committed post must remain when compensation rolls back");
    assert_eq!(body["data"]["persisted_with_errors"][0], post.id);
    assert_eq!(
        categories::Entity::find()
            .filter(categories::Column::Name.eq("persisted-category"))
            .count(&app.db)
            .await?,
        1
    );
    assert_eq!(
        tags::Entity::find()
            .filter(tags::Column::Name.eq("persisted-tag"))
            .count(&app.db)
            .await?,
        1
    );
    assert_eq!(
        post_tags::Entity::find()
            .filter(post_tags::Column::PostId.eq(post.id))
            .count(&app.db)
            .await?,
        1
    );
    assert_eq!(files::Entity::find().count(&app.db).await?, 1);
    let paths = regular_files(std::path::Path::new(&app.state.config.storage.upload_dir))?;
    assert_eq!(paths.len(), 1);
    assert_eq!(std::fs::read(&paths[0])?, png);
    Ok(())
}

fn valid_image(red: u8, format: image::ImageFormat) -> Vec<u8> {
    let image = image::RgbaImage::from_pixel(1, 1, image::Rgba([red, 1, 2, 255]));
    let mut cursor = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut cursor, format)
        .expect("test image must encode");
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
    document.save_to(&mut bytes).expect("test PDF must encode");
    bytes
}

fn valid_zip() -> Vec<u8> {
    let cursor = std::io::Cursor::new(Vec::new());
    let mut writer = zip::ZipWriter::new(cursor);
    writer
        .start_file("entry.txt", zip::write::FileOptions::default())
        .expect("test ZIP entry must start");
    writer
        .write_all(b"safe archive")
        .expect("test ZIP entry must write");
    writer.finish().expect("test ZIP must finish").into_inner()
}

fn file_form(filename: &str, mime: &str, bytes: &[u8]) -> MultipartForm {
    file_form_with_field("file", filename, mime, bytes)
}

fn batch_file_form(filename: &str, mime: &str, bytes: &[u8]) -> MultipartForm {
    file_form_with_field("files", filename, mime, bytes)
}

#[cfg(target_os = "macos")]
struct ImmutableFileGuard(std::path::PathBuf);

#[cfg(target_os = "macos")]
impl ImmutableFileGuard {
    fn clear(self) -> anyhow::Result<()> {
        let status = std::process::Command::new("chflags")
            .arg("nouchg")
            .arg(&self.0)
            .status()?;
        anyhow::ensure!(status.success(), "chflags nouchg failed: {status}");
        std::mem::forget(self);
        Ok(())
    }
}

#[cfg(target_os = "macos")]
impl Drop for ImmutableFileGuard {
    fn drop(&mut self) {
        let _ = std::process::Command::new("chflags")
            .arg("nouchg")
            .arg(&self.0)
            .status();
    }
}

#[cfg(target_os = "macos")]
fn make_file_immutable(path: &std::path::Path) -> anyhow::Result<ImmutableFileGuard> {
    let status = std::process::Command::new("chflags")
        .arg("uchg")
        .arg(path)
        .status()?;
    anyhow::ensure!(status.success(), "chflags uchg failed: {status}");
    Ok(ImmutableFileGuard(path.to_path_buf()))
}

fn file_form_with_field(field: &str, filename: &str, mime: &str, bytes: &[u8]) -> MultipartForm {
    MultipartForm::new().add_part(
        field,
        Part::bytes(bytes.to_vec())
            .file_name(filename)
            .mime_type(mime),
    )
}

#[tokio::test]
async fn upload_rejects_path_traversal_and_absolute_rename_without_side_effects(
) -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let author = app.create_user("upload-path-admin", "admin").await?;
    let png = valid_png(1);
    let outside = app.temp_root().join("outside.png");
    let absolute = app.temp_root().join("absolute.png");

    for rename in [
        "../outside.png".to_string(),
        absolute.to_string_lossy().into_owned(),
        r"..\outside.png".to_string(),
    ] {
        app.server
            .post("/api/v1/files/upload")
            .authorization_bearer(&author.token)
            .add_query_param("rename", rename)
            .multipart(file_form("safe.png", "image/png", &png))
            .await
            .assert_status_bad_request();
    }

    app.server
        .post("/api/v1/files/upload")
        .authorization_bearer(&author.token)
        .multipart(file_form("../outside.png", "image/png", &png))
        .await
        .assert_status_bad_request();

    assert!(!outside.exists());
    assert!(!absolute.exists());
    assert_eq!(app.file_count().await?, 0);
    assert_eq!(std::fs::read_dir(app.upload_dir())?.count(), 0);
    Ok(())
}

#[tokio::test]
async fn upload_requires_real_content_to_match_extension_and_declared_mime() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let author = app.create_user("upload-mime-admin", "admin").await?;
    let png = valid_png(2);

    app.server
        .post("/api/v1/files/upload")
        .authorization_bearer(&author.token)
        .multipart(file_form(
            "fake.png",
            "image/png",
            b"plain text pretending to be a png",
        ))
        .await
        .assert_status_bad_request();

    app.server
        .post("/api/v1/files/upload")
        .authorization_bearer(&author.token)
        .multipart(file_form("wrong.svg", "image/svg+xml", &png))
        .await
        .assert_status_bad_request();

    let batch_response = app
        .server
        .post("/api/v1/files/batch")
        .authorization_bearer(&author.token)
        .multipart(batch_file_form(
            "batch-fake.png",
            "image/png",
            b"plain text pretending to be a png",
        ))
        .await;
    batch_response.assert_status_ok();
    let batch_body = batch_response.json::<Value>();
    assert_eq!(batch_body["data"][0]["success"], false);
    assert!(batch_body["data"][0]["error"].is_string());

    assert_eq!(app.file_count().await?, 0);
    assert_eq!(std::fs::read_dir(app.upload_dir())?.count(), 0);
    Ok(())
}

#[tokio::test]
async fn every_configured_upload_type_accepts_valid_content_and_rejects_spoofed_content(
) -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let admin = app.create_user("upload-all-types-admin", "admin").await?;
    let fixtures = vec![
        (
            "valid.jpg",
            "image/jpeg",
            valid_image(11, image::ImageFormat::Jpeg),
        ),
        (
            "valid.png",
            "image/png",
            valid_image(12, image::ImageFormat::Png),
        ),
        (
            "valid.gif",
            "image/gif",
            valid_image(13, image::ImageFormat::Gif),
        ),
        (
            "valid.webp",
            "image/webp",
            valid_image(14, image::ImageFormat::WebP),
        ),
        ("valid.pdf", "application/pdf", valid_pdf()),
        ("valid.md", "text/markdown", b"# valid markdown\n".to_vec()),
        ("valid.txt", "text/plain", b"valid plain text\n".to_vec()),
        ("valid.zip", "application/zip", valid_zip()),
    ];
    let fixture_types = fixtures
        .iter()
        .map(|(_, mime, _)| (*mime).to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let configured_types = app
        .allowed_upload_types()
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        fixture_types, configured_types,
        "the fixture matrix must exactly track every configured upload type"
    );

    for (filename, mime, bytes) in &fixtures {
        let response = app
            .server
            .post("/api/v1/files/upload")
            .authorization_bearer(&admin.token)
            .multipart(file_form(filename, mime, bytes))
            .await;
        response.assert_status_ok();
        let body = response.json::<Value>();
        assert_eq!(body["data"]["mime_type"], *mime, "valid {filename}");
        assert_eq!(
            std::fs::read(app.upload_dir().join(filename))?,
            *bytes,
            "stored bytes for {filename}"
        );
    }

    for (filename, mime, _) in &fixtures {
        let spoofed_name = format!("spoofed-{filename}");
        let spoofed = if mime.starts_with("text/") {
            b"invalid\0control".as_slice()
        } else {
            b"plain text pretending to be another format".as_slice()
        };
        app.server
            .post("/api/v1/files/upload")
            .authorization_bearer(&admin.token)
            .multipart(file_form(&spoofed_name, mime, spoofed))
            .await
            .assert_status_bad_request();
        assert!(!app.upload_dir().join(spoofed_name).exists());
    }

    assert_eq!(app.file_count().await?, fixtures.len() as u64);
    Ok(())
}

#[tokio::test]
async fn valid_png_is_stored_only_inside_the_isolated_upload_directory() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let author = app.create_user("upload-valid-admin", "admin").await?;
    let png = valid_png(3);

    let response = app
        .server
        .post("/api/v1/files/upload")
        .authorization_bearer(&author.token)
        .multipart(file_form("safe image.png", "image/png", &png))
        .await;
    response.assert_status_ok();
    let body = response.json::<Value>();
    assert_eq!(body["data"]["filename"], "safe-image.png");
    assert_eq!(body["data"]["mime_type"], "image/png");
    assert!(app.upload_dir().join("safe-image.png").exists());
    assert_eq!(app.file_count().await?, 1);
    assert_eq!(std::fs::read_dir(app.upload_dir())?.count(), 1);

    let public = app.server.get("/uploads/safe-image.png").await;
    public.assert_status_ok();
    assert_eq!(public.as_bytes().as_ref(), png.as_slice());
    assert_eq!(
        public
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .unwrap(),
        "image/png"
    );

    let duplicate = app
        .server
        .post("/api/v1/files/upload")
        .authorization_bearer(&author.token)
        .multipart(file_form("same-content.png", "image/png", &png))
        .await;
    duplicate.assert_status_ok();
    assert_eq!(
        duplicate.json::<Value>()["data"]["filename"],
        "safe-image.png"
    );
    assert_eq!(app.file_count().await?, 1);
    assert_eq!(std::fs::read_dir(app.upload_dir())?.count(), 1);
    Ok(())
}

#[tokio::test]
async fn concurrent_non_overwrite_uploads_never_share_or_replace_a_filename() -> anyhow::Result<()>
{
    let app = TestApp::new().await?;
    let admin = app.create_user("upload-race-admin", "admin").await?;
    let responses = futures::future::join_all((1..=16u8).map(|red| {
        let request = app
            .server
            .post("/api/v1/files/upload")
            .authorization_bearer(&admin.token)
            .multipart(file_form("race.png", "image/png", &valid_png(red)));
        async move { (red, request.await) }
    }))
    .await;

    let mut stored = Vec::new();
    for response in responses {
        let (red, response) = response;
        response.assert_status_ok();
        let filename = response.json::<Value>()["data"]["filename"]
            .as_str()
            .unwrap()
            .to_string();
        stored.push((filename, valid_png(red)));
    }
    let unique_names: std::collections::HashSet<_> =
        stored.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(unique_names.len(), stored.len());
    for (filename, expected) in stored {
        assert_eq!(std::fs::read(app.upload_dir().join(filename))?, expected);
    }
    assert_eq!(app.file_count().await?, 16);
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn overwrite_replaces_a_symlink_without_following_it_outside_uploads() -> anyhow::Result<()> {
    use std::os::unix::fs::symlink;

    let app = TestApp::new().await?;
    let admin = app.create_user("upload-symlink-admin", "admin").await?;
    let png = valid_png(4);
    let outside = app.temp_root().join("outside-sentinel.png");
    std::fs::write(&outside, b"sentinel")?;
    let link = app.upload_dir().join("link.png");
    symlink(&outside, &link)?;

    app.server
        .post("/api/v1/files/upload")
        .authorization_bearer(&admin.token)
        .add_query_param("rename", "link.png")
        .add_query_param("overwrite", true)
        .multipart(file_form("safe.png", "image/png", &png))
        .await
        .assert_status_ok();

    assert_eq!(std::fs::read(&outside)?, b"sentinel");
    assert!(!std::fs::symlink_metadata(&link)?.file_type().is_symlink());
    assert_eq!(std::fs::read(&link)?, png);
    Ok(())
}

#[tokio::test]
async fn overwrite_insert_failure_preserves_the_old_row_and_bytes() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let admin = app.create_user("overwrite-rollback-admin", "admin").await?;
    let original = valid_png(41);
    app.server
        .post("/api/v1/files/upload")
        .authorization_bearer(&admin.token)
        .multipart(file_form("stable.png", "image/png", &original))
        .await
        .assert_status_ok();

    app.db
        .execute(Statement::from_string(
            app.db.get_database_backend(),
            "CREATE TRIGGER reject_file_insert BEFORE INSERT ON files \
             BEGIN SELECT RAISE(FAIL, 'forced insert failure'); END",
        ))
        .await?;

    app.server
        .post("/api/v1/files/upload")
        .authorization_bearer(&admin.token)
        .add_query_param("rename", "stable.png")
        .add_query_param("overwrite", true)
        .multipart(file_form("replacement.png", "image/png", &valid_png(42)))
        .await
        .assert_status_internal_server_error();

    assert_eq!(
        std::fs::read(app.upload_dir().join("stable.png"))?,
        original
    );
    assert_eq!(app.file_count().await?, 1);
    let root_files = std::fs::read_dir(app.upload_dir())?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .map(|kind| kind.is_file())
                .unwrap_or(false)
        })
        .count();
    assert_eq!(root_files, 1);
    assert_eq!(
        std::fs::read_dir(app.upload_dir().join(".transactions"))?.count(),
        0
    );
    Ok(())
}

#[tokio::test]
async fn overwrite_commit_failure_restores_the_exact_old_row_and_bytes() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let admin = app.create_user("overwrite-commit-admin", "admin").await?;
    let original = valid_png(43);
    app.server
        .post("/api/v1/files/upload")
        .authorization_bearer(&admin.token)
        .multipart(file_form("commit-stable.png", "image/png", &original))
        .await
        .assert_status_ok();

    let rows_before = files::Entity::find()
        .order_by_asc(files::Column::Id)
        .all(&app.db)
        .await?;
    app.db
        .execute(Statement::from_string(
            app.db.get_database_backend(),
            "CREATE TABLE forced_commit_parent (id INTEGER PRIMARY KEY); \
             CREATE TABLE forced_commit_child (parent_id INTEGER, \
               FOREIGN KEY(parent_id) REFERENCES forced_commit_parent(id) \
               DEFERRABLE INITIALLY DEFERRED); \
             CREATE TRIGGER force_file_commit_failure AFTER INSERT ON files \
               BEGIN INSERT INTO forced_commit_child(parent_id) VALUES (999); END",
        ))
        .await?;

    app.server
        .post("/api/v1/files/upload")
        .authorization_bearer(&admin.token)
        .add_query_param("rename", "commit-stable.png")
        .add_query_param("overwrite", true)
        .multipart(file_form("replacement.png", "image/png", &valid_png(44)))
        .await
        .assert_status_internal_server_error();

    let rows_after = files::Entity::find()
        .order_by_asc(files::Column::Id)
        .all(&app.db)
        .await?;
    assert_eq!(rows_after, rows_before);
    assert_eq!(
        std::fs::read(app.upload_dir().join("commit-stable.png"))?,
        original
    );
    assert_eq!(
        std::fs::read_dir(app.upload_dir().join(".transactions"))?.count(),
        0
    );
    Ok(())
}

#[tokio::test]
async fn markdown_import_reports_forged_data_url_and_does_not_store_it() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let author = app.create_user("import-security-author", "author").await?;
    let forged = STANDARD.encode(b"plain text pretending to be png");

    let response = app
        .server
        .post("/api/v1/import/posts")
        .authorization_bearer(&author.token)
        .json(&json!({
            "items": [{
                "filename": "imported.md",
                "content": "---\ntitle: Import security\nstatus: draft\n---\n![x](fake.png)",
                "images": [{
                    "name": "fake.png",
                    "data": format!("data:image/png;base64,{forged}")
                }]
            }]
        }))
        .await;
    response.assert_status_ok();
    let body = response.json::<Value>();
    assert_eq!(body["data"]["success"], false);
    assert!(body["data"]["errors"]
        .as_array()
        .is_some_and(|errors| !errors.is_empty()));
    assert_eq!(app.file_count().await?, 0);
    assert_eq!(app.post_count().await?, 0);
    assert_eq!(std::fs::read_dir(app.upload_dir())?.count(), 0);
    Ok(())
}

#[tokio::test]
async fn markdown_import_rolls_back_new_images_when_post_creation_fails() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let author = app.create_user("import-rollback-author", "author").await?;
    let existing = app
        .create_post(&author, "Existing import target", "draft")
        .await?;
    let encoded = STANDARD.encode(valid_png(61));

    let response = app
        .server
        .post("/api/v1/import/posts")
        .authorization_bearer(&author.token)
        .json(&json!({
            "items": [{
                "filename": "duplicate.md",
                "content": format!(
                    "---\ntitle: Duplicate import\nslug: {}\nstatus: draft\n---\n![x](new.png)",
                    existing.slug
                ),
                "images": [{
                    "name": "new.png",
                    "data": format!("data:image/png;base64,{encoded}")
                }]
            }]
        }))
        .await;
    response.assert_status_ok();
    let body = response.json::<Value>();
    assert_eq!(body["data"]["imported_count"], 0);
    assert_eq!(body["data"]["skipped_count"], 1);
    assert_eq!(app.post_count().await?, 1);
    assert_eq!(app.file_count().await?, 0);
    let root_files = std::fs::read_dir(app.upload_dir())?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .map(|kind| kind.is_file())
                .unwrap_or(false)
        })
        .count();
    assert_eq!(root_files, 0);
    assert_eq!(
        std::fs::read_dir(app.upload_dir().join(".transactions"))?.count(),
        0
    );
    Ok(())
}

#[tokio::test]
async fn failed_import_never_deletes_an_existing_deduplicated_file() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let admin = app.create_user("dedup-owner-admin", "admin").await?;
    let author = app.create_user("dedup-import-author", "author").await?;
    let png = valid_png(63);
    app.server
        .post("/api/v1/files/upload")
        .authorization_bearer(&admin.token)
        .multipart(file_form("existing.png", "image/png", &png))
        .await
        .assert_status_ok();
    let existing_post = app
        .create_post(&author, "Existing dedup target", "draft")
        .await?;
    let encoded = STANDARD.encode(&png);

    let response = app
        .server
        .post("/api/v1/import/posts")
        .authorization_bearer(&author.token)
        .json(&json!({
            "items": [{
                "filename": "dedup-failure.md",
                "content": format!(
                    "---\ntitle: Dedup failure\nslug: {}\nstatus: draft\n---\n![x](same.png)",
                    existing_post.slug
                ),
                "images": [{
                    "name": "same.png",
                    "data": format!("data:image/png;base64,{encoded}")
                }]
            }]
        }))
        .await;
    response.assert_status_ok();
    let body = response.json::<Value>();
    assert_eq!(body["data"]["imported_count"], 0);
    assert_eq!(body["data"]["skipped_count"], 1);
    assert_eq!(app.file_count().await?, 1);
    assert_eq!(std::fs::read(app.upload_dir().join("existing.png"))?, png);
    Ok(())
}

#[tokio::test]
async fn markdown_import_rolls_back_category_and_image_when_post_insert_fails() -> anyhow::Result<()>
{
    let app = TestApp::new().await?;
    let author = app
        .create_user("import-post-failure-author", "author")
        .await?;
    app.db
        .execute(Statement::from_string(
            app.db.get_database_backend(),
            "CREATE TRIGGER reject_post_insert BEFORE INSERT ON posts \
             BEGIN SELECT RAISE(FAIL, 'forced post insert failure'); END",
        ))
        .await?;
    let encoded = STANDARD.encode(valid_png(62));

    let response = app
        .server
        .post("/api/v1/import/posts")
        .authorization_bearer(&author.token)
        .json(&json!({
            "items": [{
                "filename": "post-failure.md",
                "content": "---\ntitle: Post failure\ncategory: rollback-category\nstatus: draft\n---\n![x](new.png)",
                "images": [{
                    "name": "new.png",
                    "data": format!("data:image/png;base64,{encoded}")
                }]
            }]
        }))
        .await;
    response.assert_status_ok();
    let body = response.json::<Value>();
    assert_eq!(body["data"]["imported_count"], 0);
    assert_eq!(body["data"]["skipped_count"], 1);
    assert_eq!(app.post_count().await?, 0);
    assert_eq!(app.category_count().await?, 0);
    assert_eq!(app.file_count().await?, 0);
    assert!(!app.upload_dir().join("new.png").exists());
    assert_eq!(
        std::fs::read_dir(app.upload_dir().join(".transactions"))?.count(),
        0
    );
    Ok(())
}

#[tokio::test]
async fn markdown_import_preflights_all_images_before_writing_any_file() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let author = app.create_user("import-preflight-author", "author").await?;
    let valid = STANDARD.encode(valid_png(5));
    let forged = STANDARD.encode(b"plain text pretending to be png");

    let response = app
        .server
        .post("/api/v1/import/posts")
        .authorization_bearer(&author.token)
        .json(&json!({
            "items": [{
                "filename": "atomic-import.md",
                "content": "---\ntitle: Atomic import\nstatus: draft\n---\n![a](valid.png) ![b](fake.png)",
                "images": [
                    {"name": "valid.png", "data": format!("data:image/png;base64,{valid}")},
                    {"name": "fake.png", "data": format!("data:image/png;base64,{forged}")}
                ]
            }]
        }))
        .await;
    response.assert_status_ok();
    assert_eq!(response.json::<Value>()["data"]["success"], false);
    assert_eq!(app.file_count().await?, 0);
    assert_eq!(app.post_count().await?, 0);
    assert_eq!(std::fs::read_dir(app.upload_dir())?.count(), 0);
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn public_upload_route_rejects_symlinks_outside_the_upload_root() -> anyhow::Result<()> {
    use std::os::unix::fs::symlink;

    let app = TestApp::new().await?;
    let outside = app.temp_root().join("public-route-secret.txt");
    std::fs::write(&outside, b"must-not-be-public")?;
    symlink(&outside, app.upload_dir().join("secret.txt"))?;

    let response = app.server.get("/uploads/secret.txt").await;
    assert_ne!(response.status_code(), axum::http::StatusCode::OK);
    assert!(!response
        .as_bytes()
        .windows(18)
        .any(|window| window == b"must-not-be-public"));
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn export_rejects_a_symlink_instead_of_reading_outside_uploads() -> anyhow::Result<()> {
    use std::os::unix::fs::symlink;

    let app = TestApp::new().await?;
    let author = app.create_user("export-symlink-author", "author").await?;
    let outside = app.temp_root().join("export-secret.png");
    std::fs::write(&outside, b"do-not-export")?;
    symlink(&outside, app.upload_dir().join("link.png"))?;
    app.insert_file_record(author.id, "link.png", "image/png")
        .await?;
    let post = app
        .create_post(&author, "Export symlink security", "draft")
        .await?;
    app.server
        .put(&format!("/api/v1/posts/{}", post.id))
        .authorization_bearer(&author.token)
        .json(&json!({
            "title": post.title,
            "status": "draft",
            "content": "![secret](./uploads/link.png)"
        }))
        .await
        .assert_status_ok();

    app.server
        .post("/api/v1/export/posts")
        .authorization_bearer(&author.token)
        .json(&json!({"post_ids": [post.id]}))
        .await
        .assert_status_bad_request();
    assert_eq!(std::fs::read(&outside)?, b"do-not-export");
    Ok(())
}

#[tokio::test]
async fn export_preserves_archive_contract_and_author_scope() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let author = app.create_user("export-contract-author", "author").await?;
    let other = app.create_user("export-contract-other", "author").await?;
    let own_post = app
        .create_post(&author, "Export / contract", "draft")
        .await?;
    let other_post = app
        .create_post(&other, "Must not be exported", "draft")
        .await?;

    let response = app
        .server
        .post("/api/v1/export/posts")
        .authorization_bearer(&author.token)
        .json(&json!({"post_ids": [own_post.id, other_post.id]}))
        .await;
    response.assert_status_ok();
    assert_eq!(response.header("content-type").to_str()?, "application/zip");

    let disposition_header = response.header("content-disposition");
    let disposition = disposition_header.to_str()?;
    assert!(
        regex::Regex::new(r#"^attachment; filename="marksharex_export_\d{8}_\d{6}\.zip"$"#)?
            .is_match(disposition),
        "unexpected content-disposition: {disposition}"
    );

    let bytes = response.as_bytes();
    assert_eq!(
        response.header("content-length").to_str()?,
        bytes.len().to_string()
    );
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes.as_ref()))?;
    assert_eq!(archive.len(), 1, "author export must exclude foreign posts");
    let mut markdown = String::new();
    let mut entry = archive.by_index(0)?;
    assert!(entry.name().ends_with("/index.md"));
    entry.read_to_string(&mut markdown)?;
    assert!(markdown.contains("title: \"Export / contract\""));
    assert!(markdown.contains(&format!("slug: \"{}\"", own_post.slug)));
    assert!(!markdown.contains("Must not be exported"));
    Ok(())
}

#[tokio::test]
async fn single_delete_db_failure_preserves_the_row_and_file_bytes() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let admin = app
        .create_user("single-delete-rollback-admin", "admin")
        .await?;
    let png = valid_png(51);
    let upload = app
        .server
        .post("/api/v1/files/upload")
        .authorization_bearer(&admin.token)
        .multipart(file_form("delete-safe.png", "image/png", &png))
        .await;
    upload.assert_status_ok();
    let file_id = upload.json::<Value>()["data"]["id"].as_i64().unwrap();
    app.db
        .execute(Statement::from_string(
            app.db.get_database_backend(),
            "CREATE TRIGGER reject_file_delete BEFORE DELETE ON files \
             BEGIN SELECT RAISE(FAIL, 'forced delete failure'); END",
        ))
        .await?;

    app.server
        .delete(&format!("/api/v1/files/{file_id}"))
        .authorization_bearer(&admin.token)
        .await
        .assert_status_internal_server_error();

    assert_eq!(
        std::fs::read(app.upload_dir().join("delete-safe.png"))?,
        png
    );
    assert_eq!(app.file_count().await?, 1);
    Ok(())
}

#[tokio::test]
async fn batch_delete_preflights_every_historical_name_before_any_deletion() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let admin = app.create_user("batch-delete-admin", "admin").await?;
    let safe_path = app.upload_dir().join("safe.png");
    let safe_bytes = valid_png(31);
    std::fs::write(&safe_path, &safe_bytes)?;
    let safe_id = app
        .insert_file_record(admin.id, "safe.png", "image/png")
        .await?;
    let malicious_id = app
        .insert_file_record(admin.id, "../outside.png", "image/png")
        .await?;

    app.server
        .delete("/api/v1/files/batch")
        .authorization_bearer(&admin.token)
        .json(&json!({"ids": [safe_id, malicious_id]}))
        .await
        .assert_status_bad_request();

    assert_eq!(std::fs::read(&safe_path)?, safe_bytes);
    assert_eq!(app.file_count().await?, 2);
    Ok(())
}

#[tokio::test]
async fn batch_delete_midway_storage_failure_restores_prior_backups_and_database_rows(
) -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let admin = app
        .create_user("batch-delete-compensation-admin", "admin")
        .await?;
    let first_path = app.upload_dir().join("first.png");
    let first_bytes = valid_png(32);
    std::fs::write(&first_path, &first_bytes)?;
    let first_id = app
        .insert_file_record(admin.id, "first.png", "image/png")
        .await?;

    let blocking_path = app.upload_dir().join("blocking.png");
    std::fs::create_dir(&blocking_path)?;
    let blocking_id = app
        .insert_file_record(admin.id, "blocking.png", "image/png")
        .await?;
    let rows_before = files::Entity::find()
        .order_by_asc(files::Column::Id)
        .all(&app.db)
        .await?;

    app.server
        .delete("/api/v1/files/batch")
        .authorization_bearer(&admin.token)
        .json(&json!({"ids": [first_id, blocking_id]}))
        .await
        .assert_status_bad_request();

    let rows_after = files::Entity::find()
        .order_by_asc(files::Column::Id)
        .all(&app.db)
        .await?;
    assert_eq!(rows_after, rows_before);
    assert_eq!(std::fs::read(&first_path)?, first_bytes);
    assert!(blocking_path.is_dir());
    assert_eq!(
        std::fs::read_dir(app.upload_dir().join(".transactions"))?.count(),
        0
    );
    Ok(())
}

#[cfg(target_os = "macos")]
#[tokio::test]
async fn batch_delete_real_second_rename_failure_restores_first_backup_and_database_rows(
) -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let admin = app.create_user("rename-failure-admin", "admin").await?;
    let first_name = "01-first.png";
    let immutable_name = "02-immutable.png";
    let first_bytes = b"first-real-file";
    let immutable_bytes = b"immutable-real-file";
    let first_id = app
        .insert_file_record(admin.id, first_name, "image/png")
        .await?;
    let immutable_id = app
        .insert_file_record(admin.id, immutable_name, "image/png")
        .await?;
    let first_path = app.upload_dir().join(first_name);
    let immutable_path = app.upload_dir().join(immutable_name);
    std::fs::write(&first_path, first_bytes)?;
    std::fs::write(&immutable_path, immutable_bytes)?;
    let immutable_guard = make_file_immutable(&immutable_path)?;
    let rows_before = files::Entity::find()
        .order_by_asc(files::Column::Id)
        .all(&app.db)
        .await?;

    let response = app
        .server
        .delete("/api/v1/files/batch")
        .authorization_bearer(&admin.token)
        .json(&json!({ "ids": [first_id, immutable_id] }))
        .await;
    assert_eq!(
        response.status_code(),
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    );

    let rows_after = files::Entity::find()
        .order_by_asc(files::Column::Id)
        .all(&app.db)
        .await?;
    assert_eq!(rows_after, rows_before);
    assert_eq!(std::fs::read(&first_path)?, first_bytes);
    assert_eq!(std::fs::read(&immutable_path)?, immutable_bytes);
    assert_eq!(
        std::fs::read_dir(app.upload_dir().join(".transactions"))?.count(),
        0
    );
    immutable_guard.clear()?;
    Ok(())
}

#[tokio::test]
async fn deleting_a_historical_traversal_record_cannot_touch_an_outside_file() -> anyhow::Result<()>
{
    let app = TestApp::new().await?;
    let admin = app.create_user("historical-file-admin", "admin").await?;
    let outside = app.temp_root().join("outside-victim.png");
    std::fs::write(&outside, b"sentinel")?;
    let file_id = app
        .insert_file_record(admin.id, "../outside-victim.png", "image/png")
        .await?;

    app.server
        .delete(&format!("/api/v1/files/{file_id}"))
        .authorization_bearer(&admin.token)
        .await
        .assert_status_bad_request();

    assert_eq!(std::fs::read(&outside)?, b"sentinel");
    assert_eq!(app.file_count().await?, 1);
    Ok(())
}
