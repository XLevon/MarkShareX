-- 迁移 ai_skills 表：合并 system_prompt + user_prompt 为 content 字段
-- SQLite 不支持 DROP COLUMN，需重建表

-- 1. 创建新表
CREATE TABLE ai_skills_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(200) NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL DEFAULT '',
    output_format VARCHAR(50) NOT NULL DEFAULT 'markdown',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 2. 迁移数据：将 system_prompt 和 user_prompt 合并到 content
--    格式保持为 Markdown，在两个 prompt 之间加分隔线
INSERT INTO ai_skills_new (id, name, description, content, output_format, created_at, updated_at)
SELECT
    id, name, description,
    CASE
        WHEN system_prompt != '' AND user_prompt != '' THEN system_prompt || char(10) || char(10) || '---' || char(10) || char(10) || user_prompt
        WHEN system_prompt != '' THEN system_prompt
        ELSE user_prompt
    END,
    output_format, created_at, updated_at
FROM ai_skills;

-- 3. 删除旧表
DROP TABLE ai_skills;

-- 4. 重命名新表
ALTER TABLE ai_skills_new RENAME TO ai_skills;
