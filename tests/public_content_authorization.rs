mod common;

use axum::http::StatusCode;
use common::TestApp;
use marksharex::models::entity::news;
use sea_orm::{ActiveModelTrait, Set};
use serde_json::{json, Value};

fn ids(response: &axum_test::TestResponse) -> Vec<i64> {
    response.json::<Value>()["data"]
        .as_array()
        .expect("list response should contain data array")
        .iter()
        .filter_map(|item| item["id"].as_i64())
        .collect()
}

fn assert_secrets_absent(
    response: &axum_test::TestResponse,
    secret_title: &str,
    secret_content: &str,
) {
    assert!(!response.text().contains(secret_title));
    assert!(!response.text().contains(secret_content));
}

fn assert_hidden(response: &axum_test::TestResponse, secret_title: &str, secret_content: &str) {
    response.assert_status(StatusCode::NOT_FOUND);
    assert_secrets_absent(response, secret_title, secret_content);
}

#[tokio::test]
async fn public_optional_auth_rejects_an_explicit_empty_api_key() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let owner = app.create_user("empty-key-owner", "author").await?;
    let post = app
        .create_post(&owner, "empty-key-public-post", "published")
        .await?;

    app.server
        .get(&format!("/api/v1/posts/{}", post.id))
        .add_header("X-API-Key", "")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);

    Ok(())
}

#[tokio::test]
async fn public_news_date_filters_reject_sql_syntax_without_leaking_drafts() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let now = marksharex::utils::now_local();
    for (title, status, topic_type) in [
        ("public-news-fixture", "published", "public-topic"),
        ("draft-news-secret-9f2c", "draft", "draft-topic-secret-9f2c"),
    ] {
        news::ActiveModel {
            title: Set(title.to_string()),
            summary: Set(format!("{title}-summary")),
            content: Set(format!("{title}-content")),
            content_html: Set(format!("<p>{title}-content</p>")),
            status: Set(status.to_string()),
            topic_type: Set(topic_type.to_string()),
            source_url: Set(String::new()),
            sort_order: Set(0),
            published_at: Set((status == "published").then_some(now)),
            user_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&app.db)
        .await?;
    }

    let injected_date = "x%27%20OR%201%3D1%20--";
    for path in [
        format!("/api/v1/news?date_from={injected_date}&include_content=true"),
        format!("/api/v1/news/topic-types?date_from={injected_date}"),
    ] {
        let response = app.server.get(&path).await;
        response.assert_status_bad_request();
        assert!(!response.text().contains("draft-news-secret-9f2c"));
        assert!(!response.text().contains("draft-topic-secret-9f2c"));
    }

    let today = now.date().format("%Y-%m-%d");
    for path in [
        format!("/api/v1/news?date_from={today}&date_to={today}&include_content=true"),
        format!("/api/v1/news/topic-types?date_from={today}&date_to={today}"),
    ] {
        let response = app.server.get(&path).await;
        response.assert_status_ok();
        assert!(response.text().contains("public"));
        assert!(!response.text().contains("draft-news-secret-9f2c"));
        assert!(!response.text().contains("draft-topic-secret-9f2c"));
    }

    Ok(())
}

