-- 先清理重复的工具记录（保留 ID 最小的）
DELETE FROM ai_tools WHERE id NOT IN (
    SELECT MIN(id) FROM ai_tools GROUP BY function_name
);
-- 再加唯一约束防止未来重复
CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_tools_function_name ON ai_tools(function_name);
