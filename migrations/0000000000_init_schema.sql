-- ============================================================
-- MarkShareX 初始化数据库脚本（全新安装 + 幂等启动执行）
-- 每次启动由 models::run_migrations() 执行
-- 所有 DDL 使用 IF NOT EXISTS，数据插入使用 NOT EXISTS 守卫
-- ============================================================

-- ════════════════════════════════════════════════════════════
--  v0.1.0 表结构
-- ════════════════════════════════════════════════════════════

-- 1. 用户表
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username VARCHAR NOT NULL UNIQUE,
    email VARCHAR NOT NULL UNIQUE,
    password_hash VARCHAR NOT NULL,
    display_name VARCHAR,
    avatar_url VARCHAR,
    role VARCHAR NOT NULL DEFAULT 'visitor',
    status VARCHAR NOT NULL DEFAULT 'active',
    title VARCHAR,
    bio TEXT,
    api_key VARCHAR,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    last_login_at DATETIME,
    deleted_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 2. 分类表
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL,
    slug VARCHAR NOT NULL UNIQUE,
    description TEXT,
    parent_id INTEGER,
    sort_order INTEGER NOT NULL DEFAULT 0,
    image_url VARCHAR,
    image_filename TEXT,
    network_resource_id INTEGER,
    is_visible BOOLEAN NOT NULL DEFAULT 1,
    user_id INTEGER REFERENCES users(id),
    deleted_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_categories_deleted ON categories(deleted_at);

-- 3. 标签表
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL,
    slug VARCHAR NOT NULL UNIQUE,
    user_id INTEGER REFERENCES users(id),
    deleted_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_tags_deleted ON tags(deleted_at);

-- 4. 文章表
CREATE TABLE IF NOT EXISTS posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    category_id INTEGER,
    title VARCHAR NOT NULL,
    slug VARCHAR NOT NULL UNIQUE,
    summary TEXT,
    content TEXT,
    content_html TEXT,
    cover_image VARCHAR,
    cover_image_url VARCHAR,
    cover_image_filename TEXT,
    cover_network_id INTEGER,
    status VARCHAR NOT NULL DEFAULT 'draft',
    post_type VARCHAR NOT NULL DEFAULT 'post',
    is_pinned BOOLEAN NOT NULL DEFAULT 0,
    allow_comment BOOLEAN NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    view_count INTEGER NOT NULL DEFAULT 0,
    like_count INTEGER NOT NULL DEFAULT 0,
    comment_count INTEGER NOT NULL DEFAULT 0,
    published_at DATETIME,
    deleted_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    article_type VARCHAR NOT NULL DEFAULT 'space',
    article_status VARCHAR NOT NULL DEFAULT 'space',
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (category_id) REFERENCES categories(id)
);
CREATE INDEX IF NOT EXISTS idx_posts_status ON posts(status);
CREATE INDEX IF NOT EXISTS idx_posts_category ON posts(category_id);
CREATE INDEX IF NOT EXISTS idx_posts_deleted ON posts(deleted_at);
CREATE INDEX IF NOT EXISTS idx_posts_published ON posts(status, published_at);

