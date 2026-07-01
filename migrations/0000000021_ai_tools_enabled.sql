-- Add enabled column to ai_tools
ALTER TABLE ai_tools ADD COLUMN enabled INTEGER NOT NULL DEFAULT 1;
