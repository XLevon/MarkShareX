//! AI 对话核心 — 从 controller 提取出来的 function calling 循环
//!
//! 供 HTTP controller 和 cron 调度器共用。

use sea_orm::*;
use serde_json::Value;
use crate::utils::{AppState, AppError};
use crate::services::ai_tools::ToolRegistry;
use crate::crypto;

/// 执行 function calling 对话循环
///
/// 参数：
/// - state: 应用状态（数据库、配置等）
/// - registry: 工具注册表
/// - system_prompt: 系统提示词
/// - user_message: 用户消息
/// - history: 历史消息（可选）
/// - provider_id: 可选指定供应商 ID（None 则用激活的）
/// - model_name: 可选指定模型名（None 则用供应商默认）
///
/// 返回：LLM 的最终文本回复
pub async fn run_function_calling(
    state: &AppState,
    registry: &ToolRegistry,
    system_prompt: &str,
    user_message: &str,
    history: &[ChatMessage],
    provider_id: Option<i32>,
    model_name: Option<String>,
) -> Result<String, AppError> {
    use crate::models::entity::ai_provider;

    // 1. 获取供应商（指定 id → 激活的）
    let provider = if let Some(pid) = provider_id {
        ai_provider::Entity::find_by_id(pid)
            .one(&state.db)
            .await?
            .ok_or_else(|| AppError::BadRequest(format!("供应商 #{} 不存在", pid)))?
    } else {
        ai_provider::Entity::find()
            .filter(ai_provider::Column::IsActive.eq(true))
            .one(&state.db)
            .await?
            .ok_or_else(|| AppError::BadRequest("没有可用的 AI 供应商，请先添加并启用".into()))?
    };

    let api_key = crypto::decrypt(&provider.api_key_encrypted);
    let base_url = provider.base_url.trim_end_matches('/');

    // 如果没指定 model_name，从 ai_models 表查该供应商的默认模型
    let model = if let Some(name) = model_name {
        name
    } else {
        use crate::models::entity::ai_model;
        let default = ai_model::Entity::find()
            .filter(ai_model::Column::ProviderId.eq(provider.id))
            .filter(ai_model::Column::IsDefault.eq(true))
            .one(&state.db).await
            .ok().flatten();
        default.map(|m| m.name).unwrap_or_else(|| "gpt-3.5-turbo".to_string())
    };

    // 2. 构建消息
    let mut messages: Vec<Value> = Vec::new();
    if !system_prompt.is_empty() {
        messages.push(serde_json::json!({"role": "system", "content": system_prompt}));
    }
    for h in history {
        messages.push(serde_json::json!({"role": h.role, "content": h.content}));
    }
    messages.push(serde_json::json!({"role": "user", "content": user_message}));

    // 3. 工具定义
    let tools = registry.to_openai_tools();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("创建 HTTP 客户端失败: {}", e)))?;
    let max_rounds: u32 = std::env::var("MARKSHAREX_AI_MAX_TOOL_ROUNDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8);

    // 4. Function calling 循环
    for _round in 0..max_rounds {
        let resp = client
            .post(format!("{}/chat/completions", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "model": model,
                "messages": messages,
                "tools": tools,
                "tool_choice": "auto",
                "temperature": 0.7,
            }))
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("LLM 请求失败: {}", e)))?;

        let body: Value = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("LLM 响应解析失败: {}", e)))?;

        let choice = &body["choices"][0];
        let msg = &choice["message"];

        let tool_calls = msg["tool_calls"].as_array();

        if tool_calls.is_none() || tool_calls.unwrap().is_empty() {
            // 无工具调用 → 最终回复
            let reply = msg["content"]
                .as_str()
                .unwrap_or("(空回复)")
                .to_string();
            return Ok(reply);
        }

        // 有工具调用 → 执行
        let calls = tool_calls.unwrap();
        messages.push(msg.clone());

        for tc in calls {
            let tc_id = tc["id"].as_str().unwrap_or("");
            let fn_name = tc["function"]["name"].as_str().unwrap_or("");
            let fn_args_str = tc["function"]["arguments"].as_str().unwrap_or("{}");

            let args: Value =
                serde_json::from_str(fn_args_str).unwrap_or(serde_json::json!({}));

            let result = match registry.get(fn_name) {
                Some(tool) => match tool.execute(args, state).await {
                    Ok(output) => output,
                    Err(e) => format!("工具执行错误: {}", e),
                },
                None => format!("未知工具: {}", fn_name),
            };

            messages.push(serde_json::json!({
                "role": "tool",
                "tool_call_id": tc_id,
                "content": result,
            }));
        }
    }

    Ok("已达到最大工具调用轮次，请简化你的请求。".to_string())
}

/// 简化的聊天消息（仅 role + content）
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}
