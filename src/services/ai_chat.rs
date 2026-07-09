//! AI 对话核心 — 从 controller 提取出来的 function calling 循环
//!
//! 供 HTTP controller 和 cron 调度器共用。

use sea_orm::*;
use serde_json::Value;
use crate::utils::{AppState, AppError};
use crate::services::ai_tools::ToolRegistry;
use crate::crypto;

/// 一次对话轮次的追踪记录
#[derive(Clone, serde::Serialize)]
pub struct TraceStep {
    pub round: u32,
    /// LLM 返回的文本（如果有，通常在第一轮思考或最终回复）
    pub llm_content: Option<String>,
    /// LLM 调用的工具列表
    pub tool_calls: Vec<TraceToolCall>,
}

/// 工具调用记录
#[derive(Clone, serde::Serialize)]
pub struct TraceToolCall {
    pub function_name: String,
    pub arguments: Value,
    /// 工具执行结果（截断到 500 字符）
    pub result_preview: String,
}

/// 执行追踪的完整结果
#[derive(Clone, serde::Serialize)]
pub struct TraceResult {
    pub steps: Vec<TraceStep>,
    pub final_reply: String,
}

/// 执行 function calling 对话循环（带追踪，可选 task_id 用于轮询缓存）
pub async fn run_function_calling_traced(
    state: &AppState,
    registry: &ToolRegistry,
    system_prompt: &str,
    user_message: &str,
    history: &[ChatMessage],
    provider_id: Option<i32>,
    model_name: Option<String>,
    trace_task_id: Option<i32>,
) -> Result<TraceResult, AppError> {
    use crate::models::entity::ai_provider;

    let provider = if let Some(pid) = provider_id {
        ai_provider::Entity::find_by_id(pid).one(&state.db).await?
            .ok_or_else(|| AppError::BadRequest(format!("供应商 #{} 不存在", pid)))?
    } else {
        ai_provider::Entity::find().filter(ai_provider::Column::IsActive.eq(true))
            .one(&state.db).await?
            .ok_or_else(|| AppError::BadRequest("没有可用的 AI 供应商".into()))?
    };

    let api_key = crypto::decrypt(&provider.api_key_encrypted);
    let base_url = provider.base_url.trim_end_matches('/');

    let model = if let Some(name) = model_name {
        name
    } else {
        use crate::models::entity::ai_model;
        ai_model::Entity::find()
            .filter(ai_model::Column::ProviderId.eq(provider.id))
            .filter(ai_model::Column::IsDefault.eq(true))
            .one(&state.db).await.ok().flatten()
            .map(|m| m.name).unwrap_or_else(|| "gpt-3.5-turbo".to_string())
    };

    let mut messages: Vec<Value> = Vec::new();
    if !system_prompt.is_empty() {
        messages.push(serde_json::json!({"role": "system", "content": system_prompt}));
    }
    for h in history {
        messages.push(serde_json::json!({"role": h.role, "content": h.content}));
    }
    messages.push(serde_json::json!({"role": "user", "content": user_message}));

    let tools = registry.to_openai_tools();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("创建 HTTP 客户端失败: {}", e)))?;
    let max_rounds: u32 = std::env::var("MARKSHAREX_AI_MAX_TOOL_ROUNDS")
        .ok().and_then(|s| s.parse().ok()).unwrap_or(8);

    let mut trace_steps: Vec<TraceStep> = Vec::new();

    for round in 0..max_rounds {
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
            .send().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("LLM 请求失败: {}", e)))?;

        let body: Value = resp.json().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("LLM 响应解析失败: {}", e)))?;

        let choice = &body["choices"][0];
        let msg = &choice["message"];
        let tool_calls = msg["tool_calls"].as_array();
        let llm_text = msg["content"].as_str().map(|s| s.to_string());

        if tool_calls.is_none() || tool_calls.unwrap().is_empty() {
            let reply = llm_text.unwrap_or_else(|| "(空回复)".to_string());
            return Ok(TraceResult { steps: trace_steps, final_reply: reply });
        }

        let calls = tool_calls.unwrap();
        messages.push(msg.clone());

        let mut step_tools: Vec<TraceToolCall> = Vec::new();

        for tc in calls {
            let tc_id = tc["id"].as_str().unwrap_or("");
            let fn_name = tc["function"]["name"].as_str().unwrap_or("");
            let fn_args_str = tc["function"]["arguments"].as_str().unwrap_or("{}");
            let args: Value = serde_json::from_str(fn_args_str).unwrap_or(serde_json::json!({}));

            let result = match registry.get(fn_name) {
                Some(tool) => match tool.execute(args.clone(), state).await {
                    Ok(output) => output,
                    Err(e) => format!("工具执行错误: {}", e),
                },
                None => format!("未知工具: {}", fn_name),
            };

            let preview: String = result.chars().take(500).collect();
            step_tools.push(TraceToolCall {
                function_name: fn_name.to_string(),
                arguments: args,
                result_preview: if result.len() > 500 { format!("{}...", preview) } else { preview },
            });

            messages.push(serde_json::json!({
                "role": "tool",
                "tool_call_id": tc_id,
                "content": result,
            }));
        }

        trace_steps.push(TraceStep {
            round: round + 1,
            llm_content: llm_text,
            tool_calls: step_tools,
        });
        // 记录到全局追踪缓存（用于前端轮询）
        if let Some(tid) = trace_task_id {
            crate::services::ai_trace::trace_add_step(tid, trace_steps.last().unwrap().clone());
        }
    }

    Ok(TraceResult {
        steps: trace_steps,
        final_reply: "已达到最大工具调用轮次，请简化你的请求。".to_string(),
    })
}

/// 执行 function calling 对话循环
pub async fn run_function_calling(
    state: &AppState,
    registry: &ToolRegistry,
    system_prompt: &str,
    user_message: &str,
    history: &[ChatMessage],
    provider_id: Option<i32>,
    model_name: Option<String>,
) -> Result<String, AppError> {
    let result = run_function_calling_traced(state, registry, system_prompt, user_message, history, provider_id, model_name, None).await?;
    Ok(result.final_reply)
}

/// 简化的聊天消息（仅 role + content）
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}
