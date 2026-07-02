-- 为 ai_tools 表添加 config 字段（JSON 格式的结构化配置，会附加到工具描述传给 LLM）
ALTER TABLE ai_tools ADD COLUMN config TEXT NOT NULL DEFAULT '{}';
