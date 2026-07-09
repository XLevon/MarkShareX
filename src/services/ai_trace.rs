//! AI 任务执行追踪缓存 — 支持前端轮询动态刷新

use std::collections::HashMap;
use std::sync::Mutex;

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
