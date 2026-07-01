-- 将 ai_agent_config 从单行改为多行（去掉 CHECK id=1）
-- SQLite 不支持 ALTER TABLE DROP CHECK，需重建表

CREATE TABLE ai_agent_config_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(100) NOT NULL DEFAULT '默认配置',
    system_prompt TEXT NOT NULL DEFAULT '',
    user_prompt TEXT NOT NULL DEFAULT '',
    is_default BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 迁移现有数据（如果有的话），自动设为默认
INSERT INTO ai_agent_config_new (id, name, system_prompt, user_prompt, is_default, created_at, updated_at)
SELECT id, '默认配置', system_prompt, user_prompt, 1, created_at, updated_at
FROM ai_agent_config;

DROP TABLE ai_agent_config;
ALTER TABLE ai_agent_config_new RENAME TO ai_agent_config;