#[tokio::test]
async fn public_post_reads_hide_drafts_from_every_non_owner() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let visitor = app.create_user("draft-visitor", "visitor").await?;
    let owner = app.create_user("draft-owner", "author").await?;
    let other_author = app.create_user("draft-other", "author").await?;
    let sub_admin = app.create_user("draft-sub-admin", "sub_admin").await?;
    let admin = app.create_user("draft-admin", "admin").await?;

    let published = app
        .create_post_with_content(
            &owner,
            "public integration article",
            Some("public integration content"),
            "published",
        )
        .await?;
    let secret_content = "draft-content-secret-7f3b9d";
    let draft = app
        .create_post_with_content(
            &owner,
            "secret integration draft",
            Some(secret_content),
            "draft",
        )
        .await?;

    app.server
        .get(&format!("/api/v1/posts/{}", published.id))
        .await
        .assert_status_ok();
    app.server
        .get(&format!("/api/v1/posts/slug/{}", published.slug))
        .await
        .assert_status_ok();

    let anonymous_by_id = app.server.get(&format!("/api/v1/posts/{}", draft.id)).await;
    assert_hidden(&anonymous_by_id, &draft.title, secret_content);
    let anonymous_by_slug = app
        .server
        .get(&format!("/api/v1/posts/slug/{}", draft.slug))
        .await;
    assert_hidden(&anonymous_by_slug, &draft.title, secret_content);

    for user in [&visitor, &other_author] {
        for path in [
            format!("/api/v1/posts/{}", draft.id),
            format!("/api/v1/posts/slug/{}", draft.slug),
        ] {
            let response = app
                .server
                .get(&path)
                .authorization_bearer(&user.token)
                .await;
            assert_hidden(&response, &draft.title, secret_content);
        }
    }

    for user in [&owner, &sub_admin, &admin] {
        for path in [
            format!("/api/v1/posts/{}", draft.id),
            format!("/api/v1/posts/slug/{}", draft.slug),
        ] {
            let response = app
                .server
                .get(&path)
                .authorization_bearer(&user.token)
                .await;
            response.assert_status_ok();
            assert_eq!(
                response.json::<Value>()["data"]["title"].as_str(),
                Some(draft.title.as_str())
            );
            assert!(response.text().contains(secret_content));
        }
    }

    let crafted_list = app
        .server
        .get("/api/v1/posts?status=draft&page_size=100")
        .await;
    crafted_list.assert_status_ok();
    let visible_ids = ids(&crafted_list);
    assert!(visible_ids.contains(&(published.id as i64)));
    assert!(!visible_ids.contains(&(draft.id as i64)));
    assert_secrets_absent(&crafted_list, &draft.title, secret_content);

    Ok(())
}

#[tokio::test]
async fn draft_related_public_endpoints_return_not_found_without_side_effects() -> anyhow::Result<()>
{
    let app = TestApp::new().await?;
    let owner = app.create_user("surface-owner", "author").await?;
    let visitor = app.create_user("surface-visitor", "visitor").await?;
    let secret_content = "related-surface-content-secret-8d4a";
    let draft = app
        .create_post_with_content(
            &owner,
            "secret related surface draft",
            Some(secret_content),
            "draft",
        )
        .await?;

    let adjacent = app
        .server
        .get(&format!("/api/v1/posts/{}/adjacent", draft.id))
        .await;
    assert_hidden(&adjacent, &draft.title, secret_content);

    let like_status = app
        .server
        .get(&format!("/api/v1/posts/{}/like-status", draft.id))
        .authorization_bearer(&visitor.token)
        .await;
    assert_hidden(&like_status, &draft.title, secret_content);

    let toggle_like = app
        .server
        .post(&format!("/api/v1/posts/{}/like", draft.id))
        .authorization_bearer(&visitor.token)
        .await;
    assert_hidden(&toggle_like, &draft.title, secret_content);

    let list_comments = app
        .server
        .get(&format!("/api/v1/posts/{}/comments", draft.id))
        .await;
    assert_hidden(&list_comments, &draft.title, secret_content);

    let create_comment = app
        .server
        .post(&format!("/api/v1/posts/{}/comments", draft.id))
        .json(&json!({
            "content": "must not persist",
            "parent_id": null,
            "author_name": "anonymous",
            "author_email": null
        }))
        .await;
    assert_hidden(&create_comment, &draft.title, secret_content);

    let read_log = app
        .server
        .post("/api/v1/read-logs")
        .json(&json!({
            "post_id": draft.id,
            "duration_seconds": 12,
            "referrer": "integration-test"
        }))
        .await;
    assert_hidden(&read_log, &draft.title, secret_content);

    let counts = app.db_counts_for_post(draft.id).await?;
    assert_eq!(counts.likes, 0);
    assert_eq!(counts.comments, 0);
    assert_eq!(counts.read_logs, 0);

    Ok(())
}

