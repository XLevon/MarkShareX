use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;
use serde::Serialize;
use tracing::field::{Field, Visit};
use tracing::Subscriber;
use tracing_subscriber::Layer;

pub(crate) static START_TIME: OnceLock<Instant> = OnceLock::new();

/// Record process startup time without exposing the global cell to callers.
pub fn mark_started() {
    let _ = START_TIME.set(Instant::now());
}

/// 返回进程启动后的秒数
pub fn uptime_seconds() -> u64 {
    START_TIME
        .get()
        .map(|t| t.elapsed().as_secs())
        .unwrap_or(0)
}

/// 单条日志记录
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
}

/// 内存环形日志缓冲区
pub struct LogBuffer {
    entries: Mutex<VecDeque<LogEntry>>,
    max_entries: usize,
}

impl LogBuffer {
    pub fn new(max_entries: usize) -> Arc<Self> {
        Arc::new(Self {
            entries: Mutex::new(VecDeque::with_capacity(max_entries)),
            max_entries,
        })
    }

    pub fn push(&self, entry: LogEntry) {
        let mut entries = self.entries.lock().unwrap();
        if entries.len() >= self.max_entries {
            entries.pop_front();
        }
        entries.push_back(entry);
    }

    /// 按条件获取日志
    pub fn query(
        &self,
        level_filter: Option<&str>,
        limit: usize,
        search: Option<&str>,
    ) -> Vec<LogEntry> {
        let entries = self.entries.lock().unwrap();
        let iter: Box<dyn Iterator<Item = &LogEntry>> = Box::new(entries.iter().rev());

        iter.filter(|e| {
            if let Some(lvl) = level_filter {
                if !e.level.eq_ignore_ascii_case(lvl) {
                    return false;
                }
            }
            if let Some(q) = search {
                if !e.message.to_lowercase().contains(&q.to_lowercase())
                    && !e.target.to_lowercase().contains(&q.to_lowercase())
                {
                    return false;
                }
            }
            true
        })
        .take(limit)
        .cloned()
        .collect()
    }

    /// 获取指定级别的最近 N 条错误/警告
    pub fn recent_errors(&self, limit: usize) -> Vec<LogEntry> {
        let entries = self.entries.lock().unwrap();
        entries
            .iter()
            .rev()
            .filter(|e| e.level == "ERROR" || e.level == "WARN")
            .take(limit)
            .cloned()
            .collect()
    }
}

// ── Tracing Layer：捕获事件写入 LogBuffer ──

pub struct LogCaptureLayer {
    buffer: Arc<LogBuffer>,
}

impl LogCaptureLayer {
    pub fn new(buffer: Arc<LogBuffer>) -> Self {
        Self { buffer }
    }
}

impl<S> Layer<S> for LogCaptureLayer
where
    S: Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let mut visitor = LogVisitor::default();
        event.record(&mut visitor);

        self.buffer.push(LogEntry {
            timestamp: chrono::Local::now()
                .format("%Y-%m-%dT%H:%M:%S%.3f")
                .to_string(),
            level: metadata.level().to_string(),
            target: metadata.target().to_string(),
            message: visitor.message,
        });
    }
}

// ── Field Visitor ──

#[derive(Default)]
struct LogVisitor {
    message: String,
}

impl Visit for LogVisitor {
    fn record_debug(&mut self, _field: &Field, value: &dyn fmt::Debug) {
        self.message = format!("{:?}", value);
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }
}
