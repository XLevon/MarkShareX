mod common;

use axum::http::StatusCode;
use common::TestApp;
use marksharex::models::entity::{categories, comments, news, post_tags, posts, tags};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, Set,
};
use serde_json::json;

async fn scalar_count(app: &TestApp, sql: &str, id: i32) -> anyhow::Result<i64> {
    Ok(app
        .db
        .query_one(sea_orm::Statement::from_sql_and_values(
            app.db.get_database_backend(),
            sql,
            [id.into()],
        ))
        .await?
        .expect("COUNT must return a row")
        .try_get_by_index::<i64>(0)?)
}

#[tokio::test]
async fn published_markdown_import_is_immediately_searchable() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let author = app.create_user("import-search-author", "author").await?;

    let response = app
        .server
        .post("/api/v1/import/posts")
        .authorization_bearer(&author.token)
        .json(&json!({
            "items": [{
                "filename": "searchable.md",
                "content": "---\ntitle: Imported searchable article\nstatus: published\n---\nimport-index-unique-token",
                "images": []
            }]
        }))
        .await;
    response.assert_status_ok();
    assert_eq!(
        response.json::<serde_json::Value>()["data"]["imported_count"],
        1
    );

    let imported = posts::Entity::find()
        .filter(posts::Column::Title.eq("Imported searchable article"))
        .one(&app.db)
        .await?
        .expect("published import must persist");
    assert_eq!(
        app.state
            .search_engine
            .search("import-index-unique-token", 10)?,
        vec![imported.id as u64]
    );
    Ok(())
}

#[tokio::test]
async fn explicit_import_slug_retained_by_soft_delete_reports_duplicate_before_insert(
) -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let author = app.create_user("import-soft-slug-author", "author").await?;
    let deleted = app
        .create_post(&author, "Retained import slug", "draft")
        .await?;
    app.server
        .delete(&format!("/api/v1/posts/{}", deleted.id))
        .authorization_bearer(&author.token)
        .await
        .assert_status_ok();

    let response = app
        .server
        .post("/api/v1/import/posts")
        .authorization_bearer(&author.token)
        .json(&json!({
            "items": [{
                "filename": "retained-slug.md",
                "content": format!(
                    "---\ntitle: Replacement import\nslug: {}\nstatus: draft\n---\nreplacement",
                    deleted.slug
                ),
                "images": []
            }]
        }))
        .await;
    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    assert_eq!(body["data"]["imported_count"], 0);
    assert_eq!(body["data"]["skipped_count"], 1);
    let error = body["data"]["errors"][0]
        .as_str()
        .expect("duplicate import must report a readable error");
    assert!(error.contains("文章已存在"), "unexpected error: {error}");
    assert!(!error.contains("UNIQUE"), "database error leaked: {error}");
    assert_eq!(
        posts::Entity::find()
            .filter(posts::Column::Slug.eq(&deleted.slug))
            .count(&app.db)
            .await?,
        1
    );
    Ok(())
}

#[tokio::test]
async fn generated_import_slug_uses_a_suffix_for_a_retained_soft_deleted_collision(
) -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let author = app
        .create_user("generated-slug-http-author", "author")
        .await?;
    let retained_base = app
        .create_post(&author, "Generated slug retained base", "draft")
        .await?;
    let retained_suffix = app
        .create_post(&author, "Generated slug retained suffix", "draft")
        .await?;
    for (retained_id, slug) in [
        (retained_base.id, "generated-base"),
        (retained_suffix.id, "generated-base-2"),
    ] {
        let retained_model = posts::Entity::find_by_id(retained_id)
            .one(&app.db)
            .await?
            .expect("fixture post must exist");
        let mut retained = retained_model.into_active_model();
        retained.slug = Set(slug.to_string());
        retained.update(&app.db).await?;
        app.server
            .delete(&format!("/api/v1/posts/{retained_id}"))
            .authorization_bearer(&author.token)
            .await
            .assert_status_ok();
    }

    let response = app
        .server
        .post("/api/v1/import/posts")
        .authorization_bearer(&author.token)
        .json(&json!({
            "items": [{
                "filename": "generated-normalized-base.md",
                "content": "---\ntitle: Generated BASE !!!\nstatus: draft\n---\nnew body",
                "images": []
            }]
        }))
        .await;
    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    assert_eq!(body["data"]["imported_count"], 1);
    assert_eq!(body["data"]["skipped_count"], 0);
    assert_eq!(
        posts::Entity::find()
            .filter(posts::Column::Slug.eq("generated-base"))
            .count(&app.db)
            .await?,
        1
    );
    assert_eq!(
        posts::Entity::find()
            .filter(posts::Column::Slug.eq("generated-base-2"))
            .count(&app.db)
            .await?,
        1
    );
    assert_eq!(
        posts::Entity::find()
            .filter(posts::Column::Slug.eq("generated-base-3"))
            .filter(posts::Column::DeletedAt.is_null())
            .count(&app.db)
            .await?,
        1
    );
    Ok(())
}

