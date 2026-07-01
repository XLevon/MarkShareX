//! AI 定时调度器 — 解析 cron 表达式，到点自动执行 AI 技能任务
//!
//! 每分钟检查一次 ai_tasks 表，对到期的 enabled 任务调用 function calling。

use chrono::Local;
use cron::Schedule;
use sea_orm::*;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{info, warn, error};

use crate::utils::AppState;
use crate::services::ai_chat;
use crate::services::ai_tools;
use crate::models::entity::{ai_task, ai_skill, ai_agent_config};

/// 任务调度器
pub struct AiScheduler {
    state: Arc<AppState>,
    /// 防止同一任务并发执行
    running: Arc<Mutex<std::collections::HashSet<i32>>>,
}

impl AiScheduler {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            running: Arc::new(Mutex::new(std::collections::HashSet::new())),
        }
    }

    /// 启动调度循环（后台运行，永不返回）
    pub async fn start(self) {
        info!("🤖 AI 定时调度器已启动，每 60 秒检查一次");

        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            if let Err(e) = self.tick().await {
                error!("调度器 tick 出错: {:?}", e);
            }
        }
    }

    /// 单次检查：查询到期任务并执行
    async fn tick(&self) -> Result<(), anyhow::Error> {
        let tasks = ai_task::Entity::find()
            .filter(ai_task::Column::Enabled.eq(true))
            .all(&self.state.db)
            .await?;

        let now = Local::now();
        tracing::debug!("调度器 tick: {} 个任务, 当前时间 {}", tasks.len(), now);

        if tasks.is_empty() {
            return Ok(());
        }

        for task in tasks {
            let cron_expr = normalize_cron(&task.cron_expr);
            tracing::debug!("  任务 #{}: cron={}", task.id, cron_expr);
            let schedule = match Schedule::from_str(&cron_expr) {
                Ok(s) => s,
                Err(e) => {
                    warn!("任务 #{} cron 表达式无效 '{}': {}", task.id, task.cron_expr, e);
                    continue;
                }
            };

            let matched = schedule.includes(now);
            tracing::debug!("  任务 #{}: matched={}", task.id, matched);
            if !matched {
                continue;
            }

            // 防重入：如果任务正在执行，跳过
            {
                let mut running = self.running.lock().await;
                if running.contains(&task.id) {
                    continue;
                }
                running.insert(task.id);
            }

            // 异步执行（不阻塞 tick 循环）
            let state = self.state.clone();
            let running = self.running.clone();
            let task_id = task.id;

            tokio::spawn(async move {
                let result = Self::execute_task(&state, task_id).await;
                // 执行完毕，从 running 集合中移除
                running.lock().await.remove(&task_id);
                match result {
                    Ok(msg) => info!("调度任务 #{} 执行完成: {}...", task_id, &msg[..msg.len().min(100)]),
                    Err(e) => error!("调度任务 #{} 执行失败: {}", task_id, e),
                }
            });
        }

        Ok(())
    }

    /// 执行单个任务
    async fn execute_task(state: &AppState, task_id: i32) -> Result<String, anyhow::Error> {
        // 1. 查询任务详情
        let task = ai_task::Entity::find_by_id(task_id)
            .one(&state.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("任务 #{} 不存在", task_id))?;

        // 2. 获取关联的技能
        let skill = ai_skill::Entity::find_by_id(task.skill_id)
            .one(&state.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("技能 #{} 不存在", task.skill_id))?;

        info!(
            "⏰ 执行调度任务 #{}: 技能='{}' cron='{}'",
            task.id, skill.name, task.cron_expr
        );

        // 3. 构建工具注册表
        let registry = ai_tools::create_registry(&state.db).await;

        // 4. 构建用户消息（skill.content 作为 user prompt，params 替换模板变量）
        let user_message = build_user_message(&skill.content, &task.params);

    // 5. 获取 Agent 配置（任务指定 → 默认 → 硬编码）
    let system_prompt = get_agent_system_prompt(state, task.agent_config_id).await;

    // 6. 回退 provider_id 和 model_id：任务指定 → Agent 配置 → 默认
    let (agent_provider_id, agent_model_id) = get_agent_defaults(state, task.agent_config_id).await;
    let provider_id = task.provider_id.or(agent_provider_id);
    let model_id = task.model_id.or(agent_model_id);

    // 7. 获取模型名（task 指定 → agent 默认 → provider 默认）
    let model_name = get_model_name(state, model_id).await;

    // 8. 执行 function calling
    let reply = ai_chat::run_function_calling(
        state,
        &registry,
        &system_prompt,
        &user_message,
        &[], // 无历史消息
        provider_id,
        model_name,
    ).await?;

        // 7. 更新任务状态
        let now = Local::now().naive_local();
        let new_run_count = task.run_count + 1;
        let mut model: ai_task::ActiveModel = task.into();
        model.last_run_at = Set(Some(now));
        model.run_count = Set(new_run_count);
        model.updated_at = Set(now);
        model.update(&state.db).await?;

        info!("调度任务 #{} 状态已更新 (run_count={})", task_id, new_run_count);

        Ok(reply)
    }
}

