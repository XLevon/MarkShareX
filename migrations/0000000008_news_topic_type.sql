-- 资讯添加题材类型字段
ALTER TABLE news ADD COLUMN topic_type VARCHAR(20) NOT NULL DEFAULT '';
CREATE INDEX IF NOT EXISTS idx_news_topic_type ON news(topic_type);
