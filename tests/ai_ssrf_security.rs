//! SEC-06 SSRF HTTP integration tests.
//!
//! Verify that the SSRF-safe URL validator is enforced at the Router /
//! controller boundary — not just at the unit-test level.

mod common;

use common::TestApp;
use serde_json::json;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;

#[tokio::test]
async fn create_provider_rejects_localhost_base_url() -> anyhow::Result<()> {
    let app = TestApp::new_with_ai(vec![]).await?;
    let admin = app.create_user("admin-ssrf", "admin").await?;

    let resp = app
        .server
        .post("/api/v1/admin/ai/providers")
        .authorization_bearer(&admin.token)
        .json(&json!({
            "name": "ssrf-test",
            "provider_type": "openai",
            "base_url": "http://127.0.0.1:8080",
            "api_key": "sk-test"
        }))
        .await;

    resp.assert_status_bad_request();
    Ok(())
}

#[tokio::test]
async fn create_provider_rejects_cloud_metadata_url() -> anyhow::Result<()> {
    let app = TestApp::new_with_ai(vec![]).await?;
    let admin = app.create_user("admin-ssrf2", "admin").await?;

    let resp = app
        .server
        .post("/api/v1/admin/ai/providers")
        .authorization_bearer(&admin.token)
        .json(&json!({
            "name": "ssrf-test",
            "provider_type": "openai",
            "base_url": "http://169.254.169.254/latest/meta-data/",
            "api_key": "sk-test"
        }))
        .await;

    resp.assert_status_bad_request();
    Ok(())
}

#[tokio::test]
async fn create_provider_rejects_private_rfc1918_url() -> anyhow::Result<()> {
    let app = TestApp::new_with_ai(vec![]).await?;
    let admin = app.create_user("admin-ssrf3", "admin").await?;

    let resp = app
        .server
        .post("/api/v1/admin/ai/providers")
        .authorization_bearer(&admin.token)
        .json(&json!({
            "name": "ssrf-test",
            "provider_type": "openai",
            "base_url": "http://192.168.1.1",
            "api_key": "sk-test"
        }))
        .await;

    resp.assert_status_bad_request();
    Ok(())
}