#[tokio::test]
async fn draft_and_soft_deleted_posts_stay_out_of_ssr_sitemap_search_and_adjacent_surfaces(
) -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let owner = app.create_user("surface-matrix-owner", "author").await?;
    let visitor = app.create_user("surface-matrix-visitor", "visitor").await?;
    let other_author = app.create_user("surface-matrix-other", "author").await?;
    let sub_admin = app
        .create_user("surface-matrix-sub-admin", "sub_admin")
        .await?;
    let admin = app.create_user("surface-matrix-admin", "admin").await?;

    let anchor = app
        .create_post_with_content(
            &owner,
            "public adjacent anchor",
            Some("public adjacent anchor content"),
            "published",
        )
        .await?;
    let draft_content = "draftsearchtoken7f3b9d";
    let draft = app
        .create_post_with_content(
            &owner,
            "draft-surface-secret-title",
            Some(draft_content),
            "draft",
        )
        .await?;
    let deleted_content = "softdeletedcontenttoken9c2e";
    let deleted = app
        .create_post_with_content(
            &owner,
            "softdeletedsearchtoken9c2e",
            Some(deleted_content),
            "published",
        )
        .await?;

    let search_before_delete = app
        .server
        .get("/api/v1/search?q=softdeletedsearchtoken9c2e")
        .await;
    search_before_delete.assert_status_ok();
    assert!(search_before_delete.text().contains(&deleted.title));
    let sitemap_before_delete = app.server.get("/sitemap.xml").await;
    sitemap_before_delete.assert_status_ok();
    assert!(sitemap_before_delete.text().contains(&deleted.slug));

    app.set_post_deleted(deleted.id).await?;

    for (post, content) in [(&draft, draft_content), (&deleted, deleted_content)] {
        for path in [
            format!("/api/v1/posts/{}", post.id),
            format!("/api/v1/posts/slug/{}", post.slug),
        ] {
            let anonymous = app.server.get(&path).await;
            assert_hidden(&anonymous, &post.title, content);
            for user in [&visitor, &other_author, &owner, &sub_admin, &admin] {
                let response = app
                    .server
                    .get(&path)
                    .authorization_bearer(&user.token)
                    .await;
                if post.id == draft.id
                    && (user.id == owner.id || user.id == sub_admin.id || user.id == admin.id)
                {
                    response.assert_status_ok();
                } else {
                    assert_hidden(&response, &post.title, content);
                }
            }
        }

        let anonymous_ssr = app.server.get(&format!("/post/{}", post.slug)).await;
        assert_hidden(&anonymous_ssr, &post.title, content);
        for user in [&visitor, &other_author, &owner, &sub_admin, &admin] {
            let response = app
                .server
                .get(&format!("/post/{}", post.slug))
                .authorization_bearer(&user.token)
                .await;
            assert_hidden(&response, &post.title, content);
        }
    }

    let sitemap = app.server.get("/sitemap.xml").await;
    sitemap.assert_status_ok();
    assert_secrets_absent(&sitemap, &draft.slug, draft_content);
    assert_secrets_absent(&sitemap, &deleted.slug, deleted_content);

    for (query, post, content) in [
        (draft_content, &draft, draft_content),
        ("softdeletedsearchtoken9c2e", &deleted, deleted_content),
    ] {
        let response = app.server.get(&format!("/api/v1/search?q={query}")).await;
        response.assert_status_ok();
        assert_secrets_absent(&response, &post.title, content);
    }

    let adjacent = app
        .server
        .get(&format!("/api/v1/posts/{}/adjacent", anchor.id))
        .await;
    adjacent.assert_status_ok();
    assert_secrets_absent(&adjacent, &draft.title, draft_content);
    assert_secrets_absent(&adjacent, &deleted.title, deleted_content);

    Ok(())
}

#[tokio::test]
async fn demoted_author_cannot_read_historical_post_comments_from_admin_api() -> anyhow::Result<()>
{
    let app = TestApp::new().await?;
    let author = app.create_user("demoted-comment-author", "author").await?;
    let post = app
        .create_post(&author, "demoted-comment-post", "published")
        .await?;
    app.server
        .post(&format!("/api/v1/posts/{}/comments", post.id))
        .authorization_bearer(&author.token)
        .json(&json!({"content": "pending-comment-secret"}))
        .await
        .assert_status_ok();

    let before_demotion = app
        .server
        .get("/api/v1/admin/comments")
        .authorization_bearer(&author.token)
        .await;
    before_demotion.assert_status_ok();
    assert!(before_demotion.text().contains("pending-comment-secret"));

    app.set_user_role(author.id, "visitor").await?;
    let after_demotion = app
        .server
        .get("/api/v1/admin/comments")
        .authorization_bearer(&author.token)
        .await;
    after_demotion.assert_status(StatusCode::FORBIDDEN);
    assert!(!after_demotion.text().contains("pending-comment-secret"));

    Ok(())
}
