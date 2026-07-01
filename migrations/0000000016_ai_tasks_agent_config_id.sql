-- ai_tasks 新增 agent_config_id（可选，NULL 则 fallback 到默认配置）
ALTER TABLE ai_tasks ADD COLUMN agent_config_id INTEGER REFERENCES ai_agent_config(id) ON DELETE SET NULL;
