mod common;

use axum_test::multipart::{MultipartForm, Part};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use sea_orm::{ConnectionTrait, Statement};
use serde_json::{json, Value};

use common::TestApp;

fn valid_png(red: u8) -> Vec<u8> {
    let image = image::RgbaImage::from_pixel(1, 1, image::Rgba([red, 0, 0, 255]));
    let mut cursor = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .expect("test PNG must encode");
    cursor.into_inner()
}

fn file_form(filename: &str, mime: &str, bytes: &[u8]) -> MultipartForm {
    file_form_with_field("file", filename, mime, bytes)
}

fn batch_file_form(filename: &str, mime: &str, bytes: &[u8]) -> MultipartForm {
    file_form_with_field("files", filename, mime, bytes)
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