/// 获取 Agent 的 system_prompt：任务指定 → 默认 → 硬编码
async fn get_agent_system_prompt(state: &AppState, agent_config_id: Option<i32>) -> String {
    // 1. 优先用任务指定的 agent config
    if let Some(cfg_id) = agent_config_id {
        if let Ok(Some(cfg)) = ai_agent_config::Entity::find_by_id(cfg_id).one(&state.db).await {
            if !cfg.system_prompt.is_empty() {
                return cfg.system_prompt;
            }
        }
    }

    // 2. Fallback 到默认配置
    if let Ok(Some(cfg)) = ai_agent_config::Entity::find()
        .filter(ai_agent_config::Column::IsDefault.eq(true))
        .one(&state.db).await
    {
        if !cfg.system_prompt.is_empty() {
            return cfg.system_prompt;
        }
    }

    // 3. 最终硬编码 fallback
    "你是一个自动化的内容采集助手。你必须使用提供的工具来完成用户的请求。\
     你无法直接回答——每次都必须调用至少一个工具。".to_string()
}

/// 获取模型名：task 指定 → ai_models 默认 → None（用 provider.default_model）
async fn get_model_name(state: &AppState, model_id: Option<i32>) -> Option<String> {
    use crate::models::entity::ai_model;
    if let Some(mid) = model_id {
        if let Ok(Some(m)) = ai_model::Entity::find_by_id(mid).one(&state.db).await {
            return Some(m.name);
        }
    }
    // 未指定 model_id，返回 None（ai_chat 会用 provider.default_model）
    None
}

/// 获取 Agent 配置中的 model_id，以及通过 model 推导的 provider_id
async fn get_agent_defaults(
    state: &AppState,
    agent_config_id: Option<i32>,
) -> (Option<i32>, Option<i32>) {
    let model_id = if let Some(cfg_id) = agent_config_id {
        if let Ok(Some(cfg)) = ai_agent_config::Entity::find_by_id(cfg_id)
            .one(&state.db).await
        {
            cfg.model_id
        } else {
            None
        }
    } else {
        // 查默认 agent
        if let Ok(Some(cfg)) = ai_agent_config::Entity::find()
            .filter(ai_agent_config::Column::IsDefault.eq(true))
            .one(&state.db).await
        {
            cfg.model_id
        } else {
            None
        }
    };

    // 通过 model 查 provider_id
    let provider_id = if let Some(mid) = model_id {
        use crate::models::entity::ai_model;
        if let Ok(Some(m)) = ai_model::Entity::find_by_id(mid).one(&state.db).await {
            Some(m.provider_id)
        } else {
            None
        }
    } else {
        None
    };

    (provider_id, model_id)
}

/// 构建用户消息：替换模板中的 {{变量}} 为实际值
fn build_user_message(template: &str, params_json: &str) -> String {
    let params: serde_json::Value = serde_json::from_str(params_json).unwrap_or(serde_json::json!({}));

    // 内置变量
    let now = Local::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    let datetime_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let mut msg = template.to_string();

    // 替换内置模板变量
    msg = msg.replace("{{date}}", &date_str);
    msg = msg.replace("{{datetime}}", &datetime_str);

    // 替换自定义参数
    if let Some(obj) = params.as_object() {
        for (key, value) in obj {
            let val_str = match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            msg = msg.replace(&format!("{{{{{}}}}}", key), &val_str);
        }
    }

    msg
}

/// 标准化 cron 表达式为 7 字段格式（cron 0.15 要求）
///
/// 5 字段: `min hour dom month dow` → 补为 `* min hour dom month dow *`
/// 6 字段: `sec min hour dom month dow` → 补为 `sec min hour dom month dow *`
/// 7 字段: 原样返回
fn normalize_cron(expr: &str) -> String {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    match parts.len() {
        5 => format!("* {} *", expr),          // +秒(wildcard) +年
        6 => format!("{} *", expr),             // +年
        7 => expr.to_string(),
        _ => expr.to_string(),                  // 未知格式，让 Schedule 去报错
    }
}
