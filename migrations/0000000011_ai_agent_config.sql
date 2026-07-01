-- AI Agent 配置（单例：一个站点一个 Agent 提示词）
CREATE TABLE IF NOT EXISTS ai_agent_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- 只允许一行
    system_prompt TEXT NOT NULL DEFAULT '',
    user_prompt TEXT NOT NULL DEFAULT '',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 插入默认配置（幂等，已存在则忽略）
INSERT OR IGNORE INTO ai_agent_config (id, system_prompt, user_prompt) VALUES (1, '', '');