#[tokio::test]
async fn unified_search_tag_counts_include_only_live_published_posts() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let admin = app.create_user("search-tag-count-admin", "admin").await?;
    let published = app
        .create_post(&admin, "Search tag published", "published")
        .await?;
    let draft = app.create_post(&admin, "Search tag draft", "draft").await?;
    let deleted = app
        .create_post(&admin, "Search tag deleted", "published")
        .await?;
    let now = chrono::Local::now().naive_local();
    let tag = tags::ActiveModel {
        name: Set("visibility-search-tag".to_string()),
        slug: Set("visibility-search-tag".to_string()),
        user_id: Set(Some(admin.id)),
        deleted_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&app.db)
    .await?;
    for post_id in [published.id, draft.id, deleted.id] {
        post_tags::ActiveModel {
            post_id: Set(post_id),
            tag_id: Set(tag.id),
        }
        .insert(&app.db)
        .await?;
    }
    app.server
        .delete(&format!("/api/v1/posts/{}", deleted.id))
        .authorization_bearer(&admin.token)
        .await
        .assert_status_ok();

    let response = app
        .server
        .get("/api/v1/search?q=visibility-search-tag")
        .await;
    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    let result = body["data"]["tags"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == tag.id)
        .expect("matching tag must be returned");
    assert_eq!(result["post_count"], 1);
    Ok(())
}

#[tokio::test]
async fn public_article_type_and_status_counts_include_only_live_published_posts(
) -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let admin = app.create_user("public-count-admin", "admin").await?;
    let published = app
        .create_post(&admin, "Public count published", "published")
        .await?;
    let draft = app
        .create_post(&admin, "Public count draft", "draft")
        .await?;
    let deleted = app
        .create_post(&admin, "Public count deleted", "published")
        .await?;

    for post in [&published, &draft, &deleted] {
        let mut active: posts::ActiveModel = app.get_post_row(post.id).await?.into();
        active.article_type = Set("tutorial".to_string());
        active.article_status = Set("latest".to_string());
        active.update(&app.db).await?;
    }
    app.server
        .delete(&format!("/api/v1/posts/{}", deleted.id))
        .authorization_bearer(&admin.token)
        .await
        .assert_status_ok();

    let types = app
        .server
        .get("/api/v1/article-types")
        .await
        .json::<serde_json::Value>();
    let tutorial = types["data"]
        .as_array()
        .and_then(|items| items.iter().find(|item| item["code"] == "tutorial"))
        .expect("tutorial type must exist");
    assert_eq!(tutorial["post_count"], 1);

    let statuses = app
        .server
        .get("/api/v1/article-statuses")
        .await
        .json::<serde_json::Value>();
    let latest = statuses["data"]
        .as_array()
        .and_then(|items| items.iter().find(|item| item["code"] == "latest"))
        .expect("latest status must exist");
    assert_eq!(latest["post_count"], 1);

    let admin_types = app
        .server
        .get("/api/v1/admin/article-types")
        .authorization_bearer(&admin.token)
        .await
        .json::<serde_json::Value>();
    let admin_tutorial = admin_types["data"]
        .as_array()
        .and_then(|items| items.iter().find(|item| item["code"] == "tutorial"))
        .expect("admin tutorial type must exist");
    assert_eq!(admin_tutorial["post_count"], 3);

    let admin_statuses = app
        .server
        .get("/api/v1/admin/article-statuses")
        .authorization_bearer(&admin.token)
        .await
        .json::<serde_json::Value>();
    let admin_latest = admin_statuses["data"]
        .as_array()
        .and_then(|items| items.iter().find(|item| item["code"] == "latest"))
        .expect("admin latest status must exist");
    assert_eq!(admin_latest["post_count"], 3);
    Ok(())
}

