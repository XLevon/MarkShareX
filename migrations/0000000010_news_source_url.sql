-- 迁移 0000000010：news 表新增 source_url 列 + 去重索引
-- 用于同一文章来源去重

ALTER TABLE news ADD COLUMN source_url VARCHAR(1000) NOT NULL DEFAULT '';

CREATE INDEX IF NOT EXISTS idx_news_source_url ON news(source_url);
