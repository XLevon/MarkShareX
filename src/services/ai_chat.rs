//! AI 对话核心 — 从 controller 提取出来的 function calling 循环
//!
//! 供 HTTP controller 和 cron 调度器共用。

use crate::crypto;
use crate::services::ai_tools::ToolRegistry;
use crate::utils::{AppError, AppState};
use sea_orm::*;
use serde_json::Value;

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
    max_rounds_override: Option<i32>,
) -> Result<TraceResult, AppError> {
    use crate::models::entity::ai_provider;

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
            .one(&state.db)
            .await
            .ok()
            .flatten()
            .map(|m| m.name)
            .unwrap_or_else(|| "gpt-3.5-turbo".to_string())
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

    let allowed = state
        .config
        .ai
        .as_ref()
        .map(|config| config.allowed_provider_networks.as_slice())
        .unwrap_or(&[]);
    let endpoint = format!("{}/chat/completions", base_url);
    let max_rounds: u32 = resolve_max_rounds(max_rounds_override, state);

    let mut trace_steps: Vec<TraceStep> = Vec::new();
    let mut consecutive_empty: u32 = 0; // 连续空响应计数

    for round in 0..max_rounds {
        let payload = serde_json::json!({
            "model": model,
            "messages": messages,
            "tools": tools,
            "tool_choice": "auto",
            "temperature": 0.7,
        });
        let resp = crate::utils::safe_url::send_safe_request(
            &endpoint,
            allowed,
            std::time::Duration::from_secs(120),
            None,
            |client, url| {
                client
                    .post(url.clone())
                    .header("Authorization", format!("Bearer {}", api_key))
                    .json(&payload)
            },
        )
        .await?;

        let status = resp.status();
        let resp_text = resp.text().await.unwrap_or_default();
        let body: Value = serde_json::from_str(&resp_text).map_err(|e| {
            AppError::Internal(anyhow::anyhow!(
                "LLM 响应解析失败 (HTTP {}): {} — body: {}",
                status.as_u16(),
                e,
                &resp_text[..resp_text.len().min(500)]
            ))
        })?;

        let choice = &body["choices"][0];
        let msg = &choice["message"];
        let tool_calls = msg["tool_calls"].as_array();
        let llm_text = msg["content"].as_str().map(|s| s.to_string());

        if tool_calls.is_none() || tool_calls.unwrap().is_empty() {
            // 有文本内容 → LLM 正常结束
            if let Some(ref t) = llm_text {
                if !t.trim().is_empty() {
                    return Ok(TraceResult {
                        steps: trace_steps,
                        final_reply: t.clone(),
                    });
                }
            }

            // 空内容：可能是 LLM 提前放弃
            consecutive_empty += 1;
            if consecutive_empty >= 2 {
                // 连续两次空响应 → 确实无法继续
                let total_rounds = trace_steps.len();
                tracing::warn!(
                    "LLM 连续 {} 次返回空内容，强制终止 (round={})",
                    consecutive_empty,
                    round + 1
                );
                return Ok(TraceResult {
                    steps: trace_steps,
                    final_reply: format!(
                        "任务已终止（LLM 连续返回空内容，共 {} 轮工具调用）",
                        total_rounds
                    ),
                });
            }

            // 第一次空响应：注入续接提示，让 LLM 继续
            tracing::info!("LLM 返回空内容 (round={})，注入续接提示", round + 1);
            let reminder = "你还没有完成任务。请继续搜索并创建资讯，不要提前停止。";
            messages.push(serde_json::json!({"role": "user", "content": reminder}));
            continue;
        }

        consecutive_empty = 0; // 有工具调用，重置空响应计数

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
                result_preview: if result.len() > 500 {
                    format!("{}...", preview)
                } else {
                    preview
                },
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
    max_rounds_override: Option<i32>,
) -> Result<String, AppError> {
    let result = run_function_calling_traced(
        state,
        registry,
        system_prompt,
        user_message,
        history,
        provider_id,
        model_name,
        None,
        max_rounds_override,
    )
    .await?;
    Ok(result.final_reply)
}

/// 解析最大工具调用轮次
/// 优先级: override > 任务配置 > 全局配置(>0) > 硬编码默认8
fn resolve_max_rounds(override_val: Option<i32>, state: &AppState) -> u32 {
    // 1. 调用方传入的 override（任务级最高优先）
    if let Some(v) = override_val {
        if v > 0 {
            return v as u32;
        }
    }
    // 2. 全局配置（0 视为未设置，用默认）
    let config_val = state
        .config
        .ai
        .as_ref()
        .map(|c| c.max_tool_rounds)
        .unwrap_or(0);
    if config_val > 0 {
        return config_val;
    }
    // 3. 硬编码默认
    8
}

/// 简化的聊天消息（仅 role + content）
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}
