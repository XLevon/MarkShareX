-- 合并迁移：文章类型/状态表 + 留言板表 + 系统设置
-- 原 20260528000003_article_types_statuses.sql + 20260528000005_guestbook.sql

-- ============================================
-- 文章类型 & 文章状态基础数据表
-- ============================================
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

-- ============================================
-- 留言板
-- ============================================
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

-- ============================================
-- 系统设置默认值
-- ============================================
INSERT OR IGNORE INTO settings (key, value, updated_at) 
VALUES ('guestbook_enabled', 'false', datetime('now'));

INSERT OR IGNORE INTO settings (key, value, updated_at) 
VALUES ('batch_load_size', '5', datetime('now'));

INSERT OR IGNORE INTO settings (key, value, updated_at) 
VALUES ('scroll_load_size', '3', datetime('now'));
