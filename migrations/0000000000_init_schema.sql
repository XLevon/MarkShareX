-- ============================================================
-- MarkShareX 初始化数据库脚本（全新安装 + 幂等启动执行）
-- 每次启动由 models::run_migrations() 执行
-- 所有 DDL 使用 IF NOT EXISTS，数据插入使用 NOT EXISTS 守卫
-- ============================================================

-- ════════════════════════════════════════════════════════════
--  表结构（含所有字段，无 ALTER TABLE 遗留）
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

-- 5. 文章-标签关联表
CREATE TABLE IF NOT EXISTS post_tags (
    post_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (post_id, tag_id),
    FOREIGN KEY (post_id) REFERENCES posts(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);

-- 6. 文件表（无 user_id FK）
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
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

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

-- ════════════════════════════════════════════════════════════
--  性能索引
-- ════════════════════════════════════════════════════════════

CREATE INDEX IF NOT EXISTS idx_posts_status ON posts(status);
CREATE INDEX IF NOT EXISTS idx_posts_category ON posts(category_id);
CREATE INDEX IF NOT EXISTS idx_posts_deleted ON posts(deleted_at);
CREATE INDEX IF NOT EXISTS idx_posts_published ON posts(status, published_at);
CREATE INDEX IF NOT EXISTS idx_post_tags_tag ON post_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_post_tags_post ON post_tags(post_id);
CREATE INDEX IF NOT EXISTS idx_files_deleted ON files(deleted_at);
CREATE INDEX IF NOT EXISTS idx_tags_deleted ON tags(deleted_at);
CREATE INDEX IF NOT EXISTS idx_categories_deleted ON categories(deleted_at);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user ON refresh_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_token ON refresh_tokens(token);
CREATE INDEX IF NOT EXISTS idx_comments_post ON comments(post_id);
CREATE INDEX IF NOT EXISTS idx_comments_status ON comments(status);
CREATE INDEX IF NOT EXISTS idx_comments_deleted ON comments(deleted_at);

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

-- ═══════════════════════════════════════  
--  留言板
-- ═══════════════════════════════════════

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