-- 5. 文章-标签关联表
CREATE TABLE IF NOT EXISTS post_tags (
    post_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (post_id, tag_id),
    FOREIGN KEY (post_id) REFERENCES posts(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);
CREATE INDEX IF NOT EXISTS idx_post_tags_tag ON post_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_post_tags_post ON post_tags(post_id);

-- 6. 文件表
CREATE TABLE IF NOT EXISTS files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    filename VARCHAR NOT NULL,
    original_name VARCHAR NOT NULL,
    mime_type VARCHAR NOT NULL,
    size INTEGER NOT NULL,
    storage_path VARCHAR,
    url VARCHAR,
    md5_hash VARCHAR,
    deleted_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
CREATE INDEX IF NOT EXISTS idx_files_deleted ON files(deleted_at);

-- 7. 系统设置表
CREATE TABLE IF NOT EXISTS settings (
    key VARCHAR PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 8. 刷新令牌表
CREATE TABLE IF NOT EXISTS refresh_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    token VARCHAR NOT NULL UNIQUE,
    expires_at DATETIME NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user ON refresh_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_token ON refresh_tokens(token);

-- 9. 点赞表
CREATE TABLE IF NOT EXISTS likes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    post_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, post_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (post_id) REFERENCES posts(id)
);

-- 10. 评论表
CREATE TABLE IF NOT EXISTS comments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    post_id INTEGER NOT NULL,
    user_id INTEGER,
    parent_id INTEGER,
    author_name VARCHAR NOT NULL,
    author_email VARCHAR,
    content TEXT NOT NULL,
    content_html TEXT NOT NULL,
    status VARCHAR NOT NULL DEFAULT 'approved',
    like_count INTEGER NOT NULL DEFAULT 0,
    ip_address VARCHAR,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME,
    FOREIGN KEY (post_id) REFERENCES posts(id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (parent_id) REFERENCES comments(id)
);
CREATE INDEX IF NOT EXISTS idx_comments_post ON comments(post_id);
CREATE INDEX IF NOT EXISTS idx_comments_status ON comments(status);
CREATE INDEX IF NOT EXISTS idx_comments_deleted ON comments(deleted_at);

-- 11. 作者申请表
CREATE TABLE IF NOT EXISTS author_applications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL UNIQUE,
    reason TEXT NOT NULL,
    content_description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'pending',
    admin_remark TEXT,
    reviewed_by INTEGER,
    reviewed_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);


