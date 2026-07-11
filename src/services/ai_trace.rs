//! AI 任务执行追踪 — 内存缓存 + 持久化到 DB

use std::collections::HashMap;
use std::sync::Mutex;
use sea_orm::{DatabaseConnection, Set, ActiveModelTrait};

#[derive(Clone, serde::Serialize)]
pub struct TraceEntry {
    pub status: String,
    pub steps: Vec<crate::services::ai_chat::TraceStep>,
    pub final_reply: String,
    pub error: Option<String>,
}

use std::sync::LazyLock;
static CACHE: LazyLock<Mutex<HashMap<i32, TraceEntry>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn trace_start(task_id: i32) {
    CACHE.lock().unwrap().insert(task_id, TraceEntry {
        status: "running".to_string(), steps: vec![], final_reply: String::new(), error: None,
    });
}

pub fn trace_add_step(task_id: i32, step: crate::services::ai_chat::TraceStep) {
    if let Some(e) = CACHE.lock().unwrap().get_mut(&task_id) { e.steps.push(step); }
}

pub fn trace_complete(task_id: i32, final_reply: String) {
    if let Some(e) = CACHE.lock().unwrap().get_mut(&task_id) {
        e.status = "completed".to_string(); e.final_reply = final_reply;
    }
}

pub fn trace_fail(task_id: i32, error: String) {
    if let Some(e) = CACHE.lock().unwrap().get_mut(&task_id) {
        e.status = "failed".to_string(); e.error = Some(error);
    }
}

pub fn trace_get(task_id: i32) -> Option<TraceEntry> {
    CACHE.lock().unwrap().get(&task_id).cloned()
}

#[allow(dead_code)]
pub fn trace_remove(task_id: i32) {
    CACHE.lock().unwrap().remove(&task_id);
}

/// 将缓存中的 trace 持久化到 ai_task_logs 表，然后清除缓存
pub async fn trace_persist(db: &DatabaseConnection, task_id: i32) {
    let entry = match CACHE.lock().unwrap().remove(&task_id) {
        Some(e) => e,
        None => {
            tracing::warn!("trace_persist: 任务 #{} 缓存已不存在，可能被重复持久化", task_id);
            return;
        }
    };

    use crate::models::entity::ai_task_log;
    let steps_json = serde_json::to_string(&entry.steps).unwrap_or_else(|_| "[]".to_string());

    let now = crate::utils::now_local();
    let model = ai_task_log::ActiveModel {
        task_id: Set(task_id),
        status: Set(entry.status),
        steps: Set(steps_json),
        final_reply: Set(entry.final_reply),
        error: Set(entry.error),
        created_at: Set(now),
        ..Default::default()
    };

    if let Err(e) = model.insert(db).await {
        tracing::error!("trace_persist: 持久化任务 #{} 失败: {}", task_id, e);
    }
}
