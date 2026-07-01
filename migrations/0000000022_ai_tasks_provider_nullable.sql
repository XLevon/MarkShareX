-- Make ai_tasks.provider_id nullable
-- SQLite doesn't support ALTER COLUMN DROP NOT NULL, must recreate table

-- Step 1: Create new table with nullable provider_id (columns in original order)
CREATE TABLE IF NOT EXISTS ai_tasks_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL DEFAULT '',
    skill_id INTEGER NOT NULL,
    provider_id INTEGER,
    cron_expr VARCHAR(100) NOT NULL DEFAULT '',
    params TEXT NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT 1,
    last_run_at DATETIME,
    run_count INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    agent_config_id INTEGER REFERENCES ai_agent_config(id) ON DELETE SET NULL,
    model_id INTEGER REFERENCES ai_models(id) ON DELETE SET NULL,
    FOREIGN KEY (skill_id) REFERENCES ai_skills(id)
);

-- Step 2: Copy data with explicit column list (not SELECT *)
INSERT INTO ai_tasks_new (id, name, skill_id, provider_id, cron_expr, params, enabled,
    last_run_at, run_count, created_at, updated_at, agent_config_id, model_id)
SELECT id, name, skill_id, provider_id, cron_expr, params, enabled,
    last_run_at, run_count, created_at, updated_at, agent_config_id, model_id
FROM ai_tasks;

-- Step 3: Drop old table
DROP TABLE ai_tasks;

-- Step 4: Rename
ALTER TABLE ai_tasks_new RENAME TO ai_tasks;
