-- Add name column to ai_tasks
ALTER TABLE ai_tasks ADD COLUMN name TEXT NOT NULL DEFAULT '';
