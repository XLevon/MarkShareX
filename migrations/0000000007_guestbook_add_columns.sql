-- Add missing columns to guestbook table (PRD fix)
-- 0000000000_init_schema.sql had old schema without these columns
-- 0000000005_merged.sql used CREATE TABLE IF NOT EXISTS which didn't alter existing tables

ALTER TABLE guestbook ADD COLUMN email TEXT DEFAULT '';
ALTER TABLE guestbook ADD COLUMN user_id INTEGER DEFAULT NULL;
ALTER TABLE guestbook ADD COLUMN content_html TEXT DEFAULT '';
ALTER TABLE guestbook ADD COLUMN status TEXT NOT NULL DEFAULT 'approved';
ALTER TABLE guestbook ADD COLUMN deleted_at DATETIME DEFAULT NULL;
