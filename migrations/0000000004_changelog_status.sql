-- 给 changelog 表添加 status 字段 (draft | published)
ALTER TABLE changelog ADD COLUMN status TEXT NOT NULL DEFAULT 'published';