#[tokio::test]
async fn updating_published_post_to_draft_removes_it_from_search() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let author = app.create_user("unpublish-search-author", "author").await?;
    let post = app
        .create_post(&author, "Unpublish searchable token", "published")
        .await?;
    assert_eq!(
        app.state
            .search_engine
            .search("Unpublish searchable token", 10)?,
        vec![post.id as u64]
    );

    app.server
        .put(&format!("/api/v1/posts/{}", post.id))
        .authorization_bearer(&author.token)
        .json(&json!({"status": "draft"}))
        .await
        .assert_status_ok();

    assert!(app
        .state
        .search_engine
        .search("Unpublish searchable token", 10)?
        .is_empty());
    Ok(())
}

#[tokio::test]
async fn deleting_news_hard_deletes_the_persisted_row() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let admin = app.create_user("news-hard-delete-admin", "admin").await?;
    let created = app
        .server
        .post("/api/v1/admin/news")
        .authorization_bearer(&admin.token)
        .json(&json!({
            "title": "News hard-delete evidence",
            "content": "evidence",
            "status": "draft"
        }))
        .await;
    created.assert_status_ok();
    let news_id = created.json::<serde_json::Value>()["data"]["id"]
        .as_i64()
        .expect("created news id") as i32;

    app.server
        .delete(&format!("/api/v1/admin/news/{news_id}"))
        .authorization_bearer(&admin.token)
        .await
        .assert_status_ok();

    assert!(news::Entity::find_by_id(news_id)
        .one(&app.db)
        .await?
        .is_none());
    Ok(())
}

#[tokio::test]
async fn deleting_a_post_is_soft_and_preserves_relations() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let admin = app.create_user("delete-semantics-admin", "admin").await?;
    let post = app
        .create_post_with_content(
            &admin,
            "Soft delete search token",
            Some("soft-delete-unique-search-token"),
            "published",
        )
        .await?;
    let now = marksharex::utils::now_local();
    let tag = tags::ActiveModel {
        name: Set("Preserved tag".to_string()),
        slug: Set("preserved-tag".to_string()),
        user_id: Set(Some(admin.id)),
        deleted_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&app.db)
    .await?;
    post_tags::ActiveModel {
        post_id: Set(post.id),
        tag_id: Set(tag.id),
    }
    .insert(&app.db)
    .await?;
    app.db
        .execute(sea_orm::Statement::from_sql_and_values(
            app.db.get_database_backend(),
            "INSERT INTO likes (user_id, post_id) VALUES (?, ?);\
             INSERT INTO comments (post_id, user_id, author_name, content, content_html)\
             VALUES (?, ?, 'reader', 'comment', '<p>comment</p>');\
             INSERT INTO read_logs (post_id, user_id, duration_seconds) VALUES (?, ?, 5);",
            [
                admin.id.into(),
                post.id.into(),
                post.id.into(),
                admin.id.into(),
                post.id.into(),
                admin.id.into(),
            ],
        ))
        .await?;

    assert_eq!(
        app.state
            .search_engine
            .search("soft-delete-unique-search-token", 10)?,
        vec![post.id as u64]
    );

    app.server
        .delete(&format!("/api/v1/posts/{}", post.id))
        .authorization_bearer(&admin.token)
        .await
        .assert_status_ok();

    let deleted = posts::Entity::find_by_id(post.id)
        .one(&app.db)
        .await?
        .expect("soft-deleted post row must remain");
    assert!(deleted.deleted_at.is_some());
    assert_eq!(
        scalar_count(
            &app,
            "SELECT COUNT(*) FROM post_tags WHERE post_id = ?",
            post.id
        )
        .await?,
        1
    );
    assert_eq!(
        scalar_count(
            &app,
            "SELECT COUNT(*) FROM likes WHERE post_id = ?",
            post.id
        )
        .await?,
        1
    );
    assert_eq!(
        scalar_count(
            &app,
            "SELECT COUNT(*) FROM comments WHERE post_id = ?",
            post.id
        )
        .await?,
        1
    );
    assert_eq!(
        scalar_count(
            &app,
            "SELECT COUNT(*) FROM read_logs WHERE post_id = ?",
            post.id
        )
        .await?,
        1
    );
    assert!(app
        .state
        .search_engine
        .search("soft-delete-unique-search-token", 10)?
        .is_empty());
    app.server
        .get(&format!("/api/v1/posts/{}", post.id))
        .await
        .assert_status(StatusCode::NOT_FOUND);
    Ok(())
}