#[tokio::test]
async fn update_provider_rejects_loopback_base_url() -> anyhow::Result<()> {
    let app = TestApp::new_with_ai(vec![]).await?;
    let admin = app.create_user("admin-ssrf4", "admin").await?;

    // First create with a valid public URL that can be stored (URL validation
    // needs DNS; skip the create step entirely and instead insert a provider
    // directly into the database, then try to update its base_url to a
    // private address).
    //
    // Insert a provider row straight into DB so we don't need DNS.
    use marksharex::models::entity::ai_provider;
    use sea_orm::{ActiveModelTrait, Set};

    let now = marksharex::utils::now_local();
    let provider = ai_provider::ActiveModel {
        name: Set("safe-provider".to_string()),
        provider_type: Set("openai".to_string()),
        base_url: Set("https://api.example.com".to_string()),
        api_key_encrypted: Set("encrypted-dummy".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&app.db)
    .await?;

    let resp = app
        .server
        .put(&format!("/api/v1/admin/ai/providers/{}", provider.id))
        .authorization_bearer(&admin.token)
        .json(&json!({
            "base_url": "http://127.0.0.1:11434"
        }))
        .await;

    resp.assert_status_bad_request();
    Ok(())
}

#[tokio::test]
async fn create_provider_accepts_allowlisted_private_ip() -> anyhow::Result<()> {
    // Allowlist includes 192.168.1.100 — a local Ollama instance.
    let app = TestApp::new_with_ai(vec!["192.168.1.100".to_string()]).await?;
    let admin = app.create_user("admin-ollama", "admin").await?;

    // 192.168.1.100 should be allowed because it's in the allowlist
    let resp = app
        .server
        .post("/api/v1/admin/ai/providers")
        .authorization_bearer(&admin.token)
        .json(&json!({
            "name": "local-ollama",
            "provider_type": "ollama",
            "base_url": "http://192.168.1.100:11434",
            "api_key": ""
        }))
        .await;

    // Should be accepted — allowlisted IP
    resp.assert_status_ok();
    Ok(())
}

#[tokio::test]
async fn safe_get_revalidates_redirect_targets_before_connecting() -> anyhow::Result<()> {
    use axum::{response::Redirect, routing::get, Router};

    let target_hits = Arc::new(AtomicUsize::new(0));
    let target_hits_for_handler = target_hits.clone();
    let target_app = Router::new().route(
        "/metadata",
        get(move || {
            let hits = target_hits_for_handler.clone();
            async move {
                hits.fetch_add(1, Ordering::SeqCst);
                "must-not-be-reached"
            }
        }),
    );
    let target_listener = tokio::net::TcpListener::bind("[::1]:0").await?;
    let target_addr = target_listener.local_addr()?;
    let target_task = tokio::spawn(async move {
        axum::serve(target_listener, target_app).await.unwrap();
    });

    let redirect_url = format!("http://{target_addr}/metadata");
    let entry_app = Router::new().route(
        "/start",
        get(move || {
            let location = redirect_url.clone();
            async move { Redirect::temporary(&location) }
        }),
    );
    let entry_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let entry_addr = entry_listener.local_addr()?;
    let entry_task = tokio::spawn(async move {
        axum::serve(entry_listener, entry_app).await.unwrap();
    });

    let result = marksharex::utils::safe_url::safe_get_follow_redirects(
        &format!("http://{entry_addr}/start"),
        &["127.0.0.1".to_string()],
        Duration::from_secs(2),
        None,
    )
    .await;

    assert!(
        result.is_err(),
        "redirect to a non-allowlisted loopback must fail"
    );
    assert_eq!(target_hits.load(Ordering::SeqCst), 0);
    entry_task.abort();
    target_task.abort();
    Ok(())
}

#[tokio::test]
async fn authenticated_provider_tests_never_follow_redirects() -> anyhow::Result<()> {
    use axum::{response::Redirect, routing::any, Router};
    use marksharex::models::entity::ai_provider;
    use sea_orm::{ActiveModelTrait, Set};

    let target_hits = Arc::new(AtomicUsize::new(0));
    let target_handler_hits = Arc::clone(&target_hits);
    let target_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let target_addr = target_listener.local_addr()?;
    let target_task = tokio::spawn(async move {
        axum::serve(
            target_listener,
            Router::new().route(
                "/*path",
                any(move || {
                    let hits = Arc::clone(&target_handler_hits);
                    async move {
                        hits.fetch_add(1, Ordering::SeqCst);
                        "credential-leak-target"
                    }
                }),
            ),
        )
        .await
        .unwrap();
    });

    let location = format!("http://{target_addr}/stolen");
    let entry_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let entry_addr = entry_listener.local_addr()?;
    let entry_task = tokio::spawn(async move {
        axum::serve(
            entry_listener,
            Router::new().route(
                "/*path",
                any(move || {
                    let location = location.clone();
                    async move { Redirect::temporary(&location) }
                }),
            ),
        )
        .await
        .unwrap();
    });

    let app = TestApp::new_with_ai(vec!["127.0.0.1".to_string()]).await?;
    let admin = app.create_user("redirect-provider-admin", "admin").await?;
    let now = marksharex::utils::now_local();

    for (index, provider_type) in ["openai", "anthropic"].into_iter().enumerate() {
        let provider = ai_provider::ActiveModel {
            name: Set(format!("redirect-{provider_type}-{index}")),
            provider_type: Set(provider_type.to_string()),
            base_url: Set(format!("http://{entry_addr}/v1")),
            api_key_encrypted: Set(marksharex::crypto::encrypt("secret-provider-key")),
            is_active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&app.db)
        .await?;

        app.server
            .post(&format!("/api/v1/admin/ai/providers/{}/test", provider.id))
            .authorization_bearer(&admin.token)
            .await
            .assert_status_ok();
    }

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(target_hits.load(Ordering::SeqCst), 0);
    entry_task.abort();
    target_task.abort();
    Ok(())
}

#[tokio::test]
async fn mapped_ipv6_runtime_provider_is_blocked_before_connect() -> anyhow::Result<()> {
    use axum::{routing::any, Router};
    use marksharex::models::entity::ai_provider;
    use sea_orm::{ActiveModelTrait, Set};

    let hits = Arc::new(AtomicUsize::new(0));
    let server_hits = Arc::clone(&hits);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new().route(
                "/*path",
                any(move || {
                    let hits = Arc::clone(&server_hits);
                    async move {
                        hits.fetch_add(1, Ordering::SeqCst);
                        "should-not-be-reached"
                    }
                }),
            ),
        )
        .await
        .unwrap();
    });

    let app = TestApp::new_with_ai(vec![]).await?;
    let admin = app.create_user("mapped-admin", "admin").await?;
    let provider = ai_provider::ActiveModel {
        name: Set("mapped-loopback-provider".to_string()),
        provider_type: Set("openai".to_string()),
        base_url: Set(format!("http://[::ffff:127.0.0.1]:{port}")),
        api_key_encrypted: Set(marksharex::crypto::encrypt("test-key")),
        is_active: Set(true),
        created_at: Set(marksharex::utils::now_local()),
        updated_at: Set(marksharex::utils::now_local()),
        ..Default::default()
    }
    .insert(&app.db)
    .await?;

    app.server
        .post(&format!("/api/v1/admin/ai/providers/{}/test", provider.id))
        .authorization_bearer(&admin.token)
        .await
        .assert_status_bad_request();

    app.server
        .post("/api/v1/admin/ai/chat")
        .authorization_bearer(&admin.token)
        .json(&json!({
            "message": "mapped runtime check",
            "history": [],
            "agent_config_id": null,
            "session_id": null,
            "in_admin": false
        }))
        .await
        .assert_status_bad_request();
    assert_eq!(hits.load(Ordering::SeqCst), 0);

    server.abort();
    Ok(())
}

