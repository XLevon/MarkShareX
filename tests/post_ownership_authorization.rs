mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{json, Value};

fn list_ids(response: &axum_test::TestResponse) -> Vec<i64> {
    response.json::<Value>()["data"]
        .as_array()
        .expect("admin post list should contain data array")
        .iter()
        .filter_map(|item| item["id"].as_i64())
        .collect()
}

#[tokio::test]
async fn post_create_role_matrix_is_enforced_through_http() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let visitor = app.create_user("create-visitor", "visitor").await?;
    let author = app.create_user("create-author", "author").await?;
    let sub_admin = app.create_user("create-sub-admin", "sub_admin").await?;
    let admin = app.create_user("create-admin", "admin").await?;
    let before = app.post_count().await?;

    app.server
        .post("/api/v1/posts")
        .json(&json!({"title": "anonymous forbidden", "status": "draft"}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    app.server
        .post("/api/v1/posts")
        .authorization_bearer(&visitor.token)
        .json(&json!({"title": "visitor forbidden", "status": "draft"}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(app.post_count().await?, before);

    for (user, title) in [
        (&author, "author legal create"),
        (&sub_admin, "sub admin legal create"),
        (&admin, "admin legal create"),
    ] {
        let response = app
            .server
            .post("/api/v1/posts")
            .authorization_bearer(&user.token)
            .json(&json!({"title": title, "status": "draft"}))
            .await;
        response.assert_status_ok();
        assert_eq!(
            response.json::<Value>()["data"]["user_id"].as_i64(),
            Some(user.id as i64)
        );
    }
    assert_eq!(app.post_count().await?, before + 3);

    Ok(())
}

#[tokio::test]
async fn post_update_ownership_matrix_preserves_denied_rows() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let visitor = app.create_user("update-visitor", "visitor").await?;
    let owner = app.create_user("update-owner", "author").await?;
    let other = app.create_user("update-other", "author").await?;
    let sub_admin = app.create_user("update-sub-admin", "sub_admin").await?;
    let admin = app.create_user("update-admin", "admin").await?;
    let post = app
        .create_post(&owner, "ownership original", "draft")
        .await?;

    app.server
        .put(&format!("/api/v1/posts/{}", post.id))
        .json(&json!({"title": "anonymous overwrite"}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    app.server
        .put(&format!("/api/v1/posts/{}", post.id))
        .authorization_bearer(&visitor.token)
        .json(&json!({"title": "visitor overwrite"}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    app.server
        .put(&format!("/api/v1/posts/{}", post.id))
        .authorization_bearer(&other.token)
        .json(&json!({"title": "other overwrite"}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    app.server
        .put(&format!("/api/v1/posts/{}", post.id))
        .authorization_bearer(&owner.token)
        .json(&json!({"author_id": other.id}))
        .await
        .assert_status(StatusCode::FORBIDDEN);

    let unchanged = app.get_post_row(post.id).await?;
    assert_eq!(unchanged.title, "ownership original");
    assert_eq!(unchanged.user_id, owner.id);

    app.server
        .put(&format!("/api/v1/posts/{}", post.id))
        .authorization_bearer(&owner.token)
        .json(&json!({"title": "owner legal update"}))
        .await
        .assert_status_ok();
    assert_eq!(app.get_post_row(post.id).await?.title, "owner legal update");

    app.server
        .put(&format!("/api/v1/posts/{}", post.id))
        .authorization_bearer(&sub_admin.token)
        .json(&json!({"title": "sub admin legal update"}))
        .await
        .assert_status_ok();
    app.server
        .put(&format!("/api/v1/posts/{}", post.id))
        .authorization_bearer(&admin.token)
        .json(&json!({"title": "admin legal reassignment", "author_id": admin.id}))
        .await
        .assert_status_ok();

    let reassigned = app.get_post_row(post.id).await?;
    assert_eq!(reassigned.title, "admin legal reassignment");
    assert_eq!(reassigned.user_id, admin.id);

    Ok(())
}

#[tokio::test]
async fn post_delete_ownership_matrix_preserves_forbidden_posts() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let visitor = app.create_user("delete-visitor", "visitor").await?;
    let owner = app.create_user("delete-owner", "author").await?;
    let other = app.create_user("delete-other", "author").await?;
    let sub_admin = app.create_user("delete-sub-admin", "sub_admin").await?;
    let admin = app.create_user("delete-admin", "admin").await?;

    let owner_draft = app
        .create_post(&owner, "owner deletable draft", "draft")
        .await?;
    let owner_published = app
        .create_post(&owner, "owner protected published", "published")
        .await?;
    let other_draft = app
        .create_post(&other, "other protected draft", "draft")
        .await?;
    let privileged_draft = app.create_post(&other, "sub admin target", "draft").await?;
    let admin_target = app
        .create_post(&owner, "admin published target", "published")
        .await?;

    app.server
        .delete(&format!("/api/v1/posts/{}", owner_published.id))
        .authorization_bearer(&owner.token)
        .await
        .assert_status(StatusCode::FORBIDDEN);
    app.server
        .delete(&format!("/api/v1/posts/{}", other_draft.id))
        .authorization_bearer(&owner.token)
        .await
        .assert_status(StatusCode::FORBIDDEN);
    app.server
        .delete(&format!("/api/v1/posts/{}", other_draft.id))
        .authorization_bearer(&visitor.token)
        .await
        .assert_status(StatusCode::FORBIDDEN);
    app.server
        .delete(&format!("/api/v1/posts/{}", other_draft.id))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);

    assert!(app
        .get_post_row(owner_published.id)
        .await?
        .deleted_at
        .is_none());
    assert!(app.get_post_row(other_draft.id).await?.deleted_at.is_none());

    app.server
        .delete(&format!("/api/v1/posts/{}", owner_draft.id))
        .authorization_bearer(&owner.token)
        .await
        .assert_status_ok();
    app.server
        .delete(&format!("/api/v1/posts/{}", privileged_draft.id))
        .authorization_bearer(&sub_admin.token)
        .await
        .assert_status_ok();
    app.server
        .delete(&format!("/api/v1/posts/{}", admin_target.id))
        .authorization_bearer(&admin.token)
        .await
        .assert_status_ok();

    assert!(app.get_post_row(owner_draft.id).await.is_err());
    assert!(app.get_post_row(privileged_draft.id).await.is_err());
    assert!(app.get_post_row(admin_target.id).await.is_err());

    Ok(())
}

#[tokio::test]
async fn admin_post_list_rejects_visitors_and_cannot_escape_author_scope() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let visitor = app.create_user("list-visitor", "visitor").await?;
    let owner = app.create_user("list-owner", "author").await?;
    let other = app.create_user("list-other", "author").await?;
    let sub_admin = app.create_user("list-sub-admin", "sub_admin").await?;
    let admin = app.create_user("list-admin", "admin").await?;
    let owner_post = app.create_post(&owner, "list owner post", "draft").await?;
    let other_post = app.create_post(&other, "list other post", "draft").await?;

    app.server
        .get("/api/v1/admin/posts")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    app.server
        .get("/api/v1/admin/posts")
        .authorization_bearer(&visitor.token)
        .await
        .assert_status(StatusCode::FORBIDDEN);

    let owner_list = app
        .server
        .get(&format!("/api/v1/admin/posts?author_id={}", other.id))
        .authorization_bearer(&owner.token)
        .await;
    owner_list.assert_status_ok();
    let owner_ids = list_ids(&owner_list);
    assert!(owner_ids.contains(&(owner_post.id as i64)));
    assert!(!owner_ids.contains(&(other_post.id as i64)));

    for user in [&sub_admin, &admin] {
        let response = app
            .server
            .get("/api/v1/admin/posts?page_size=100")
            .authorization_bearer(&user.token)
            .await;
        response.assert_status_ok();
        let visible = list_ids(&response);
        assert!(visible.contains(&(owner_post.id as i64)));
        assert!(visible.contains(&(other_post.id as i64)));
    }

    Ok(())
}
