-- 迁移 0000000011：新增访客文章复制和右键权限设置
-- 默认开启；已有显式配置时保持原值

INSERT OR IGNORE INTO settings (key, value, updated_at)
VALUES ('guest_copy_enabled', 'true', datetime('now'));