#[tokio::test]
async fn dirty_runtime_provider_url_is_blocked_before_provider_test_and_chat_connect(
) -> anyhow::Result<()> {
    use axum::{routing::any, Router};
    use marksharex::models::entity::ai_provider;
    use sea_orm::{ActiveModelTrait, Set};

    let hits = Arc::new(AtomicUsize::new(0));
    let hits_for_handler = hits.clone();
    let target_app = Router::new().route(
        "/*path",
        any(move || {
            let hits = hits_for_handler.clone();
            async move {
                hits.fetch_add(1, Ordering::SeqCst);
                "must-not-be-reached"
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let target_task = tokio::spawn(async move {
        axum::serve(listener, target_app).await.unwrap();
    });

    let app = TestApp::new_with_ai(vec![]).await?;
    let admin = app.create_user("admin-runtime-ssrf", "admin").await?;
    let author = app.create_user("author-runtime-ssrf", "author").await?;
    let now = marksharex::utils::now_local();
    let provider = ai_provider::ActiveModel {
        name: Set("dirty-runtime-provider".to_string()),
        provider_type: Set("openai".to_string()),
        base_url: Set(format!("http://{addr}/v1")),
        api_key_encrypted: Set(marksharex::crypto::encrypt("sk-runtime-test")),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&app.db)
    .await?;

    app.server
        .post(&format!("/api/v1/admin/ai/providers/{}/test", provider.id))
        .authorization_bearer(&admin.token)
        .await
        .assert_status_bad_request();

    app.server
        .post("/api/v1/admin/ai/chat")
        .authorization_bearer(&author.token)
        .json(&json!({
            "message": "runtime SSRF guard",
            "history": [],
            "agent_config_id": null,
            "session_id": null,
            "in_admin": false
        }))
        .await
        .assert_status_bad_request();

    assert_eq!(hits.load(Ordering::SeqCst), 0);
    target_task.abort();
    Ok(())
}
