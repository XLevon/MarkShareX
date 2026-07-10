-- 为 ai_tasks 表新增 max_tool_rounds 字段
-- 任务级工具调用轮次限制，优先级最高（NULL=使用全局默认）
ALTER TABLE ai_tasks ADD COLUMN max_tool_rounds INTEGER;
