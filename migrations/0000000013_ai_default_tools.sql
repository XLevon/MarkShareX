-- 预置 AI 内置工具（web_search, web_extract, create_news）
INSERT OR IGNORE INTO ai_tools (name, description, function_name, parameters_schema, created_at, updated_at)
VALUES
(
    '网络搜索',
    '搜索网络资讯，返回标题、URL 和摘要。适合查找最新新闻、技术动态等。',
    'web_search',
    '{"type":"object","properties":{"query":{"type":"string","description":"搜索关键词"},"limit":{"type":"integer","description":"返回结果数量，默认 5，最大 10","default":5}},"required":["query"]}',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
),
(
    '网页抓取',
    '抓取指定 URL 的网页内容，返回 Markdown 格式正文。适合获取文章全文。',
    'web_extract',
    '{"type":"object","properties":{"urls":{"type":"array","items":{"type":"string"},"description":"要抓取的 URL 列表，最多 5 个","maxItems":5}},"required":["urls"]}',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
),
(
    '创建资讯',
    '创建一条资讯草稿。需要提供 title（标题）、summary（摘要）、content（Markdown 正文）。可选 source_url（来源链接）。',
    'create_news',
    '{"type":"object","properties":{"title":{"type":"string","description":"资讯标题"},"summary":{"type":"string","description":"简短摘要，200字以内"},"content":{"type":"string","description":"Markdown 格式正文"},"source_url":{"type":"string","description":"原文链接（可选）"}},"required":["title","summary","content"]}',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);
