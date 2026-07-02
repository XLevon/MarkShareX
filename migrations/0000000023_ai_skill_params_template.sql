-- 为 ai_skills 表添加 params_template 字段，存储技能参数 JSON 模板
ALTER TABLE ai_skills ADD COLUMN params_template TEXT NOT NULL DEFAULT '{}';
