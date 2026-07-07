-- ════════════════════════════════════════════════════════════════
--  合并迁移: 0000000008 ~ 0000000024 → 1 个文件
--  仅包含 init_schema 中不存在的表和数据
--  全部操作幂等（IF NOT EXISTS / INSERT OR IGNORE）
-- ════════════════════════════════════════════════════════════════

-- ── 1. ai_agent_config ──
CREATE TABLE IF NOT EXISTS ai_agent_config (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(100) NOT NULL DEFAULT '默认配置',
    system_prompt TEXT NOT NULL DEFAULT '',
    user_prompt TEXT NOT NULL DEFAULT '',
    is_default BOOLEAN NOT NULL DEFAULT 0,
    model_id INTEGER REFERENCES ai_models(id) ON DELETE SET NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT OR IGNORE INTO ai_agent_config (id, name, system_prompt, user_prompt, is_default)
VALUES (1, '默认配置', '', '', 1);

-- ── 2. ai_tools ──
CREATE TABLE IF NOT EXISTS ai_tools (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(200) NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    function_name VARCHAR(200) NOT NULL DEFAULT '',
    parameters_schema TEXT NOT NULL DEFAULT '{}',
    enabled INTEGER NOT NULL DEFAULT 1,
    config TEXT NOT NULL DEFAULT '{}',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_tools_function_name ON ai_tools(function_name);

INSERT OR IGNORE INTO ai_tools (name, description, function_name, parameters_schema, created_at, updated_at)
VALUES
('获取当前日期时间',
 '获取当前日期和时间（含星期），返回服务器本地时间（CST/UTC+8）。无需参数。',
 'get_current_datetime',
 '{"type":"object","properties":{}}',
 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('站内 API 请求',
 '搜索站内资源或调用外部 API 并返回超链接。用于调用站内搜索（GET /api/v1/search?q=关键词），或查询分类、标签、用户等。返回 JSON 结果，你自行解析并整理为可读格式，以 Markdown 超链接呈现。也可调用外部接口。',
 'api_request',
 '{"type":"object","properties":{"url":{"type":"string","description":"请求 URL"},"method":{"type":"string","description":"HTTP 方法：GET 或 POST","enum":["GET","POST"],"default":"GET"},"body":{"type":"string","description":"POST 请求体（JSON 字符串），仅 method=POST 时使用"}},"required":["url"]}',
 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('网络搜索',
 '搜索网络资讯，返回标题、URL 和摘要。适合查找最新新闻、技术动态等。',
 'web_search',
 '{"type":"object","properties":{"query":{"type":"string","description":"搜索关键词"},"limit":{"type":"integer","description":"返回结果数量，默认 5，最大 10","default":5}},"required":["query"]}',
 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('网页抓取',
 '抓取指定 URL 的网页内容，返回 Markdown 格式正文。适合获取文章全文。',
 'web_extract',
 '{"type":"object","properties":{"urls":{"type":"array","items":{"type":"string"},"description":"要抓取的 URL 列表，最多 5 个","maxItems":5}},"required":["urls"]}',
 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('创建资讯',
 '创建一条资讯草稿。需要提供 title（标题）、summary（摘要）、content（Markdown 正文）。可选 source_url（来源链接）。',
 'create_news',
 '{"type":"object","properties":{"title":{"type":"string","description":"资讯标题"},"summary":{"type":"string","description":"简短摘要，200字以内"},"content":{"type":"string","description":"Markdown 格式正文"},"source_url":{"type":"string","description":"原文链接（可选）"}},"required":["title","summary","content"]}',
 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('创建文章',
 '创建一篇知识库文章。需要提供 title（标题）、content（Markdown 正文，可用 nr:ID 引用资源库图片）、category_id（分类 ID）。可选 summary、cover_image（nr:ID 或 URL）、status（draft 或 published，默认 draft）。',
 'create_post',
 '{"type":"object","properties":{"title":{"type":"string","description":"文章标题"},"content":{"type":"string","description":"Markdown 格式正文。图片可用 nr:ID 引用资源库图片"},"category_id":{"type":"integer","description":"分类 ID（必填，需先查询分类获取）"},"summary":{"type":"string","description":"文章摘要（可选）"},"cover_image":{"type":"string","description":"封面图：nr:资源ID 或完整 URL（可选）"},"status":{"type":"string","description":"发布状态：draft（草稿，默认）或 published（已发布）","enum":["draft","published"],"default":"draft"}},"required":["title","content","category_id"]}',
 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);
