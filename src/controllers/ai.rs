use axum::{
    extract::{State, Path},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use sea_orm::*;
use sea_orm::sea_query::Expr;
use crate::utils::{AppState, AppError, ApiResponse};
use crate::middleware::auth::AuthUser;
use crate::models::entity::{ai_provider, ai_skill, ai_task, ai_agent_config, ai_tool, ai_model, ai_chat_session, ai_chat_message};
use crate::crypto;

// ── Provider ──

#[derive(Serialize, Deserialize, ToSchema)]
pub struct AiProviderResponse {
    pub id: i32,
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub is_active: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_preview: Option<String>,
}

fn mask_key_preview(key: &str) -> String {
    let len = key.chars().count();
    if len <= 10 {
        let first: String = key.chars().take(2).collect();
        let last: String = key.chars().rev().take(1).collect::<Vec<_>>().into_iter().rev().collect();
        format!("{}***{}", first, last)
    } else {
        let first: String = key.chars().take(6).collect();
        let last: String = key.chars().rev().take(2).collect::<Vec<_>>().into_iter().rev().collect();
        let middle = "*".repeat((len - 8).min(12).max(3));
        format!("{}{}{}", first, middle, last)
    }
}

fn provider_to_response(m: ai_provider::Model) -> AiProviderResponse {
    let key = crypto::decrypt(&m.api_key_encrypted);
    let key_preview = if key.is_empty() { None } else { Some(mask_key_preview(&key)) };
    AiProviderResponse {
        id: m.id, name: m.name, provider_type: m.provider_type,
        base_url: m.base_url,
        is_active: m.is_active, created_at: m.created_at, updated_at: m.updated_at,
        key_preview,
    }
}

#[derive(Deserialize, ToSchema)]
pub struct CreateProviderRequest {
    pub name: String,
    #[serde(default = "default_provider_type")]
    pub provider_type: String,
    #[serde(default)]
    pub base_url: String,
    pub api_key: String,
}

fn default_provider_type() -> String { "openai".to_string() }

#[derive(Deserialize, ToSchema)]
pub struct UpdateProviderRequest {
    pub name: Option<String>,
    pub provider_type: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub is_active: Option<bool>,
}

// ── Skill ──

#[derive(Serialize, Deserialize, ToSchema)]
pub struct AiSkillResponse {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub content: String,
    pub output_format: String,
    pub params_template: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl From<ai_skill::Model> for AiSkillResponse {
    fn from(m: ai_skill::Model) -> Self {
        Self {
            id: m.id, name: m.name, description: m.description,
            content: m.content, output_format: m.output_format,
            params_template: m.params_template,
            created_at: m.created_at, updated_at: m.updated_at,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct CreateSkillRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub content: String,
    #[serde(default = "default_output_format")]
    pub output_format: String,
    #[serde(default)]
    pub params_template: String,
}

fn default_output_format() -> String { "markdown".to_string() }

/// Parse {{variable}} placeholders from content and generate JSON template.
/// System variables (date, datetime, time) retain their {{name}} placeholder.
/// User variables get empty string default.
fn generate_params_template(content: &str) -> String {
    use regex::Regex;
    let re = Regex::new(r"\{\{(\w+)\}\}").unwrap();
    let mut vars: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for cap in re.captures_iter(content) {
        vars.insert(cap.get(1).unwrap().as_str());
    }
    if vars.is_empty() {
        return "{}".to_string();
    }
    let system_vars: std::collections::HashSet<&str> =
        ["date", "datetime", "time"].iter().cloned().collect();
    let mut map = serde_json::Map::new();
    for var in vars {
        if system_vars.contains(var) {
            map.insert(var.to_string(), serde_json::Value::String(format!("{{{{{}}}}}", var)));
        } else {
            map.insert(var.to_string(), serde_json::Value::String(String::new()));
        }
    }
    serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string())
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateSkillRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub output_format: Option<String>,
    pub params_template: Option<String>,
}

// ── Task ──

#[derive(Serialize, Deserialize, ToSchema)]
pub struct AiTaskResponse {
    pub id: i32,
    pub name: String,
    pub skill_id: i32,
    pub provider_id: Option<i32>,
    pub agent_config_id: Option<i32>,
    pub model_id: Option<i32>,
    pub cron_expr: String,
    pub params: String,
    pub enabled: bool,
    pub last_run_at: Option<chrono::NaiveDateTime>,
    pub run_count: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl From<ai_task::Model> for AiTaskResponse {
    fn from(m: ai_task::Model) -> Self {
        Self {
            id: m.id, name: m.name, skill_id: m.skill_id, provider_id: m.provider_id,
            agent_config_id: m.agent_config_id,
            model_id: m.model_id,
            cron_expr: m.cron_expr, params: m.params, enabled: m.enabled,
            last_run_at: m.last_run_at, run_count: m.run_count,
            created_at: m.created_at, updated_at: m.updated_at,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct CreateTaskRequest {
    pub skill_id: i32,
    pub provider_id: Option<i32>,
    pub cron_expr: String,
    #[serde(default)]
    pub params: String,
    #[serde(default)]
    pub name: String,
    pub agent_config_id: Option<i32>,
    pub model_id: Option<i32>,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateTaskRequest {
    pub skill_id: Option<i32>,
    pub provider_id: Option<Option<i32>>,
    pub cron_expr: Option<String>,
    pub params: Option<String>,
    pub enabled: Option<bool>,
    pub name: Option<String>,
    pub agent_config_id: Option<Option<i32>>,
    pub model_id: Option<Option<i32>>,
}

// ═══════════════════════════════════════════════════════
//  Default Agent (public)
// ═══════════════════════════════════════════════════════

/// GET /api/v1/ai/default-agent — 检查是否有默认 Agent 配置（公开接口，无需认证）
#[derive(Debug, Serialize, ToSchema)]
pub struct DefaultAgentInfo {
    pub has_default: bool,
    pub id: Option<i32>,
    pub name: Option<String>,
}

pub async fn get_default_agent(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<DefaultAgentInfo>>, AppError> {
    let config = ai_agent_config::Entity::find()
        .filter(ai_agent_config::Column::IsDefault.eq(true))
        .one(&state.db).await?;

    Ok(Json(ApiResponse {
        data: DefaultAgentInfo {
            has_default: config.is_some(),
            id: config.as_ref().map(|c| c.id),
            name: config.map(|c| c.name),
        },
        pagination: None,
    }))
}

// ═══════════════════════════════════════════════════════
//  Providers
// ═══════════════════════════════════════════════════════

/// GET /api/v1/admin/ai/providers
#[utoipa::path(get, path = "/api/v1/admin/ai/providers", responses((status = 200)), tag = "AI")]
pub async fn list_providers(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<AiProviderResponse>>>, AppError> {
    let items: Vec<AiProviderResponse> = ai_provider::Entity::find()
        .order_by_asc(ai_provider::Column::Id)
        .all(&state.db).await?
        .into_iter().map(provider_to_response).collect();
    Ok(Json(ApiResponse { data: items, pagination: None }))
}

/// POST /api/v1/admin/ai/providers
#[utoipa::path(post, path = "/api/v1/admin/ai/providers", responses((status = 200)), tag = "AI")]
pub async fn create_provider(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<CreateProviderRequest>,
) -> Result<Json<ApiResponse<AiProviderResponse>>, AppError> {
    let now = crate::utils::now_local();
    let model = ai_provider::ActiveModel {
        name: Set(req.name),
        provider_type: Set(req.provider_type),
        base_url: Set(req.base_url),
        api_key_encrypted: Set(crypto::encrypt(&req.api_key)),
        created_at: Set(now), updated_at: Set(now),
        ..Default::default()
    };
    let inserted = model.insert(&state.db).await?;
    Ok(Json(ApiResponse { data: provider_to_response(inserted), pagination: None }))
}

/// PUT /api/v1/admin/ai/providers/{id}
#[utoipa::path(put, path = "/api/v1/admin/ai/providers/{id}", responses((status = 200)), tag = "AI")]
pub async fn update_provider(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateProviderRequest>,
) -> Result<Json<ApiResponse<AiProviderResponse>>, AppError> {
    let entry = ai_provider::Entity::find_by_id(id).one(&state.db).await?
        .ok_or_else(|| AppError::NotFound("供应商不存在".into()))?;
    let mut model: ai_provider::ActiveModel = entry.into();
    if let Some(v) = req.name { model.name = Set(v); }
    if let Some(v) = req.provider_type { model.provider_type = Set(v); }
    if let Some(v) = req.base_url { model.base_url = Set(v); }
    if let Some(v) = req.api_key { model.api_key_encrypted = Set(crypto::encrypt(&v)); }
    if let Some(v) = req.is_active { model.is_active = Set(v); }
    model.updated_at = Set(crate::utils::now_local());
    let updated = model.update(&state.db).await?;
    Ok(Json(ApiResponse { data: provider_to_response(updated), pagination: None }))
}

/// DELETE /api/v1/admin/ai/providers/{id}
#[utoipa::path(delete, path = "/api/v1/admin/ai/providers/{id}", responses((status = 200)), tag = "AI")]
pub async fn delete_provider(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    // 检查模型引用
    let model_count = ai_model::Entity::find()
        .filter(ai_model::Column::ProviderId.eq(id))
        .count(&state.db).await?;
    if model_count > 0 {
        return Err(AppError::BadRequest(format!("该供应商下有 {} 个模型，无法删除", model_count)));
    }
    // 检查任务引用
    let task_count = ai_task::Entity::find()
        .filter(ai_task::Column::ProviderId.eq(id))
        .count(&state.db).await?;
    if task_count > 0 {
        return Err(AppError::BadRequest(format!("该供应商被 {} 个定时任务引用，无法删除", task_count)));
    }
    ai_provider::Entity::delete_by_id(id).exec(&state.db).await?;
    Ok(Json(ApiResponse { data: (), pagination: None }))
}

/// POST /api/v1/admin/ai/providers/{id}/test — 测试供应商连接
#[derive(Serialize, ToSchema)]
pub struct ProviderTestResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,
}

#[utoipa::path(post, path = "/api/v1/admin/ai/providers/{id}/test", responses((status = 200)), tag = "AI")]
pub async fn test_provider(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<ProviderTestResponse>>, AppError> {
    let provider = ai_provider::Entity::find_by_id(id).one(&state.db).await?
        .ok_or_else(|| AppError::NotFound("供应商不存在".into()))?;

    let api_key = crypto::decrypt(&provider.api_key_encrypted);
    let base_url = provider.base_url.trim_end_matches('/');

    if base_url.is_empty() {
        return Ok(Json(ApiResponse {
            data: ProviderTestResponse {
                success: false,
                message: "Base URL 未设置".into(),
                models: None,
            },
            pagination: None,
        }));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("创建 HTTP 客户端失败: {}", e)))?;

    match provider.provider_type.as_str() {
        "openai" => test_openai_compatible(&client, base_url, &api_key).await,
        "anthropic" => test_anthropic(&client, base_url, &api_key).await,
        "ollama" => test_ollama(&client, base_url).await,
        other => Ok(Json(ApiResponse {
            data: ProviderTestResponse {
                success: false,
                message: format!("不支持测试的供应商类型: {}", other),
                models: None,
            },
            pagination: None,
        })),
    }
}

/// 测试 OpenAI 兼容供应商 (DeepSeek, SiliconFlow, Groq 等)
async fn test_openai_compatible(
    client: &reqwest::Client, base_url: &str, api_key: &str,
) -> Result<Json<ApiResponse<ProviderTestResponse>>, AppError> {
    // 1. 先测连通性：GET /models
    match client
        .get(format!("{}/models", base_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .send().await
    {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                let models: Vec<String> = body["data"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|m| m["id"].as_str().map(|s| s.to_string())).take(20).collect())
                    .unwrap_or_default();
                Ok(Json(ApiResponse {
                    data: ProviderTestResponse {
                        success: true,
                        message: format!("连接成功 (HTTP 200)，共 {} 个模型", models.len()),
                        models: Some(models),
                    },
                    pagination: None,
                }))
            } else {
                let body_text = resp.text().await.unwrap_or_default();
                Ok(Json(ApiResponse {
                    data: ProviderTestResponse {
                        success: false,
                        message: format!("认证失败 (HTTP {status})：{}", &body_text[..body_text.len().min(200)]),
                        models: None,
                    },
                    pagination: None,
                }))
            }
        }
        Err(e) => Ok(Json(ApiResponse {
            data: ProviderTestResponse {
                success: false,
                message: format!("连接失败：{}", e),
                models: None,
            },
            pagination: None,
        })),
    }
}

/// 测试 Anthropic 供应商 — 发一个轻量 Messages 请求验证认证
async fn test_anthropic(
    client: &reqwest::Client, base_url: &str, api_key: &str,
) -> Result<Json<ApiResponse<ProviderTestResponse>>, AppError> {
    match client
        .post(format!("{}/messages", base_url))
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&serde_json::json!({
            "model": "claude-3-haiku-20240307",
            "max_tokens": 1,
            "messages": [{"role": "user", "content": "hi"}]
        }))
        .send().await
    {
        Ok(resp) => {
            let status = resp.status();
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            if status.is_success() {
                let model = body["model"].as_str().unwrap_or("unknown");
                Ok(Json(ApiResponse {
                    data: ProviderTestResponse {
                        success: true,
                        message: format!("连接成功 (HTTP 200)，测试模型: {model}"),
                        models: Some(vec![model.to_string()]),
                    },
                    pagination: None,
                }))
            } else {
                let err_msg = body["error"]["message"].as_str().unwrap_or("未知错误");
                Ok(Json(ApiResponse {
                    data: ProviderTestResponse {
                        success: false,
                        message: format!("认证失败 (HTTP {status})：{err_msg}"),
                        models: None,
                    },
                    pagination: None,
                }))
            }
        }
        Err(e) => Ok(Json(ApiResponse {
            data: ProviderTestResponse {
                success: false,
                message: format!("连接失败：{}", e),
                models: None,
            },
            pagination: None,
        })),
    }
}

/// 测试 Ollama 供应商 — GET /api/tags（无需认证）
async fn test_ollama(
    client: &reqwest::Client, base_url: &str,
) -> Result<Json<ApiResponse<ProviderTestResponse>>, AppError> {
    match client
        .get(format!("{}/api/tags", base_url))
        .send().await
    {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                let models: Vec<String> = body["models"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|m| m["name"].as_str().map(|s| s.to_string())).take(20).collect())
                    .unwrap_or_default();
                Ok(Json(ApiResponse {
                    data: ProviderTestResponse {
                        success: true,
                        message: format!("连接成功 (HTTP 200)，共 {} 个模型", models.len()),
                        models: Some(models),
                    },
                    pagination: None,
                }))
            } else {
                Ok(Json(ApiResponse {
                    data: ProviderTestResponse {
                        success: false,
                        message: format!("连接失败 (HTTP {status})"),
                        models: None,
                    },
                    pagination: None,
                }))
            }
        }
        Err(e) => Ok(Json(ApiResponse {
            data: ProviderTestResponse {
                success: false,
                message: format!("连接失败：{}", e),
                models: None,
            },
            pagination: None,
        })),
    }
}

// ═══════════════════════════════════════════════════════
//  Skills
// ═══════════════════════════════════════════════════════

/// GET /api/v1/admin/ai/skills
#[utoipa::path(get, path = "/api/v1/admin/ai/skills", responses((status = 200)), tag = "AI")]
pub async fn list_skills(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<AiSkillResponse>>>, AppError> {
    let items: Vec<AiSkillResponse> = ai_skill::Entity::find()
        .order_by_asc(ai_skill::Column::Id)
        .all(&state.db).await?
        .into_iter().map(AiSkillResponse::from).collect();
    Ok(Json(ApiResponse { data: items, pagination: None }))
}

/// POST /api/v1/admin/ai/skills
#[utoipa::path(post, path = "/api/v1/admin/ai/skills", responses((status = 200)), tag = "AI")]
pub async fn create_skill(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<CreateSkillRequest>,
) -> Result<Json<ApiResponse<AiSkillResponse>>, AppError> {
    let now = crate::utils::now_local();
    // Auto-generate params_template from content if not provided
    let params_template = if req.params_template.is_empty() {
        generate_params_template(&req.content)
    } else {
        req.params_template.clone()
    };
    let model = ai_skill::ActiveModel {
        name: Set(req.name), description: Set(req.description),
        content: Set(req.content),
        output_format: Set(req.output_format),
        params_template: Set(params_template),
        created_at: Set(now), updated_at: Set(now),
        ..Default::default()
    };
    let inserted = model.insert(&state.db).await?;
    Ok(Json(ApiResponse { data: AiSkillResponse::from(inserted), pagination: None }))
}

/// PUT /api/v1/admin/ai/skills/{id}
#[utoipa::path(put, path = "/api/v1/admin/ai/skills/{id}", responses((status = 200)), tag = "AI")]
pub async fn update_skill(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateSkillRequest>,
) -> Result<Json<ApiResponse<AiSkillResponse>>, AppError> {
    let entry = ai_skill::Entity::find_by_id(id).one(&state.db).await?
        .ok_or_else(|| AppError::NotFound("技能不存在".into()))?;
    let mut model: ai_skill::ActiveModel = entry.into();
    if let Some(v) = req.name { model.name = Set(v); }
    if let Some(v) = req.description { model.description = Set(v); }
    if let Some(v) = req.content { model.content = Set(v); }
    if let Some(v) = req.output_format { model.output_format = Set(v); }
    if let Some(v) = req.params_template { model.params_template = Set(v); }
    model.updated_at = Set(crate::utils::now_local());
    let updated = model.update(&state.db).await?;
    Ok(Json(ApiResponse { data: AiSkillResponse::from(updated), pagination: None }))
}

/// DELETE /api/v1/admin/ai/skills/{id}
#[utoipa::path(delete, path = "/api/v1/admin/ai/skills/{id}", responses((status = 200)), tag = "AI")]
pub async fn delete_skill(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let task_count = ai_task::Entity::find()
        .filter(ai_task::Column::SkillId.eq(id))
        .count(&state.db).await?;
    if task_count > 0 {
        return Err(AppError::BadRequest(format!("该技能被 {} 个定时任务引用，无法删除", task_count)));
    }
    ai_skill::Entity::delete_by_id(id).exec(&state.db).await?;
    Ok(Json(ApiResponse { data: (), pagination: None }))
}

// ═══════════════════════════════════════════════════════
//  Tasks
// ═══════════════════════════════════════════════════════

/// GET /api/v1/admin/ai/tasks
#[utoipa::path(get, path = "/api/v1/admin/ai/tasks", responses((status = 200)), tag = "AI")]
pub async fn list_tasks(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<AiTaskResponse>>>, AppError> {
    let items: Vec<AiTaskResponse> = ai_task::Entity::find()
        .order_by_asc(ai_task::Column::Id)
        .all(&state.db).await?
        .into_iter().map(AiTaskResponse::from).collect();
    Ok(Json(ApiResponse { data: items, pagination: None }))
}

/// POST /api/v1/admin/ai/tasks
#[utoipa::path(post, path = "/api/v1/admin/ai/tasks", responses((status = 200)), tag = "AI")]
pub async fn create_task(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<ApiResponse<AiTaskResponse>>, AppError> {
    let now = crate::utils::now_local();
    let model = ai_task::ActiveModel {
        name: Set(req.name),
        skill_id: Set(req.skill_id), provider_id: Set(req.provider_id),
        cron_expr: Set(req.cron_expr), params: Set(req.params),
        agent_config_id: Set(req.agent_config_id),
        model_id: Set(req.model_id),
        enabled: Set(req.enabled),
        created_at: Set(now), updated_at: Set(now),
        ..Default::default()
    };
    let inserted = model.insert(&state.db).await?;
    Ok(Json(ApiResponse { data: AiTaskResponse::from(inserted), pagination: None }))
}

/// PUT /api/v1/admin/ai/tasks/{id}
#[utoipa::path(put, path = "/api/v1/admin/ai/tasks/{id}", responses((status = 200)), tag = "AI")]
pub async fn update_task(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<ApiResponse<AiTaskResponse>>, AppError> {
    let entry = ai_task::Entity::find_by_id(id).one(&state.db).await?
        .ok_or_else(|| AppError::NotFound("任务不存在".into()))?;
    let mut model: ai_task::ActiveModel = entry.into();
    if let Some(v) = req.skill_id { model.skill_id = Set(v); }
    if let Some(v) = req.provider_id { model.provider_id = Set(v); }
    if let Some(v) = req.cron_expr { model.cron_expr = Set(v); }
    if let Some(v) = req.params { model.params = Set(v); }
    if let Some(v) = req.enabled { model.enabled = Set(v); }
    if let Some(v) = req.name { model.name = Set(v); }
    if let Some(v) = req.agent_config_id { model.agent_config_id = Set(v); }
    if let Some(v) = req.model_id { model.model_id = Set(v); }
    model.updated_at = Set(crate::utils::now_local());
    let updated = model.update(&state.db).await?;
    Ok(Json(ApiResponse { data: AiTaskResponse::from(updated), pagination: None }))
}

/// DELETE /api/v1/admin/ai/tasks/{id}
#[utoipa::path(delete, path = "/api/v1/admin/ai/tasks/{id}", responses((status = 200)), tag = "AI")]
pub async fn delete_task(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    ai_task::Entity::delete_by_id(id).exec(&state.db).await?;
    Ok(Json(ApiResponse { data: (), pagination: None }))
}

/// POST /api/v1/admin/ai/tasks/{id}/run — 手动执行一次任务（异步启动+轮询追踪）
pub async fn run_task(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    use crate::services::ai_scheduler::AiScheduler;
    use crate::services::ai_trace;

    let task_id = id;
    ai_trace::trace_start(task_id);

    let state_clone = state.clone();
    tokio::spawn(async move {
        match AiScheduler::execute_task_traced(&state_clone, task_id).await {
            Ok(trace) => ai_trace::trace_complete(task_id, trace.final_reply),
            Err(e) => ai_trace::trace_fail(task_id, e.to_string()),
        }
    });

    Ok(Json(ApiResponse {
        data: serde_json::json!({"task_id": task_id, "status": "started"}),
        pagination: None,
    }))
}

/// GET /api/v1/admin/ai/tasks/{id}/trace — 轮询任务执行追踪
pub async fn get_task_trace(
    State(_state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    use crate::services::ai_trace;
    let entry = ai_trace::trace_get(id).unwrap_or_else(|| ai_trace::TraceEntry {
        status: "not_found".to_string(),
        steps: vec![],
        final_reply: String::new(),
        error: None,
    });
    Ok(Json(ApiResponse {
        data: serde_json::to_value(entry).unwrap_or_default(),
        pagination: None,
    }))
}

// ═══════════════════════════════════════════════════════
//  Agent Config
// ═══════════════════════════════════════════════════════

#[derive(Serialize, Deserialize, ToSchema)]
pub struct AgentConfigResponse {
    pub id: i32,
    pub name: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub is_default: bool,
    pub model_id: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl From<ai_agent_config::Model> for AgentConfigResponse {
    fn from(m: ai_agent_config::Model) -> Self {
        Self {
            id: m.id, name: m.name,
            system_prompt: m.system_prompt, user_prompt: m.user_prompt,
            is_default: m.is_default, model_id: m.model_id,
            created_at: m.created_at, updated_at: m.updated_at,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct CreateAgentConfigRequest {
    pub name: String,
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default)]
    pub user_prompt: String,
    pub model_id: Option<i32>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateAgentConfigRequest {
    pub name: Option<String>,
    pub system_prompt: Option<String>,
    pub user_prompt: Option<String>,
    pub is_default: Option<bool>,
    pub model_id: Option<Option<i32>>,
}

/// GET /api/v1/admin/ai/agent-configs
#[utoipa::path(get, path = "/api/v1/admin/ai/agent-configs", responses((status = 200)), tag = "AI")]
pub async fn list_agent_configs(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<AgentConfigResponse>>>, AppError> {
    let items: Vec<AgentConfigResponse> = ai_agent_config::Entity::find()
        .order_by_asc(ai_agent_config::Column::Id)
        .all(&state.db).await?
        .into_iter().map(AgentConfigResponse::from).collect();
    Ok(Json(ApiResponse { data: items, pagination: None }))
}

/// POST /api/v1/admin/ai/agent-configs
#[utoipa::path(post, path = "/api/v1/admin/ai/agent-configs", responses((status = 200)), tag = "AI")]
pub async fn create_agent_config(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<CreateAgentConfigRequest>,
) -> Result<Json<ApiResponse<AgentConfigResponse>>, AppError> {
    let now = crate::utils::now_local();
    let model = ai_agent_config::ActiveModel {
        name: Set(req.name),
        system_prompt: Set(req.system_prompt),
        user_prompt: Set(req.user_prompt),
        model_id: Set(req.model_id),
        created_at: Set(now), updated_at: Set(now),
        ..Default::default()
    };
    let inserted = model.insert(&state.db).await?;
    Ok(Json(ApiResponse { data: AgentConfigResponse::from(inserted), pagination: None }))
}

/// PUT /api/v1/admin/ai/agent-configs/{id}
#[utoipa::path(put, path = "/api/v1/admin/ai/agent-configs/{id}", responses((status = 200)), tag = "AI")]
pub async fn update_agent_config(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateAgentConfigRequest>,
) -> Result<Json<ApiResponse<AgentConfigResponse>>, AppError> {
    let entry = ai_agent_config::Entity::find_by_id(id).one(&state.db).await?
        .ok_or_else(|| AppError::NotFound("Agent 配置不存在".into()))?;
    let mut model: ai_agent_config::ActiveModel = entry.into();
    if let Some(v) = req.name { model.name = Set(v); }
    if let Some(v) = req.system_prompt { model.system_prompt = Set(v); }
    if let Some(v) = req.user_prompt { model.user_prompt = Set(v); }
    if let Some(v) = req.model_id { model.model_id = Set(v); }
    if let Some(v) = req.is_default {
        // 如果设为默认，先取消其他配置的默认
        if v {
            let _ = ai_agent_config::Entity::update_many()
                .col_expr(ai_agent_config::Column::IsDefault, Expr::value(false))
                .filter(ai_agent_config::Column::Id.ne(id))
                .exec(&state.db).await;
        }
        model.is_default = Set(v);
    }
    model.updated_at = Set(crate::utils::now_local());
    let updated = model.update(&state.db).await?;
    Ok(Json(ApiResponse { data: AgentConfigResponse::from(updated), pagination: None }))
}

/// DELETE /api/v1/admin/ai/agent-configs/{id}
#[utoipa::path(delete, path = "/api/v1/admin/ai/agent-configs/{id}", responses((status = 200)), tag = "AI")]
pub async fn delete_agent_config(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    // 检查任务引用
    let task_count = ai_task::Entity::find()
        .filter(ai_task::Column::AgentConfigId.eq(id))
        .count(&state.db).await?;
    if task_count > 0 {
        return Err(AppError::BadRequest(format!("该智能体被 {} 个定时任务引用，无法删除", task_count)));
    }
    // 检查会话引用
    let sess_count = ai_chat_session::Entity::find()
        .filter(ai_chat_session::Column::AgentConfigId.eq(id))
        .count(&state.db).await?;
    if sess_count > 0 {
        return Err(AppError::BadRequest(format!("该智能体被 {} 个聊天会话引用，无法删除", sess_count)));
    }
    ai_agent_config::Entity::delete_by_id(id).exec(&state.db).await?;
    Ok(Json(ApiResponse { data: (), pagination: None }))
}


// ═══════════════════════════════════════════════════════
//  Models
// ═══════════════════════════════════════════════════════

#[derive(Serialize, Deserialize, ToSchema)]
pub struct AiModelResponse {
    pub id: i32,
    pub provider_id: i32,
    pub name: String,
    pub is_default: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl From<ai_model::Model> for AiModelResponse {
    fn from(m: ai_model::Model) -> Self {
        Self {
            id: m.id, provider_id: m.provider_id, name: m.name,
            is_default: m.is_default,
            created_at: m.created_at, updated_at: m.updated_at,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct CreateModelRequest {
    pub provider_id: i32,
    pub name: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateModelRequest {
    pub name: Option<String>,
    pub is_default: Option<bool>,
}

/// GET /api/v1/admin/ai/models?provider_id={id}
#[utoipa::path(get, path = "/api/v1/admin/ai/models", responses((status = 200)), tag = "AI")]
pub async fn list_models(
    State(state): State<AppState>,
    _auth: AuthUser,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<AiModelResponse>>>, AppError> {
    let mut q = ai_model::Entity::find();
    if let Some(pid) = params.get("provider_id").and_then(|v| v.parse::<i32>().ok()) {
        q = q.filter(ai_model::Column::ProviderId.eq(pid));
    }
    let items: Vec<AiModelResponse> = q.order_by_asc(ai_model::Column::Id)
        .all(&state.db).await?
        .into_iter().map(AiModelResponse::from).collect();
    Ok(Json(ApiResponse { data: items, pagination: None }))
}

/// POST /api/v1/admin/ai/models
#[utoipa::path(post, path = "/api/v1/admin/ai/models", responses((status = 200)), tag = "AI")]
pub async fn create_model(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<CreateModelRequest>,
) -> Result<Json<ApiResponse<AiModelResponse>>, AppError> {
    let now = crate::utils::now_local();
    let model = ai_model::ActiveModel {
        provider_id: Set(req.provider_id),
        name: Set(req.name),
        created_at: Set(now), updated_at: Set(now),
        ..Default::default()
    };
    let inserted = model.insert(&state.db).await?;
    Ok(Json(ApiResponse { data: AiModelResponse::from(inserted), pagination: None }))
}

/// PUT /api/v1/admin/ai/models/{id}
#[utoipa::path(put, path = "/api/v1/admin/ai/models/{id}", responses((status = 200)), tag = "AI")]
pub async fn update_model(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateModelRequest>,
) -> Result<Json<ApiResponse<AiModelResponse>>, AppError> {
    let entry = ai_model::Entity::find_by_id(id).one(&state.db).await?
        .ok_or_else(|| AppError::NotFound("模型不存在".into()))?;
    let mut model: ai_model::ActiveModel = entry.into();
    if let Some(v) = req.name { model.name = Set(v); }
    if let Some(v) = req.is_default {
        if v {
            let _ = ai_model::Entity::update_many()
                .col_expr(ai_model::Column::IsDefault, Expr::value(false))
                .filter(ai_model::Column::ProviderId.eq(model.provider_id.clone().unwrap()))
                .filter(ai_model::Column::Id.ne(id))
                .exec(&state.db).await;
        }
        model.is_default = Set(v);
    }
    model.updated_at = Set(crate::utils::now_local());
    let updated = model.update(&state.db).await?;
    Ok(Json(ApiResponse { data: AiModelResponse::from(updated), pagination: None }))
}

/// DELETE /api/v1/admin/ai/models/{id}
#[utoipa::path(delete, path = "/api/v1/admin/ai/models/{id}", responses((status = 200)), tag = "AI")]
pub async fn delete_model(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    // 检查任务引用
    let task_count = ai_task::Entity::find()
        .filter(ai_task::Column::ModelId.eq(id))
        .count(&state.db).await?;
    if task_count > 0 {
        return Err(AppError::BadRequest(format!("该模型被 {} 个定时任务引用，无法删除", task_count)));
    }
    // 检查 Agent 引用
    let agent_count = ai_agent_config::Entity::find()
        .filter(ai_agent_config::Column::ModelId.eq(id))
        .count(&state.db).await?;
    if agent_count > 0 {
        return Err(AppError::BadRequest(format!("该模型被 {} 个智能体引用，无法删除", agent_count)));
    }
    ai_model::Entity::delete_by_id(id).exec(&state.db).await?;
    Ok(Json(ApiResponse { data: (), pagination: None }))
}


// ═══════════════════════════════════════════════════════
//  Chat Sessions
// ═══════════════════════════════════════════════════════

#[derive(Serialize, ToSchema)]
pub struct ChatSessionResponse {
    pub id: i32,
    pub title: String,
    pub user_id: i32,
    pub agent_config_id: Option<i32>,
    pub msg_count: usize,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Serialize, ToSchema)]
pub struct ChatSessionDetail {
    pub id: i32,
    pub title: String,
    pub user_id: i32,
    pub agent_config_id: Option<i32>,
    pub messages: Vec<ChatMessageResponse>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ChatMessageResponse {
    pub id: i32,
    pub role: String,
    pub content: String,
    pub tool_calls: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

/// GET /api/v1/admin/ai/sessions
#[utoipa::path(get, path = "/api/v1/admin/ai/sessions", responses((status = 200)), tag = "AI")]
pub async fn list_sessions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<ChatSessionResponse>>>, AppError> {
    // Admin 可看所有，普通用户只看自己的
    let mut q = ai_chat_session::Entity::find();
    if auth.role != "admin" {
        q = q.filter(ai_chat_session::Column::UserId.eq(auth.user_id));
    }

    let sessions = q.order_by_desc(ai_chat_session::Column::UpdatedAt)
        .all(&state.db).await?;

    // 收集所有 user_id 查 display_name
    let user_ids: Vec<i32> = sessions.iter().map(|s| s.user_id).collect();
    let users = crate::models::entity::users::Entity::find()
        .filter(crate::models::entity::users::Column::Id.is_in(user_ids))
        .all(&state.db).await?;
    use std::collections::HashMap;
    let name_map: HashMap<i32, String> = users.iter().map(|u| {
        (u.id, u.display_name.clone().unwrap_or_else(|| u.username.clone()))
    }).collect();

    let mut result = Vec::new();
    for s in sessions {
        let count = ai_chat_message::Entity::find()
            .filter(ai_chat_message::Column::SessionId.eq(s.id))
            .count(&state.db).await?;
        let user_label = name_map.get(&s.user_id).cloned().unwrap_or_default();
        // Admin 看到：前缀 [显示名]，非 admin 只看到自己的标题不加前缀
        let title = if auth.role == "admin" {
            format!("[{}] {}", user_label, s.title)
        } else {
            s.title.clone()
        };
        result.push(ChatSessionResponse {
            id: s.id, title, user_id: s.user_id,
            agent_config_id: s.agent_config_id,
            msg_count: count as usize,
            created_at: s.created_at, updated_at: s.updated_at,
        });
    }

    Ok(Json(ApiResponse { data: result, pagination: None }))
}

/// POST /api/v1/admin/ai/sessions
#[utoipa::path(post, path = "/api/v1/admin/ai/sessions", responses((status = 200)), tag = "AI")]
pub async fn create_session(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<ApiResponse<ChatSessionResponse>>, AppError> {
    let now = crate::utils::now_local();
    let title = req.title.unwrap_or_else(|| "新会话".to_string());
    let model = ai_chat_session::ActiveModel {
        user_id: Set(auth.user_id),
        title: Set(title),
        agent_config_id: Set(req.agent_config_id),
        created_at: Set(now), updated_at: Set(now),
        ..Default::default()
    };
    let inserted = model.insert(&state.db).await?;
    Ok(Json(ApiResponse {
        data: ChatSessionResponse {
            id: inserted.id, title: inserted.title,
            user_id: inserted.user_id, agent_config_id: inserted.agent_config_id,
            msg_count: 0,
            created_at: inserted.created_at, updated_at: inserted.updated_at,
        },
        pagination: None,
    }))
}

/// GET /api/v1/admin/ai/sessions/{id}
#[utoipa::path(get, path = "/api/v1/admin/ai/sessions/{id}", responses((status = 200)), tag = "AI")]
pub async fn get_session(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<ChatSessionDetail>>, AppError> {
    let session = ai_chat_session::Entity::find_by_id(id).one(&state.db).await?
        .ok_or_else(|| AppError::NotFound("会话不存在".into()))?;

    let messages: Vec<ChatMessageResponse> = ai_chat_message::Entity::find()
        .filter(ai_chat_message::Column::SessionId.eq(id))
        .order_by_asc(ai_chat_message::Column::CreatedAt)
        .all(&state.db).await?
        .into_iter().map(|m| ChatMessageResponse {
            id: m.id, role: m.role, content: m.content,
            tool_calls: m.tool_calls,
            created_at: m.created_at,
        }).collect();

    Ok(Json(ApiResponse {
        data: ChatSessionDetail {
            id: session.id, title: session.title,
            user_id: session.user_id, agent_config_id: session.agent_config_id,
            messages,
            created_at: session.created_at, updated_at: session.updated_at,
        },
        pagination: None,
    }))
}

/// DELETE /api/v1/admin/ai/sessions/{id}
#[utoipa::path(delete, path = "/api/v1/admin/ai/sessions/{id}", responses((status = 200)), tag = "AI")]
pub async fn delete_session(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    ai_chat_session::Entity::delete_by_id(id).exec(&state.db).await?;
    Ok(Json(ApiResponse { data: (), pagination: None }))
}

#[derive(Deserialize, ToSchema)]
pub struct CreateSessionRequest {
    pub title: Option<String>,
    pub agent_config_id: Option<i32>,
}

// ═══════════════════════════════════════════════════════
//  Chat
// ═══════════════════════════════════════════════════════

#[derive(Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct ChatRequest {
    pub message: String,
    #[serde(default)]
    pub history: Vec<ChatMessage>,
    pub agent_config_id: Option<i32>,
    pub session_id: Option<i32>,
    #[serde(default)]
    pub in_admin: bool,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ChatMessage {
    pub role: String,  // "user" | "assistant"
    pub content: String,
}

#[derive(Serialize, ToSchema)]
pub struct ChatResponse {
    pub reply: String,
    pub session_id: i32,
}

/// POST /api/v1/admin/ai/chat — 支持 function calling 的对话接口
#[utoipa::path(post, path = "/api/v1/admin/ai/chat", responses((status = 200)), tag = "AI")]
pub async fn chat(
    State(state): State<AppState>,
    headers: HeaderMap,
    auth: AuthUser,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ApiResponse<ChatResponse>>, AppError> {
    use crate::services::{ai_tools, ai_chat};

    let now = crate::utils::now_local();
    let user_msg = req.message.trim().to_string();

    // ── 斜杠命令处理 ──
    if user_msg.starts_with('/') {
        return handle_slash_command(&state, &auth, &user_msg, &req).await;
    }

    // ── 1. 获取或创建 Session ──
    let session_id = if let Some(sid) = req.session_id {
        // 验证会话存在
        let s = ai_chat_session::Entity::find_by_id(sid).one(&state.db).await?;
        if s.is_none() {
            return Err(AppError::NotFound("会话不存在".into()));
        }
        sid
    } else {
        // 自动创建新会话
        let title = if user_msg.len() > 30 {
            format!("{}...", &user_msg[..30])
        } else {
            user_msg.clone()
        };
        let model = ai_chat_session::ActiveModel {
            user_id: Set(auth.user_id),
            title: Set(title),
            agent_config_id: Set(req.agent_config_id),
            created_at: Set(now), updated_at: Set(now),
            ..Default::default()
        };
        model.insert(&state.db).await?.id
    };

    // ── 2. 保存用户消息 ──
    let msg_model = ai_chat_message::ActiveModel {
        session_id: Set(session_id),
        role: Set("user".to_string()),
        content: Set(user_msg.clone()),
        created_at: Set(now),
        ..Default::default()
    };
    msg_model.insert(&state.db).await?;

    // ── 3. 获取 Agent 配置 ──
    let agent_cfg = if let Some(cfg_id) = req.agent_config_id {
        ai_agent_config::Entity::find_by_id(cfg_id).one(&state.db).await?
    } else {
        // 查找默认配置
        ai_agent_config::Entity::find()
            .filter(ai_agent_config::Column::IsDefault.eq(true))
            .one(&state.db).await?
    }.unwrap_or(ai_agent_config::Model {
        id: 0, name: String::new(),
        system_prompt: String::new(), user_prompt: String::new(),
        is_default: false, model_id: None,
        created_at: chrono::NaiveDateTime::MIN,
        updated_at: chrono::NaiveDateTime::MIN,
    });

    // ── 4. 构建用户消息 ──
    let user_content = if agent_cfg.user_prompt.is_empty() {
        user_msg.clone()
    } else {
        format!("{}\n\n{}", agent_cfg.user_prompt, user_msg)
    };

    // ── 5. 获取历史消息（从 DB 加载） ──
    let history_msgs: Vec<ai_chat_message::Model> = ai_chat_message::Entity::find()
        .filter(ai_chat_message::Column::SessionId.eq(session_id))
        .filter(ai_chat_message::Column::Id.ne(
            ai_chat_message::Entity::find()
                .filter(ai_chat_message::Column::SessionId.eq(session_id))
                .order_by_desc(ai_chat_message::Column::Id)
                .one(&state.db).await?.map(|m| m.id).unwrap_or(0)
        ))
        .order_by_asc(ai_chat_message::Column::CreatedAt)
        .all(&state.db).await?;

    let history: Vec<ai_chat::ChatMessage> = history_msgs.iter().map(|h| {
        ai_chat::ChatMessage { role: h.role.clone(), content: h.content.clone() }
    }).collect();

    // ── 6. 获取模型名 ──
    let model_name = if let Some(mid) = agent_cfg.model_id {
        ai_model::Entity::find_by_id(mid).one(&state.db).await?.map(|m| m.name)
    } else { None };

    // ── 执行 AI ──
    let user_ctx = headers.get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|token| ai_tools::UserContext { token: token.to_string() });
    let registry = ai_tools::create_registry(&state.db, auth.is_privileged(), user_ctx.as_ref()).await;
    let reply = ai_chat::run_function_calling(
        &state, &registry, &agent_cfg.system_prompt,
        &user_content, &history, None, model_name,
    ).await?;

    // ── 8. 保存 assistant 回复 ──
    let now2 = crate::utils::now_local();
    let assistant_msg = ai_chat_message::ActiveModel {
        session_id: Set(session_id),
        role: Set("assistant".to_string()),
        content: Set(reply.clone()),
        created_at: Set(now2),
        ..Default::default()
    };
    assistant_msg.insert(&state.db).await?;

    // ── 9. 更新 session 时间 ──
    let session = ai_chat_session::Entity::find_by_id(session_id).one(&state.db).await?
        .ok_or_else(|| AppError::NotFound("会话不存在".into()))?;
    let mut s_model: ai_chat_session::ActiveModel = session.into();
    s_model.updated_at = Set(now2);
    s_model.update(&state.db).await?;

    Ok(Json(ApiResponse {
        data: ChatResponse { reply, session_id },
        pagination: None,
    }))
}

/// 处理斜杠命令
async fn handle_slash_command(
    state: &AppState,
    auth: &AuthUser,
    cmd: &str,
    req: &ChatRequest,
) -> Result<Json<ApiResponse<ChatResponse>>, AppError> {
    let now = crate::utils::now_local();
    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
    let command = parts[0].to_lowercase();

    let reply = match command.as_str() {
        "/new" => {
            // 创建新会话
            let title = String::from(*parts.get(1).unwrap_or(&"新会话"));
            let model = ai_chat_session::ActiveModel {
                user_id: Set(auth.user_id),
                title: Set(title.clone()),
                agent_config_id: Set(req.agent_config_id),
                created_at: Set(now), updated_at: Set(now),
                ..Default::default()
            };
            let session = model.insert(&state.db).await?;
            format!("✅ 已创建新会话「{}」(ID: {})", title, session.id)
        }
        "/model" => {
            let models = ai_model::Entity::find()
                .order_by_asc(ai_model::Column::ProviderId)
                .order_by_asc(ai_model::Column::Name)
                .all(&state.db).await?;
            if models.is_empty() {
                "📭 暂无可用模型。请在管理后台添加模型。".to_string()
            } else {
                let mut out = "📋 可用模型：\n\n".to_string();
                for m in &models {
                    let provider = ai_provider::Entity::find_by_id(m.provider_id).one(&state.db).await?;
                    let pname = provider.map(|p| p.name).unwrap_or_else(|| "未知".to_string());
                    out.push_str(&format!("  • {} ({})  {}\n", m.name, pname,
                        if m.is_default { "⭐默认" } else { "" }));
                }
                out
            }
        }
        "/help" => {
            "可用命令：\n\
             • /new [标题] — 新建会话\n\
             • /model — 查看可用模型\n\
             • /help — 显示帮助".to_string()
        }
        _ => {
            format!("❓ 未知命令「{}」。输入 /help 查看可用命令。", command)
        }
    };

    // 命令结果也保存为一个 system 消息（不参与 AI 对话）
    let session_id = if let Some(sid) = req.session_id {
        let msg = ai_chat_message::ActiveModel {
            session_id: Set(sid),
            role: Set("system".to_string()),
            content: Set(format!("用户执行命令: {}", cmd)),
            created_at: Set(now),
            ..Default::default()
        };
        msg.insert(&state.db).await?;
        sid
    } else {
        // 没有会话，创建临时会话来保存
        let model = ai_chat_session::ActiveModel {
            user_id: Set(auth.user_id),
            title: Set("命令".to_string()),
            agent_config_id: Set(req.agent_config_id),
            created_at: Set(now), updated_at: Set(now),
            ..Default::default()
        };
        model.insert(&state.db).await?.id
    };

    Ok(Json(ApiResponse {
        data: ChatResponse { reply, session_id },
        pagination: None,
    }))
}

// ═══════════════════════════════════════════════════════
//  Tools
// ═══════════════════════════════════════════════════════

#[derive(Serialize, Deserialize, ToSchema)]
pub struct AiToolResponse {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub function_name: String,
    pub parameters_schema: String,
    pub enabled: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl From<ai_tool::Model> for AiToolResponse {
    fn from(m: ai_tool::Model) -> Self {
        Self {
            id: m.id, name: m.name, description: m.description,
            function_name: m.function_name, parameters_schema: m.parameters_schema,
            enabled: m.enabled,
            created_at: m.created_at, updated_at: m.updated_at,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct CreateToolRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub function_name: String,
    #[serde(default)]
    pub parameters_schema: String,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateToolRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub function_name: Option<String>,
    pub parameters_schema: Option<String>,
    pub enabled: Option<bool>,
}

/// GET /api/v1/admin/ai/tools
#[utoipa::path(get, path = "/api/v1/admin/ai/tools", responses((status = 200)), tag = "AI")]
pub async fn list_tools(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<Json<ApiResponse<Vec<AiToolResponse>>>, AppError> {
    let items: Vec<AiToolResponse> = ai_tool::Entity::find()
        .order_by_asc(ai_tool::Column::Id)
        .all(&state.db).await?
        .into_iter().map(AiToolResponse::from).collect();
    Ok(Json(ApiResponse { data: items, pagination: None }))
}

/// POST /api/v1/admin/ai/tools
#[utoipa::path(post, path = "/api/v1/admin/ai/tools", responses((status = 200)), tag = "AI")]
pub async fn create_tool(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<CreateToolRequest>,
) -> Result<Json<ApiResponse<AiToolResponse>>, AppError> {
    let now = crate::utils::now_local();
    let model = ai_tool::ActiveModel {
        name: Set(req.name), description: Set(req.description),
        function_name: Set(req.function_name), parameters_schema: Set(req.parameters_schema),
        enabled: Set(req.enabled),
        created_at: Set(now), updated_at: Set(now),
        ..Default::default()
    };
    let inserted = model.insert(&state.db).await?;
    Ok(Json(ApiResponse { data: AiToolResponse::from(inserted), pagination: None }))
}

/// PUT /api/v1/admin/ai/tools/{id}
#[utoipa::path(put, path = "/api/v1/admin/ai/tools/{id}", responses((status = 200)), tag = "AI")]
pub async fn update_tool(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateToolRequest>,
) -> Result<Json<ApiResponse<AiToolResponse>>, AppError> {
    let entry = ai_tool::Entity::find_by_id(id).one(&state.db).await?
        .ok_or_else(|| AppError::NotFound("工具不存在".into()))?;
    let mut model: ai_tool::ActiveModel = entry.into();
    if let Some(v) = req.name { model.name = Set(v); }
    if let Some(v) = req.description { model.description = Set(v); }
    if let Some(v) = req.function_name { model.function_name = Set(v); }
    if let Some(v) = req.parameters_schema { model.parameters_schema = Set(v); }
    if let Some(v) = req.enabled { model.enabled = Set(v); }
    model.updated_at = Set(crate::utils::now_local());
    let updated = model.update(&state.db).await?;
    Ok(Json(ApiResponse { data: AiToolResponse::from(updated), pagination: None }))
}

/// DELETE /api/v1/admin/ai/tools/{id}
#[utoipa::path(delete, path = "/api/v1/admin/ai/tools/{id}", responses((status = 200)), tag = "AI")]
pub async fn delete_tool(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    ai_tool::Entity::delete_by_id(id).exec(&state.db).await?;
    Ok(Json(ApiResponse { data: (), pagination: None }))
}
