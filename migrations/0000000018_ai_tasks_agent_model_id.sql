-- ai_tasks 和 ai_agent_config 新增 model_id
ALTER TABLE ai_tasks ADD COLUMN model_id INTEGER REFERENCES ai_models(id) ON DELETE SET NULL;
ALTER TABLE ai_agent_config ADD COLUMN model_id INTEGER REFERENCES ai_models(id) ON DELETE SET NULL;
