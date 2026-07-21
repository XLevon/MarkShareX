use crate::middleware::auth::AdminUser;
use crate::models::entity::settings;
use crate::utils::{ApiResponse, AppError, AppState};
use axum::{extract::State, Json};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct SettingsResponse {
    pub settings: HashMap<String, String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum UpdateSettingsRequest {
    Nested { settings: HashMap<String, String> },
    Flat(HashMap<String, String>),
}
/// GET /api/v1/settings — Get site settings (public)

#[utoipa::path(
    get,
    path = "/api/v1/settings",
    responses((status = 200, description = "成功", body = SettingsResponse)),
    tag = "Settings"
)]
pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<SettingsResponse>>, AppError> {
    let items = settings::Entity::find().all(&state.db).await?;
    let map: HashMap<String, String> = items.into_iter().map(|s| (s.key, s.value)).collect();
    Ok(Json(ApiResponse::new(SettingsResponse { settings: map })))
}
/// PUT /api/v1/settings — Update site settings (admin)

#[utoipa::path(
    put,
    path = "/api/v1/settings",
    responses((status = 200, description = "成功", body = SettingsResponse)),
    tag = "Settings"
)]
pub async fn update_settings(
    State(state): State<AppState>,
    _auth: AdminUser,
    Json(req): Json<UpdateSettingsRequest>,
) -> Result<Json<ApiResponse<SettingsResponse>>, AppError> {
    let settings_map = match req {
        UpdateSettingsRequest::Nested { settings } => settings,
        UpdateSettingsRequest::Flat(map) => map,
    };
    let updates_ip_guard = settings_map.keys().any(|key| {
        matches!(
            key.as_str(),
            "ip_blacklist_enabled" | "ip_blacklist" | "ip_whitelist_enabled" | "ip_whitelist"
        )
    });

    let now = crate::utils::now_local();

    for (key, value) in &settings_map {
        let existing = settings::Entity::find()
            .filter(settings::Column::Key.eq(key.as_str()))
            .one(&state.db)
            .await?;

        match existing {
            Some(model) => {
                let mut active: settings::ActiveModel = model.into();
                active.value = Set(value.clone());
                active.updated_at = Set(now);
                active.update(&state.db).await?;
            }
            None => {
                let model = settings::ActiveModel {
                    key: Set(key.clone()),
                    value: Set(value.clone()),
                    updated_at: Set(now),
                };
                model.insert(&state.db).await?;
            }
        }
    }

    if updates_ip_guard {
        state.invalidate_ip_guard_rules_cache().await;
    }

    // Return updated settings
    let items = settings::Entity::find().all(&state.db).await?;
    let map: HashMap<String, String> = items.into_iter().map(|s| (s.key, s.value)).collect();
    Ok(Json(ApiResponse::new(SettingsResponse { settings: map })))
}
