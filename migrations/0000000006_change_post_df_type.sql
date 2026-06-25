-- 修改 article_type 默认值：ai_organized → space
-- SQLite 不支持 ALTER COLUMN SET DEFAULT，需重建表
-- 注意：迁移引擎自带事务，勿在此文件内写 BEGIN/COMMIT

CREATE TABLE posts_new (
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

INSERT INTO posts_new SELECT
    id, user_id, category_id, title, slug, summary, content, content_html,
    cover_image, cover_image_url, cover_image_filename, cover_network_id,
    status, post_type, is_pinned, allow_comment, sort_order,
    COALESCE(view_count, 0), COALESCE(like_count, 0), COALESCE(comment_count, 0),
    published_at, deleted_at, created_at, updated_at,
    article_type, article_status
FROM posts;

ALTER TABLE posts RENAME TO posts_bak260529;
ALTER TABLE posts_new RENAME TO posts;

-- 重建索引
CREATE INDEX IF NOT EXISTS idx_posts_user_id ON posts(user_id);
CREATE INDEX IF NOT EXISTS idx_posts_category_id ON posts(category_id);
CREATE INDEX IF NOT EXISTS idx_posts_status ON posts(status);
CREATE INDEX IF NOT EXISTS idx_posts_slug ON posts(slug);
CREATE INDEX IF NOT EXISTS idx_posts_published_at ON posts(published_at);


-- 修复 post_tags 外键引用
-- 迁移 0000000006 重建 posts 表后，post_tags 的 FK 被 SQLite 自动指向了
-- 备份表 posts_bak26052 而非活跃的 posts 表，导致新文章无法关联标签。
-- 方案：重建 post_tags 表，FK 指向正确的 posts(id)。
CREATE TABLE post_tags_new (
    post_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (post_id, tag_id),
    FOREIGN KEY (post_id) REFERENCES posts(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);

INSERT INTO post_tags_new SELECT * FROM post_tags;

DROP TABLE post_tags;

ALTER TABLE post_tags_new RENAME TO post_tags;

CREATE INDEX IF NOT EXISTS idx_post_tags_tag ON post_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_post_tags_post ON post_tags(post_id);