-- ══════════════════════════════════════════════
--  网络资源（network_resources）
-- ══════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS network_resources (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    url         TEXT NOT NULL,
    label       TEXT,
    source_type VARCHAR NOT NULL DEFAULT 'image',
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_network_resources_url ON network_resources(url);

-- ══════════════════════════════════════════════
--  登录日志（login_logs）
-- ══════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS login_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    username VARCHAR NOT NULL,
    ip_address VARCHAR,
    user_agent VARCHAR,
    device_type VARCHAR,
    login_method VARCHAR NOT NULL DEFAULT 'password',
    success BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_login_logs_user ON login_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_login_logs_created ON login_logs(created_at);

-- ══════════════════════════════════════════════
--  阅读日志（read_logs）
-- ══════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS read_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    post_id INTEGER NOT NULL,
    user_id INTEGER,
    ip_address VARCHAR,
    user_agent VARCHAR,
    device_type VARCHAR,
    referrer VARCHAR,
    duration_seconds INTEGER DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (post_id) REFERENCES posts(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);
CREATE INDEX IF NOT EXISTS idx_read_logs_post ON read_logs(post_id);
CREATE INDEX IF NOT EXISTS idx_read_logs_user ON read_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_read_logs_created ON read_logs(created_at);

-- ══════════════════════════════════════════════
--  版本更新说明（changelog）
-- ══════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS changelog (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    version TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'published',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ════════════════════════════════════════════════════════════
--  文章类型 & 状态标签基础数据表（知识库）
-- ════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS article_types (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code VARCHAR(32) NOT NULL UNIQUE,
    display_name VARCHAR(64) NOT NULL,
    color VARCHAR(32) NOT NULL DEFAULT '#6b7280',
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS article_statuses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code VARCHAR(32) NOT NULL UNIQUE,
    display_name VARCHAR(64) NOT NULL,
    color VARCHAR(32) NOT NULL DEFAULT '#6b7280',
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO article_types (code, display_name, color, sort_order) VALUES
    ('space',              '',           '#6b7280', 0),
    ('ai_organized',       '🤖 AI整理',   '#a855f7', 1),
    ('original',           '📝 原创',     '#3b82f6', 2),
    ('tutorial',           '📚 教程',     '#22c55e', 3),
    ('repost',             '🔗 转载',     '#fb923c', 4),
    ('translation',        '🌐 翻译',     '#fb923c', 5),
    ('opinion_essay',      '💡 随笔',     '#ec4899', 6);

INSERT OR IGNORE INTO article_statuses (code, display_name, color, sort_order) VALUES
    ('space',                  '',           '#6b7280', 1),
    ('latest',                 '✅ 最新',     '#22c55e', 2);

 
--  留言板
CREATE TABLE IF NOT EXISTS guestbook (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nickname TEXT NOT NULL,
    email TEXT DEFAULT '',
    user_id INTEGER DEFAULT NULL,
    content TEXT NOT NULL,
    content_html TEXT DEFAULT '',
    status TEXT NOT NULL DEFAULT 'approved',
    reply TEXT DEFAULT NULL,
    is_replied BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at DATETIME NOT NULL DEFAULT (datetime('now', 'localtime')),
    deleted_at DATETIME DEFAULT NULL
);

-- ════════════════════════════════════════════════════════════
--  迁移追踪表（migrations.rs 增量迁移使用）
-- ════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS _migrations (
    name TEXT PRIMARY KEY,
    executed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ════════════════════════════════════════════════════════════
-- v0.4.0 版本新增的表 - 资讯模块
-- ════════════════════════════════════════════════════════════
-- 19. 咨询信息表
CREATE TABLE IF NOT EXISTS news (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title VARCHAR(255) NOT NULL DEFAULT '',
    summary VARCHAR(500) NOT NULL DEFAULT '',
    content TEXT NOT NULL DEFAULT '',
    content_html TEXT NOT NULL DEFAULT '',
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    topic_type VARCHAR(20) NOT NULL DEFAULT '',
    source_url VARCHAR(1000) NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0,
    published_at DATETIME,
    user_id INTEGER,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_news_status ON news(status);
CREATE INDEX IF NOT EXISTS idx_news_topic_type ON news(topic_type);
CREATE INDEX IF NOT EXISTS idx_news_sort_order ON news(sort_order);
CREATE INDEX IF NOT EXISTS idx_news_created_at ON news(created_at);
CREATE INDEX IF NOT EXISTS idx_news_source_url ON news(source_url);

-- ════════════════════════════════════════════════════════════
-- v0.4.0 版本新增的表 - AI 模块
-- ════════════════════════════════════════════════════════════
-- 20. AI 模型供应商
CREATE TABLE IF NOT EXISTS ai_providers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(100) NOT NULL DEFAULT '',
    provider_type VARCHAR(50) NOT NULL DEFAULT 'openai',
    base_url VARCHAR(500) NOT NULL DEFAULT '',
    api_key_encrypted TEXT NOT NULL DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- AI 模型（每个供应商下可配置多个模型）
CREATE TABLE IF NOT EXISTS ai_models (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider_id INTEGER NOT NULL,
    name VARCHAR(200) NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (provider_id) REFERENCES ai_providers(id) ON DELETE CASCADE
);

-- AI 工具（Agent 能力注册表）
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
('获取当前日期时间', '获取当前日期和时间（含星期），返回服务器本地时间（CST/UTC+8）。无需参数。', 
    'get_current_datetime', '{"type":"object","properties":{}}', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('站内 API 请求', '搜索站内资源，需携带用户token，返回用户权限范围内的数据结果。常用接口：① GET /api/v1/search?q=关键词（知识文章Tantivy全文搜索）② GET /api/v1/news?search=关键词（资讯标题搜索，可组合 topic_type/date_from/date_to 筛选）③ GET /api/v1/categories（分类列表）④ GET /api/v1/tags（标签列表）。仅支持本站 API 的相对路径。返回 JSON 结果，你自行解析并整理为可读格式。', 
    'api_request', '{{"type":"object","properties":{"url":{"type":"string","description":"请求 URL"},"method":{"type":"string","description":"HTTP 方法：GET 或 POST","enum":["GET","POST"],"default":"GET"},"body":{"type":"string","description":"POST 请求体（JSON 字符串），仅 method=POST 时使用"}},"required":["url"]}', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('网络搜索', '搜索网络资讯，返回标题、URL 和摘要。适合查找最新新闻、技术动态等。', 
    'web_search', '{"type":"object","properties":{"query":{"type":"string","description":"搜索关键词"},"limit":{"type":"integer","description":"返回结果数量，默认 5，最大 10","default":5}},"required":["query"]}', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('网页抓取', '抓取指定 URL 的网页内容，返回 Markdown 格式正文。适合获取文章全文。', 
    'web_extract', '{"type":"object","properties":{"urls":{"type":"array","items":{"type":"string"},"description":"要抓取的 URL 列表，最多 5 个","maxItems":5}},"required":["urls"]}', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('创建资讯', '创建一条资讯。需要提供 title（标题）、summary（摘要）、content（Markdown 正文）、source_url（来源链接）、topic_type（题材类型，可选）、status（draft 草稿 或 published 已发布，默认 draft）。', 
    'create_news', '{"type": "object", "properties": {"title": {"type": "string", "description": "资讯标题"}, "summary": {"type": "string", "description": "简短摘要，200字以内"}, "content": {"type": "string", "description": "Markdown 格式正文"}, "source_url": {"type": "string", "description": "原文链接"}, "status": {"type": "string", "description": "发布状态：draft（草稿，默认）或 published（已发布）", "enum": ["draft", "published"], "default": "draft"}, "topic_type": {"type": "string", "description": "题材类型，为空则不分类。可选值：politics(时政新闻)、finance(财经新闻)、technology(科技新闻)、society(社会新闻)、entertainment(文娱新闻)、sports(体育新闻)、international(国际新闻)、law(法治新闻)、education(教育新闻)", "enum": ["politics", "finance", "technology", "society", "entertainment", "sports", "international", "law", "education"], "default": ""}}, "required": ["title", "summary", "content", "source_url"]}', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('创建文章', '创建一篇知识库文章。需要提供 title（标题）、content（Markdown 正文，支持本地路径/nr:ID 引用资源库/外部URL 三种图片格式）、category_id（分类 ID）。可选 summary、cover_image（nr:ID 或 URL）、status（draft 或 published，默认 draft）。', 
    'create_post', '{"type": "object", "properties": {"title": {"type": "string", "description": "文章标题"}, "content": {"type": "string", "description": "Markdown 格式正文。图片可用 nr:ID 引用资源库图片"}, "category_id": {"type": "integer", "description": "分类 ID（必填，需先查询 categories 获取）"}, "summary": {"type": "string", "description": "文章摘要（可选）"}, "cover_image": {"type": "string", "description": "封面图：nr: 资源 ID 或完整 URL（可选）"}, "status": {"type": "string", "description": "发布状态：draft（草稿，默认）或 published（已发布）", "enum": ["draft", "published"], "default": "draft"}, "article_type": {"type": "string", "description": "文章类型：space、original（原创）、ai_organized（AI 整理）、tutorial（教程）、repost（转载）、translation（翻译）、opinion_essay（随笔）", "enum": ["space", "original", "ai_organized", "tutorial", "repost", "translation", "opinion_essay"], "default": "space"}}, "required": ["title", "content", "category_id"]}', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

-- AI Agent 配置
CREATE TABLE IF NOT EXISTS ai_agent_config (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(100) NOT NULL DEFAULT '',
    system_prompt TEXT NOT NULL DEFAULT '',
    user_prompt TEXT NOT NULL DEFAULT '',
    is_default BOOLEAN NOT NULL DEFAULT 0,
    model_id INTEGER REFERENCES ai_models(id) ON DELETE SET NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
INSERT OR IGNORE INTO ai_agent_config (id, name, system_prompt, user_prompt, is_default)
VALUES (1, ' AI助手', '', '', 0);

-- AI 技能（每个技能对应一个可执行的任务）
CREATE TABLE IF NOT EXISTS ai_skills (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(200) NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL DEFAULT '',
    output_format VARCHAR(50) NOT NULL DEFAULT 'markdown',
    params_template TEXT NOT NULL DEFAULT '{}',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- AI 任务（每个任务对应一个 AI 技能）
CREATE TABLE IF NOT EXISTS ai_tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL DEFAULT '',
    skill_id INTEGER NOT NULL,
    provider_id INTEGER,
    cron_expr VARCHAR(100) NOT NULL DEFAULT '',
    params TEXT NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT 1,
    last_run_at DATETIME,
    run_count INTEGER NOT NULL DEFAULT 0,
    agent_config_id INTEGER,
    model_id INTEGER,
    max_tool_rounds INTEGER,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (skill_id) REFERENCES ai_skills(id) ON DELETE CASCADE,
    FOREIGN KEY (agent_config_id) REFERENCES ai_agent_config(id) ON DELETE SET NULL,
    FOREIGN KEY (model_id) REFERENCES ai_models(id) ON DELETE SET NULL
);

-- AI 任务日志
CREATE TABLE IF NOT EXISTS ai_task_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'completed',
    steps TEXT NOT NULL DEFAULT '[]',
    final_reply TEXT NOT NULL DEFAULT '',
    error TEXT,
    created_at DATETIME NOT NULL DEFAULT (datetime('now','localtime')),
    FOREIGN KEY (task_id) REFERENCES ai_tasks(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_ai_task_logs_task_id ON ai_task_logs(task_id);

-- AI 聊天会话
CREATE TABLE IF NOT EXISTS ai_chat_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    title VARCHAR(200) NOT NULL DEFAULT '新会话',
    agent_config_id INTEGER,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (agent_config_id) REFERENCES ai_agent_config(id) ON DELETE SET NULL
);

-- AI 聊天消息
CREATE TABLE IF NOT EXISTS ai_chat_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'user',
    content TEXT NOT NULL DEFAULT '',
    tool_calls TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES ai_chat_sessions(id) ON DELETE CASCADE
);

-- ════════════════════════════════════════════════════════════
--  默认系统设置（首次启动，key 不存在时插入）
-- ════════════════════════════════════════════════════════════

INSERT OR IGNORE INTO settings (key, value, updated_at) 
VALUES ('site_title', 'Mark-Share-X', datetime('now'));
INSERT OR IGNORE INTO settings (key, value, updated_at) 
VALUES ('site_subtitle', '纯 AI 构建・AI 系统管理・Rust 驱动・轻量高效・Markdown 多人协作博客', datetime('now'));
INSERT OR IGNORE INTO settings (key, value, updated_at) 
VALUES ('site_description', '用 AI Vibe Coding 的方式写代码，用 Rust 的安全效率跑服务，用 Markdown 的纯粹方式写文章，用最开放的架构迎接 AI 时代，利用 AI 来学习 AI、驾驭 AI。', datetime('now'));
INSERT OR IGNORE INTO settings (key, value, updated_at) 
VALUES ('comment_moderation', 'false', datetime('now'));
INSERT OR IGNORE INTO settings (key, value, updated_at) 
VALUES ('friend_links', '[{"name": "MarkShareX 代码库", "url": "https://github.com/XLevon/MarkShareX"}]', datetime('now'));
INSERT OR IGNORE INTO settings (key, value, updated_at) 
VALUES ('site-manager', '', datetime('now'));
INSERT OR IGNORE INTO settings (key, value, updated_at) 
VALUES ('guestbook_enabled', 'false', datetime('now'));
INSERT OR IGNORE INTO settings (key, value, updated_at) 
VALUES ('batch_load_size', '5', datetime('now'));
INSERT OR IGNORE INTO settings (key, value, updated_at) 
VALUES ('scroll_load_size', '3', datetime('now'));
INSERT OR IGNORE INTO settings (key, value, updated_at)
VALUES ('guest_copy_enabled', 'true', datetime('now'));