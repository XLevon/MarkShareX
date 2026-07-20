//! SEC-06 SSRF HTTP integration tests.
//!
//! Verify that the SSRF-safe URL validator is enforced at the Router /
//! controller boundary — not just at the unit-test level.

mod common;

use common::TestApp;
use serde_json::json;

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
    use sea_orm::{ActiveModelTrait, Set};
    use marksharex::models::entity::ai_provider;

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
