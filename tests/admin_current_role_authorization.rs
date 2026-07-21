mod common;

use axum::http::StatusCode;
use common::TestApp;
use marksharex::models::entity::{categories, network_resources};
use sea_orm::{EntityTrait, PaginatorTrait};
use serde_json::json;

#[tokio::test]
async fn scalar_and_ops_use_current_active_database_role_for_old_tokens() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let visitor = app.create_user("current-role-visitor", "visitor").await?;
    let sub_admin = app
        .create_user("current-role-sub-admin", "sub_admin")
        .await?;
    let admin = app.create_user("current-role-admin", "admin").await?;

    for path in ["/scalar", "/api/v1/admin/health"] {
        app.server
            .get(path)
            .await
            .assert_status(StatusCode::UNAUTHORIZED);
        app.server
            .get(path)
            .authorization_bearer(&sub_admin.token)
            .await
            .assert_status(StatusCode::FORBIDDEN);
        app.server
            .get(path)
            .authorization_bearer(&admin.token)
            .await
            .assert_status_ok();
    }

    app.set_user_role(visitor.id, "admin").await?;
    for path in ["/scalar", "/api/v1/admin/health"] {
        app.server
            .get(path)
            .authorization_bearer(&visitor.token)
            .await
            .assert_status_ok();
    }
    app.server
        .get("/scalar")
        .add_header("Cookie", format!("scalar_token={}", visitor.token))
        .await
        .assert_status_ok();

    app.set_user_role(admin.id, "author").await?;
    for path in ["/scalar", "/api/v1/admin/health"] {
        app.server
            .get(path)
            .authorization_bearer(&admin.token)
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }

    app.set_user_role(visitor.id, "visitor").await?;
    app.server
        .get("/scalar")
        .add_header("Cookie", format!("scalar_token={}", visitor.token))
        .await
        .assert_status(StatusCode::FORBIDDEN);

    app.set_user_role(admin.id, "admin").await?;
    app.set_user_status(admin.id, "inactive").await?;
    for path in ["/scalar", "/api/v1/admin/health"] {
        app.server
            .get(path)
            .authorization_bearer(&admin.token)
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }

    Ok(())
}

#[tokio::test]
async fn old_jwt_uses_live_principal_for_privileged_and_authenticated_writes() -> anyhow::Result<()>
{
    let app = TestApp::new().await?;
    let user = app.create_user("live-principal-user", "visitor").await?;

    let network_before = network_resources::Entity::find().count(&app.db).await?;
    let network_payload = json!({
        "url": "https://live-principal.example.com/resource",
        "label": "live-principal"
    });
    app.server
        .post("/api/v1/network-resources")
        .authorization_bearer(&user.token)
        .json(&network_payload)
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(
        network_resources::Entity::find().count(&app.db).await?,
        network_before
    );

    app.set_user_role(user.id, "sub_admin").await?;
    app.server
        .post("/api/v1/network-resources")
        .authorization_bearer(&user.token)
        .json(&network_payload)
        .await
        .assert_status_ok();

    app.set_user_role(user.id, "visitor").await?;
    let network_rows = network_resources::Entity::find().all(&app.db).await?;
    app.server
        .post("/api/v1/network-resources")
        .authorization_bearer(&user.token)
        .json(&json!({"url": "https://live-principal.example.com/denied"}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(
        network_resources::Entity::find().all(&app.db).await?,
        network_rows
    );

    app.set_user_status(user.id, "inactive").await?;
    let category_rows = categories::Entity::find().all(&app.db).await?;
    app.server
        .post("/api/v1/categories")
        .authorization_bearer(&user.token)
        .json(&json!({"name": "inactive-denied", "slug": "inactive-denied"}))
        .await
        .assert_status(StatusCode::FORBIDDEN);
    assert_eq!(
        categories::Entity::find().all(&app.db).await?,
        category_rows
    );

    Ok(())
}
