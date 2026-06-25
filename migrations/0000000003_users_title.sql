-- 用户表增加 title（抬头）字段，用于替代前台"站点管理员"硬编码
ALTER TABLE users ADD COLUMN title VARCHAR;