#[tokio::test]
async fn referenced_tag_cannot_be_deleted_but_unreferenced_tag_is_hard_deleted(
) -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let admin = app.create_user("tag-delete-admin", "admin").await?;
    let post = app
        .create_post(&admin, "Tag retention post", "draft")
        .await?;
    let now = marksharex::utils::now_local();
    let referenced = tags::ActiveModel {
        name: Set("Referenced".to_string()),
        slug: Set("referenced".to_string()),
        user_id: Set(Some(admin.id)),
        deleted_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&app.db)
    .await?;
    post_tags::ActiveModel {
        post_id: Set(post.id),
        tag_id: Set(referenced.id),
    }
    .insert(&app.db)
    .await?;

    app.server
        .delete(&format!("/api/v1/tags/{}", referenced.id))
        .authorization_bearer(&admin.token)
        .await
        .assert_status(StatusCode::BAD_REQUEST);
    assert!(tags::Entity::find_by_id(referenced.id)
        .one(&app.db)
        .await?
        .is_some());
    assert_eq!(
        post_tags::Entity::find()
            .filter(post_tags::Column::TagId.eq(referenced.id))
            .count(&app.db)
            .await?,
        1
    );

    let unreferenced = tags::ActiveModel {
        name: Set("Unreferenced".to_string()),
        slug: Set("unreferenced".to_string()),
        user_id: Set(Some(admin.id)),
        deleted_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&app.db)
    .await?;
    app.server
        .delete(&format!("/api/v1/tags/{}", unreferenced.id))
        .authorization_bearer(&admin.token)
        .await
        .assert_status_ok();
    assert!(tags::Entity::find_by_id(unreferenced.id)
        .one(&app.db)
        .await?
        .is_none());
    Ok(())
}

#[tokio::test]
async fn referenced_category_is_rejected_and_unreferenced_category_is_hard_deleted(
) -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let admin = app.create_user("category-delete-admin", "admin").await?;
    let post = app
        .create_post(&admin, "Category retention post", "draft")
        .await?;
    let now = marksharex::utils::now_local();
    let referenced = categories::ActiveModel {
        name: Set("Referenced category".to_string()),
        slug: Set("referenced-category".to_string()),
        description: Set(None),
        image_url: Set(None),
        image_filename: Set(None),
        network_resource_id: Set(None),
        is_visible: Set(true),
        parent_id: Set(None),
        sort_order: Set(0),
        user_id: Set(Some(admin.id)),
        deleted_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&app.db)
    .await?;
    let mut post_row: posts::ActiveModel = app.get_post_row(post.id).await?.into();
    post_row.category_id = Set(Some(referenced.id));
    post_row.update(&app.db).await?;

    app.server
        .delete(&format!("/api/v1/categories/{}", referenced.id))
        .authorization_bearer(&admin.token)
        .await
        .assert_status(StatusCode::BAD_REQUEST);
    assert!(categories::Entity::find_by_id(referenced.id)
        .one(&app.db)
        .await?
        .is_some());

    app.server
        .delete(&format!("/api/v1/posts/{}", post.id))
        .authorization_bearer(&admin.token)
        .await
        .assert_status_ok();
    app.server
        .delete(&format!("/api/v1/categories/{}", referenced.id))
        .authorization_bearer(&admin.token)
        .await
        .assert_status(StatusCode::BAD_REQUEST);

    let unreferenced = categories::ActiveModel {
        name: Set("Unreferenced category".to_string()),
        slug: Set("unreferenced-category".to_string()),
        description: Set(None),
        image_url: Set(None),
        image_filename: Set(None),
        network_resource_id: Set(None),
        is_visible: Set(true),
        parent_id: Set(None),
        sort_order: Set(0),
        user_id: Set(Some(admin.id)),
        deleted_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&app.db)
    .await?;
    app.server
        .delete(&format!("/api/v1/categories/{}", unreferenced.id))
        .authorization_bearer(&admin.token)
        .await
        .assert_status_ok();
    assert!(categories::Entity::find_by_id(unreferenced.id)
        .one(&app.db)
        .await?
        .is_none());
    Ok(())
}

#[tokio::test]
async fn deleting_a_comment_sets_deleted_at_without_removing_the_row() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let admin = app.create_user("comment-delete-admin", "admin").await?;
    let post = app
        .create_post(&admin, "Comment retention post", "published")
        .await?;
    let now = marksharex::utils::now_local();
    let comment = comments::ActiveModel {
        post_id: Set(post.id),
        user_id: Set(Some(admin.id)),
        parent_id: Set(None),
        author_name: Set("admin".to_string()),
        author_email: Set(None),
        content: Set("retained comment".to_string()),
        content_html: Set("<p>retained comment</p>".to_string()),
        status: Set("approved".to_string()),
        like_count: Set(0),
        ip_address: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        ..Default::default()
    }
    .insert(&app.db)
    .await?;

    app.server
        .put(&format!("/api/v1/admin/comments/{}", comment.id))
        .authorization_bearer(&admin.token)
        .json(&serde_json::json!({"status": "deleted"}))
        .await
        .assert_status_ok();

    let retained = comments::Entity::find_by_id(comment.id)
        .one(&app.db)
        .await?
        .expect("soft-deleted comment must remain");
    assert!(retained.deleted_at.is_some());
    assert_eq!(retained.content, "retained comment");
    Ok(())
}

