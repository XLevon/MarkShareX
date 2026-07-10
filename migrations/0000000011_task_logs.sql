CREATE TABLE IF NOT EXISTS ai_task_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'completed',
    steps TEXT NOT NULL DEFAULT '[]',
    final_reply TEXT NOT NULL DEFAULT '',
    error TEXT,
    created_at DATETIME NOT NULL DEFAULT (datetime('now','localtime')),
    FOREIGN KEY (task_id) REFERENCES ai_tasks(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_ai_task_logs_task_id ON ai_task_logs(task_id);
