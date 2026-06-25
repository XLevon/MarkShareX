-- 为已有数据库补充 categories.is_visible 列
ALTER TABLE categories ADD COLUMN is_visible BOOLEAN NOT NULL DEFAULT 1;