#[tokio::test]
async fn soft_deleted_posts_cannot_be_republished_unpublished_or_pinned() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let admin = app.create_user("deleted-mutation-admin", "admin").await?;
    let draft = app
        .create_post(&admin, "Deleted draft mutation guard", "draft")
        .await?;
    let published = app
        .create_post_with_content(
            &admin,
            "Deleted published mutation guard",
            Some("deleted-published-index-token"),
            "published",
        )
        .await?;

    for post_id in [draft.id, published.id] {
        app.server
            .delete(&format!("/api/v1/posts/{post_id}"))
            .authorization_bearer(&admin.token)
            .await
            .assert_status_ok();
    }

    app.server
        .post("/api/v1/admin/posts/batch-publish")
        .authorization_bearer(&admin.token)
        .json(&serde_json::json!({ "ids": [draft.id] }))
        .await
        .assert_status(StatusCode::NOT_FOUND);
    app.server
        .post("/api/v1/admin/posts/batch-unpublish")
        .authorization_bearer(&admin.token)
        .json(&serde_json::json!({ "ids": [published.id] }))
        .await
        .assert_status(StatusCode::NOT_FOUND);
    app.server
        .post(&format!("/api/v1/admin/posts/{}/pin", draft.id))
        .authorization_bearer(&admin.token)
        .await
        .assert_status(StatusCode::NOT_FOUND);
    app.server
        .post(&format!("/api/v1/admin/posts/{}/unpin", published.id))
        .authorization_bearer(&admin.token)
        .await
        .assert_status(StatusCode::NOT_FOUND);
    app.server
        .put("/api/v1/admin/posts/pin-order")
        .authorization_bearer(&admin.token)
        .json(&serde_json::json!({ "post_ids": [draft.id, published.id] }))
        .await
        .assert_status(StatusCode::NOT_FOUND);

    let draft_after = app.get_post_row(draft.id).await?;
    let published_after = app.get_post_row(published.id).await?;
    assert_eq!(draft_after.status, "draft");
    assert_eq!(published_after.status, "published");
    assert!(!draft_after.is_pinned);
    assert!(!published_after.is_pinned);
    assert!(draft_after.deleted_at.is_some());
    assert!(published_after.deleted_at.is_some());
    assert!(app
        .state
        .search_engine
        .search("deleted-published-index-token", 10)?
        .is_empty());
    Ok(())
}

#[tokio::test]
async fn failed_category_hard_delete_rolls_back_child_unlink() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let admin = app.create_user("category-atomic-admin", "admin").await?;
    let now = marksharex::utils::now_local();
    let parent = categories::ActiveModel {
        name: Set("Atomic parent".to_string()),
        slug: Set("atomic-parent".to_string()),
        is_visible: Set(true),
        sort_order: Set(0),
        user_id: Set(Some(admin.id)),
        deleted_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&app.db)
    .await?;
    let child = categories::ActiveModel {
        name: Set("Atomic child".to_string()),
        slug: Set("atomic-child".to_string()),
        is_visible: Set(true),
        parent_id: Set(Some(parent.id)),
        sort_order: Set(0),
        user_id: Set(Some(admin.id)),
        deleted_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&app.db)
    .await?;
    app.db
        .execute_unprepared(&format!(
            "CREATE TRIGGER reject_atomic_parent_delete
             BEFORE DELETE ON categories WHEN OLD.id = {}
             BEGIN SELECT RAISE(ABORT, 'forced category delete failure'); END;",
            parent.id
        ))
        .await?;

    app.server
        .delete(&format!("/api/v1/categories/{}", parent.id))
        .authorization_bearer(&admin.token)
        .await
        .assert_status(StatusCode::INTERNAL_SERVER_ERROR);

    let child_after = categories::Entity::find_by_id(child.id)
        .one(&app.db)
        .await?
        .expect("child category must remain");
    assert_eq!(child_after.parent_id, Some(parent.id));
    assert!(categories::Entity::find_by_id(parent.id)
        .one(&app.db)
        .await?
        .is_some());
    Ok(())
}
