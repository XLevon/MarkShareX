# MarkShareX 数据库 Schema 文档

> 数据库：SQLite
> ORM：SeaORM (Rust)
> 来源：`migrations/0000000000_init_schema.sql`（DDL） + `src/models/entity/*.rs`（Rust Entity）

---

## 目录

1. [表一览](#1-表一览)
2. [users — 用户表](#2-users--用户表)
3. [posts — 文章表](#3-posts--文章表)
4. [categories — 分类表](#4-categories--分类表)
5. [tags — 标签表](#5-tags--标签表)
6. [post_tags — 文章-标签关联表](#6-post_tags--文章-标签关联表)
7. [files — 文件表](#7-files--文件表)
8. [settings — 系统设置 KV 表](#8-settings--系统设置-kv-表)
9. [refresh_tokens — Refresh Token 表](#9-refresh_tokens--refresh-token-表)
10. [comments — 评论表](#10-comments--评论表)
11. [author_applications — 作者申请表](#11-author_applications--作者申请表)
12. [read_logs — 阅读日志表](#12-read_logs--阅读日志表)
13. [login_logs — 登录日志表](#13-login_logs--登录日志表)
14. [network_resources — 网络资源表](#14-network_resources--网络资源表)
15. [changelog — 版本更新说明表](#15-changelog--版本更新说明表)
16. [likes — 点赞表](#16-likes--点赞表)
17. [_migrations — 迁移追踪表](#17-_migrations--迁移追踪表)
18. [外键约束汇总](#18-外键约束汇总)
19. [Entity 对应说明](#19-entity-对应说明)
20. [软删除策略](#20-软删除策略)

---

## 1. 表一览

| 表名 | 主键 | Rust Entity | 说明 |
|------|------|-------------|------|
| `users` | `id` (INTEGER AUTOINCREMENT) | `users.rs` | 用户 |
| `posts` | `id` (INTEGER AUTOINCREMENT) | `posts.rs` | 文章 |
| `categories` | `id` (INTEGER AUTOINCREMENT) | `categories.rs` | 分类 |
| `tags` | `id` (INTEGER AUTOINCREMENT) | `tags.rs` | 标签 |
| `post_tags` | `(post_id, tag_id)` 复合主键 | `post_tags.rs` | 文章-标签关联 |
| `files` | `id` (INTEGER AUTOINCREMENT) | `files.rs` | 上传文件 |
| `settings` | `key` (VARCHAR PK, 非自增) | `settings.rs` | 系统设置 KV |
| `refresh_tokens` | `id` (INTEGER AUTOINCREMENT) | `refresh_tokens.rs` | JWT Refresh Token |
| `comments` | `id` (INTEGER AUTOINCREMENT) | `comments.rs` | 文章评论 |
| `author_applications` | `id` (INTEGER AUTOINCREMENT) | `author_applications.rs` | 作者申请 |
| `read_logs` | `id` (INTEGER AUTOINCREMENT) | `read_logs.rs` | 阅读日志 |
| `login_logs` | `id` (INTEGER AUTOINCREMENT) | `login_logs.rs` | 登录日志 |
| `network_resources` | `id` (INTEGER AUTOINCREMENT) | `network_resources.rs` | 网络资源引用 |
| `changelog` | `id` (INTEGER AUTOINCREMENT) | `changelog.rs` | 版本更新说明 |
| `likes` | `id` (INTEGER AUTOINCREMENT) | **无 Entity** | 点赞记录 |
| `_migrations` | `name` (TEXT PK) | **无 Entity** | 迁移追踪 |

> **注意**：`likes` 在 SQL 中存在但尚未定义对应的 Rust Entity（`src/models/entity/` 下无对应文件）。

---

## 2. users — 用户表

**SQL DDL（实际）：**

```sql
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
    is_active BOOLEAN NOT NULL DEFAULT 1,
    last_login_at DATETIME,
    deleted_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Rust Entity（`src/models/entity/users.rs`）：**

| 字段 | Rust 类型 | 可为空 | 说明 |
|------|-----------|--------|------|
| `id` | `i32` | ❌ PK | 用户 ID（自增） |
| `username` | `String` | ❌ UNIQUE | 登录用户名 |
| `email` | `String` | ❌ UNIQUE | 邮箱地址 |
| `password_hash` | `String` | ❌ | bcrypt 密码哈希 |
| `display_name` | `Option<String>` | ✅ | 显示名称 |
| `avatar_url` | `Option<String>` | ✅ | 头像 URL |
| `role` | `String` | ❌ (DEFAULT 'visitor') | 角色 |
| `status` | `String` | ❌ (DEFAULT 'active') | 账户状态 |
| `bio` | `Option<String>` | ✅ | 个人简介 |
| `is_active` | `bool` | ❌ (DEFAULT 1) | 是否启用 |
| `last_login_at` | `Option<NaiveDateTime>` | ✅ | 最后登录时间 |
| `deleted_at` | `Option<NaiveDateTime>` | ✅ | 软删除时间 |
| `created_at` | `NaiveDateTime` | ❌ | 创建时间 |
| `updated_at` | `NaiveDateTime` | ❌ | 更新时间 |

**外键**：无（被其他表引用）

---

## 3. posts — 文章表

**SQL DDL（实际）：**

```sql
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
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (category_id) REFERENCES categories(id)
);
```

**Rust Entity（`src/models/entity/posts.rs`）：**

| 字段 | Rust 类型 | 可为空 | 说明 |
|------|-----------|--------|------|
| `id` | `i32` | ❌ PK | 文章 ID（自增） |
| `user_id` | `i32` | ❌ FK→users.id | 作者 ID |
| `category_id` | `Option<i32>` | ✅ FK→categories.id | 分类 ID |
| `title` | `String` | ❌ | 文章标题 |
| `slug` | `String` | ❌ UNIQUE | URL 标识 |
| `summary` | `Option<String>` | ✅ | 摘要 |
| `content` | `Option<String>` | ✅ | Markdown 正文 |
| `content_html` | `Option<String>` | ✅ | 渲染后 HTML |
| `cover_image` | `Option<String>` | ✅ | 封面图 URL |
| `status` | `String` | ❌ (DEFAULT 'draft') | 状态 |
| `post_type` | `String` | ❌ (DEFAULT 'post') | 类型 |
| `is_pinned` | `bool` | ❌ (DEFAULT 0) | 是否置顶 |
| `allow_comment` | `bool` | ❌ (DEFAULT 1) | 是否允许评论 |
| `sort_order` | `i32` | ❌ (DEFAULT 0) | 排序权重 |
| `view_count` | `i32` | ❌ (DEFAULT 0) | 浏览次数（已废弃，API 层改用 read_logs 实时统计） |
| `like_count` | `i32` | ❌ (DEFAULT 0) | 点赞数 |
| `comment_count` | `i32` | ❌ (DEFAULT 0) | 评论数 |
| `published_at` | `Option<NaiveDateTime>` | ✅ | 发布时间 |
| `deleted_at` | `Option<NaiveDateTime>` | ✅ | 软删除时间 |
| `created_at` | `NaiveDateTime` | ❌ | 创建时间 |
| `updated_at` | `NaiveDateTime` | ❌ | 更新时间 |

**外键**：
- `user_id` → `users(id)` — 无 ON DELETE / ON UPDATE 子句（SQLite 默认行为）
- `category_id` → `categories(id)` — 无 ON DELETE / ON UPDATE 子句

**实际索引**（DDL 中定义）：
- `idx_posts_status` ON `(status)`
- `idx_posts_category` ON `(category_id)`
- `idx_posts_deleted` ON `(deleted_at)`
- `idx_posts_published` ON `(status, published_at)`

> **注意**：`view_count` 列已不再由后端维护（旧 `increment_view_count` 已删除）。API 层和 SSR 页面均从 `read_logs` 表实时 `COUNT(*)` 获取阅读量。该列仅保留在表中用于向后兼容。

---

## 4. categories — 分类表

**SQL DDL（实际）：**

```sql
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL,
    slug VARCHAR NOT NULL UNIQUE,
    description TEXT,
    parent_id INTEGER,
    sort_order INTEGER NOT NULL DEFAULT 0,
    image_url VARCHAR,
    image_filename TEXT,
    user_id INTEGER REFERENCES users(id),
    deleted_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Rust Entity（`src/models/entity/categories.rs`）：**

| 字段 | Rust 类型 | 可为空 | 说明 |
|------|-----------|--------|------|
| `id` | `i32` | ❌ PK | 分类 ID（自增） |
| `name` | `String` | ❌ | 分类名称 |
| `slug` | `String` | ❌ UNIQUE | URL 标识 |
| `description` | `Option<String>` | ✅ | 分类描述 |
| `image_url` | `Option<String>` | ✅ | 分类图片 URL |
| `image_filename` | `Option<String>` | ✅ | 图片文件名 |
| `parent_id` | `Option<i32>` | ✅ | 父分类 ID（自引用） |
| `sort_order` | `i32` | ❌ (DEFAULT 0) | 排序权重 |
| `user_id` | `Option<i32>` | ✅ REFERENCES users(id) | 创建者 ID |
| `deleted_at` | `Option<NaiveDateTime>` | ✅ | 软删除时间 |
| `created_at` | `NaiveDateTime` | ❌ | 创建时间 |
| `updated_at` | `NaiveDateTime` | ❌ | 更新时间 |

**外键**：
- `user_id` → `users(id)` — 使用 `REFERENCES` 语法（非独立 FOREIGN KEY 子句）

**实际索引**：
- `idx_categories_deleted` ON `(deleted_at)`

> **注意**：`parent_id` 无 FOREIGN KEY 约束（自引用未声明）。

---

## 5. tags — 标签表

**SQL DDL（实际）：**

```sql
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL,
    slug VARCHAR NOT NULL UNIQUE,
    user_id INTEGER REFERENCES users(id),
    deleted_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Rust Entity（`src/models/entity/tags.rs`）：**

| 字段 | Rust 类型 | 可为空 | 说明 |
|------|-----------|--------|------|
| `id` | `i32` | ❌ PK | 标签 ID（自增） |
| `name` | `String` | ❌ | 标签名称 |
| `slug` | `String` | ❌ UNIQUE | URL 标识 |
| `user_id` | `Option<i32>` | ✅ REFERENCES users(id) | 创建者 ID |
| `deleted_at` | `Option<NaiveDateTime>` | ✅ | 软删除时间 |
| `created_at` | `NaiveDateTime` | ❌ | 创建时间 |
| `updated_at` | `NaiveDateTime` | ❌ | 更新时间 |

**外键**：
- `user_id` → `users(id)` — 使用 `REFERENCES` 语法

**实际索引**：
- `idx_tags_deleted` ON `(deleted_at)`

---

## 6. post_tags — 文章-标签关联表

**SQL DDL（实际）：**

```sql
CREATE TABLE IF NOT EXISTS post_tags (
    post_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (post_id, tag_id),
    FOREIGN KEY (post_id) REFERENCES posts(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);
```

**Rust Entity（`src/models/entity/post_tags.rs`）：**

| 字段 | Rust 类型 | 可为空 | 说明 |
|------|-----------|--------|------|
| `post_id` | `i32` | ❌ PK + FK→posts.id | 文章 ID |
| `tag_id` | `i32` | ❌ PK + FK→tags.id | 标签 ID |

**外键**：
- `post_id` → `posts(id)` — 无 ON DELETE / ON UPDATE 子句
- `tag_id` → `tags(id)` — 无 ON DELETE / ON UPDATE 子句

**实际索引**：
- `idx_post_tags_tag` ON `(tag_id)`
- `idx_post_tags_post` ON `(post_id)`

> **注意**：此表没有独立 `id` 列，无 `created_at`。Entity 中 post_id 和 tag_id 均为 `#[sea_orm(primary_key)]`。

---

## 7. files — 文件表

**SQL DDL（实际）：**

```sql
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
```

**Rust Entity（`src/models/entity/files.rs`）：**

| 字段 | Rust 类型 | 可为空 | 说明 |
|------|-----------|--------|------|
| `id` | `i32` | ❌ PK | 文件 ID（自增） |
| `user_id` | `i32` | ❌ | 上传者 ID |
| `filename` | `String` | ❌ | 存储文件名 |
| `original_name` | `String` | ❌ | 原始文件名 |
| `mime_type` | `String` | ❌ | MIME 类型 |
| `size` | `i64` | ❌ | 文件大小（字节） |
| `storage_path` | `Option<String>` | ✅ | 存储路径 |
| `url` | `Option<String>` | ✅ | 访问 URL |
| `md5_hash` | `Option<String>` | ✅ | MD5 哈希 |
| `deleted_at` | `Option<NaiveDateTime>` | ✅ | 软删除时间 |
| `created_at` | `NaiveDateTime` | ❌ | 创建时间 |
| `updated_at` | `NaiveDateTime` | ❌ | 更新时间 |

**外键**：无（`user_id` 没有 FOREIGN KEY 约束，仅作为普通 INTEGER 列）

**实际索引**：
- `idx_files_deleted` ON `(deleted_at)`

> **注意**：列名是 `filename`（不是 `stored_name`），`size`（不是 `file_size`）。没有 `file_path`、`width`、`height`、`file_type`、`metadata` 等列。`user_id` 无外键约束。

---

## 8. settings — 系统设置 KV 表

**SQL DDL（实际）：**

```sql
CREATE TABLE IF NOT EXISTS settings (
    key VARCHAR PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Rust Entity（`src/models/entity/settings.rs`）：**

| 字段 | Rust 类型 | 可为空 | 说明 |
|------|-----------|--------|------|
| `key` | `String` | ❌ PK（非自增） | 设置键名 |
| `value` | `String` | ❌ | 设置值 |
| `updated_at` | `NaiveDateTime` | ❌ | 更新时间 |

**外键**：无

> **重要**：这是纯 KV 表，主键是 `key`（VARCHAR），没有自增 `id`。没有 `group_name`、`value_type`、`description`、`created_at` 等列。Entity 中标记 `auto_increment = false`。

**已知 Key 清单：**

| Key | 默认值 | 说明 |
|---|---|---|
| `site_title` | `"MarkShareX"` | 站点标题 |
| `site_subtitle` | `""` | 站点副标题 |
| `site_description` | `"A Markdown Blog"` | 站点描述 |
| `site_logo` | `""` | Logo URL（支持 `nr:{id}`） |
| `friend_links` | `"[]"` | 友情链接 JSON |
| `comment_moderation` | `"false"` | 评论审核开关 |
| `sidebar_collapse` | `"false"` | 侧栏分类折叠 |
| `guestbook_enabled` | `"true"` | 留言板开关 |
| `batch_load_size` | `"5"` | 列表首次加载文章数 |
| `scroll_load_size` | `"3"` | 滚动追加文章数 |
| `site-manager` | `""` | 站长用户 ID |

---

## 9. refresh_tokens — Refresh Token 表

**SQL DDL（实际）：**

```sql
CREATE TABLE IF NOT EXISTS refresh_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    token VARCHAR NOT NULL UNIQUE,
    expires_at DATETIME NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
```

**Rust Entity（`src/models/entity/refresh_tokens.rs`）：**

| 字段 | Rust 类型 | 可为空 | 说明 |
|------|-----------|--------|------|
| `id` | `i32` | ❌ PK | Token ID（自增） |
| `user_id` | `i32` | ❌ FK→users.id | 所属用户 ID |
| `token` | `String` | ❌ UNIQUE | Token 值 |
| `expires_at` | `NaiveDateTime` | ❌ | 过期时间 |
| `revoked` | `bool` | ❌ (DEFAULT 0) | 是否已吊销 |
| `created_at` | `NaiveDateTime` | ❌ | 创建时间 |

**外键**：
- `user_id` → `users(id)` — 无 ON DELETE / ON UPDATE 子句

**实际索引**：
- `idx_refresh_tokens_user` ON `(user_id)`
- `idx_refresh_tokens_token` ON `(token)`

> **注意**：列名是 `token`（不是 `token_hash`），`revoked` 是 BOOLEAN（不是 `revoked_at` TIMESTAMP）。没有 `device_info`、`ip_address`、`replaced_by` 等列。

---

## 10. comments — 评论表

**SQL DDL（实际）：**

```sql
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
```

**Rust Entity（`src/models/entity/comments.rs`）：**

| 字段 | Rust 类型 | 可为空 | 说明 |
|------|-----------|--------|------|
| `id` | `i32` | ❌ PK | 评论 ID（自增） |
| `post_id` | `i32` | ❌ FK→posts.id | 所属文章 ID |
| `user_id` | `Option<i32>` | ✅ FK→users.id | 评论者用户 ID（匿名时 NULL） |
| `parent_id` | `Option<i32>` | ✅ FK→comments.id | 父评论 ID（顶级为 NULL） |
| `author_name` | `String` | ❌ | 评论者名称 |
| `author_email` | `Option<String>` | ✅ | 评论者邮箱 |
| `content` | `String` | ❌ | 评论内容 |
| `content_html` | `String` | ❌ | 渲染后 HTML |
| `status` | `String` | ❌ (DEFAULT 'approved') | 评论状态 |
| `like_count` | `i32` | ❌ (DEFAULT 0) | 点赞数 |
| `ip_address` | `Option<String>` | ✅ | 评论者 IP |
| `created_at` | `NaiveDateTime` | ❌ | 创建时间 |
| `updated_at` | `NaiveDateTime` | ❌ | 更新时间 |
| `deleted_at` | `Option<NaiveDateTime>` | ✅ | 软删除时间 |

**外键**：
- `post_id` → `posts(id)` — 无 ON DELETE / ON UPDATE 子句
- `user_id` → `users(id)` — 无 ON DELETE / ON UPDATE 子句
- `parent_id` → `comments(id)` — 无 ON DELETE / ON UPDATE 子句

**实际索引**：
- `idx_comments_post` ON `(post_id)`
- `idx_comments_status` ON `(status)`
- `idx_comments_deleted` ON `(deleted_at)`

---

## 11. author_applications — 作者申请表

**SQL DDL（实际）：**

```sql
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
```

**Rust Entity（`src/models/entity/author_applications.rs`）：**

| 字段 | Rust 类型 | 可为空 | 说明 |
|------|-----------|--------|------|
| `id` | `i32` | ❌ PK | 申请 ID（自增） |
| `user_id` | `i32` | ❌ UNIQUE FK→users.id | 申请人 ID（一人一申请） |
| `reason` | `String` | ❌ | 申请理由 |
| `content_description` | `String` | ❌ (DEFAULT '') | 内容方向描述 |
| `status` | `String` | ❌ (DEFAULT 'pending') | 审批状态 |
| `admin_remark` | `Option<String>` | ✅ | 管理员审核备注 |
| `reviewed_by` | `Option<i32>` | ✅ | 审核人 ID |
| `reviewed_at` | `Option<NaiveDateTime>` | ✅ | 审核时间 |
| `created_at` | `NaiveDateTime` | ❌ | 创建时间 |
| `updated_at` | `NaiveDateTime` | ❌ | 更新时间 |

**外键**：
- `user_id` → `users(id)` **ON DELETE CASCADE** — 这是整个 Schema 中唯一带 ON DELETE CASCADE 的外键

> **注意**：`reviewed_by` 没有 FOREIGN KEY 约束。

---

## 12. read_logs — 阅读日志表

**SQL DDL（实际）：**

```sql
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
```

**Rust Entity（`src/models/entity/read_logs.rs`）：**

| 字段 | Rust 类型 | 可为空 | 说明 |
|------|-----------|--------|------|
| `id` | `i32` | ❌ PK | 记录 ID（自增） |
| `post_id` | `i32` | ❌ FK→posts.id | 被阅读文章 ID |
| `user_id` | `Option<i32>` | ✅ FK→users.id | 读者用户 ID（未登录时 NULL） |
| `ip_address` | `Option<String>` | ✅ | 访问者 IP |
| `user_agent` | `Option<String>` | ✅ | User-Agent 头 |
| `device_type` | `Option<String>` | ✅ | 设备类型（desktop/mobile/tablet/bot） |
| `referrer` | `Option<String>` | ✅ | 来源页面 URL |
| `duration_seconds` | `i32` | ❌ (DEFAULT 0) | 阅读时长（秒） |
| `created_at` | `NaiveDateTime` | ❌ | 记录时间 |

**外键**：
- `post_id` → `posts(id)` — 无 ON DELETE / ON UPDATE 子句
- `user_id` → `users(id)` — 无 ON DELETE / ON UPDATE 子句

**去重策略**：后端 `record_read_log` 端点对同一 IP + 同一 post_id 在 30 秒内重复请求自动跳过。

**用途**：
- API `/api/v1/analytics/total-views` — 从本表 `COUNT(*)` 获取总阅读量
- API `/api/v1/analytics/trend` — 按日期分组统计阅读趋势
- API `/api/v1/analytics/post-views` — 单篇文章阅读排行
- SSR 页面 `view_count` — 从本表实时查询
- 前端 `PostDetail.vue` — 进入和离开页面时各记录一次（离开时带 `duration_seconds`）

---

## 13. login_logs — 登录日志表

**SQL DDL（实际）：**

```sql
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
```

**Rust Entity（`src/models/entity/login_logs.rs`）：**

| 字段 | Rust 类型 | 可为空 | 说明 |
|------|-----------|--------|------|
| `id` | `i32` | ❌ PK | 记录 ID（自增） |
| `user_id` | `Option<i32>` | ✅ | 用户 ID（登录失败时可能 NULL） |
| `username` | `String` | ❌ | 登录用户名 |
| `ip_address` | `Option<String>` | ✅ | 登录 IP |
| `user_agent` | `Option<String>` | ✅ | User-Agent 头 |
| `device_type` | `Option<String>` | ✅ | 设备类型 |
| `login_method` | `String` | ❌ (DEFAULT 'password') | 登录方式（password/api_key） |
| `success` | `bool` | ❌ (DEFAULT 1) | 是否登录成功 |
| `created_at` | `NaiveDateTime` | ❌ | 登录时间 |

**外键**：无（`user_id` 无 FOREIGN KEY 约束）

**索引**：
- `idx_login_logs_user` ON `(user_id)`
- `idx_login_logs_created` ON `(created_at)`

> **注意**：此表无 `deleted_at`，采用硬删除。`user_id` 无外键约束，登录失败时可能为 NULL。

---

## 14. network_resources — 网络资源表

**SQL DDL（实际）：**

```sql
CREATE TABLE IF NOT EXISTS network_resources (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    url         TEXT NOT NULL,
    label       TEXT,
    source_type VARCHAR NOT NULL DEFAULT 'image',
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_network_resources_url ON network_resources(url);
```

**Rust Entity（`src/models/entity/network_resources.rs`）：**

| 字段 | Rust 类型 | 可为空 | 说明 |
|------|-----------|--------|------|
| `id` | `i32` | ❌ PK | 资源 ID（自增） |
| `url` | `String` | ❌ UNIQUE | 网络资源 URL |
| `label` | `Option<String>` | ✅ | 资源标签/描述 |
| `source_type` | `String` | ❌ (DEFAULT 'image') | 资源类型 |
| `created_at` | `NaiveDateTime` | ❌ | 创建时间 |
| `updated_at` | `NaiveDateTime` | ❌ | 更新时间 |

**外键**：无

**唯一索引**：`idx_network_resources_url` ON `(url)` — 每个 URL 只能登记一次

**用途**：文章中的外部图片引用通过 `nr:{id}` 占位符替换为真实 URL，避免 Markdown 原文包含冗长外链。该表存储 URL → ID 映射。

---

## 15. changelog — 版本更新说明表

**SQL DDL（实际）：**

```sql
CREATE TABLE IF NOT EXISTS changelog (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    version TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Rust Entity（`src/models/entity/changelog.rs`）：**

| 字段 | Rust 类型 | 可为空 | 说明 |
|------|-----------|--------|------|
| `id` | `i32` | ❌ PK | 记录 ID（自增） |
| `version` | `String` | ❌ UNIQUE | 版本号（如 "v0.2.0"；草稿时为空字符串） |
| `content` | `String`（Text） | ❌ (DEFAULT '') | Markdown 格式的更新说明 |
| `created_at` | `NaiveDateTime` | ❌ | 创建时间 |
| `updated_at` | `NaiveDateTime` | ❌ | 更新时间 |

**外键**：无

> **注意**：此表无 `deleted_at`，采用硬删除。`content` 为 Markdown 格式，前端通过 `/changelog` 路由展示。

---

## 16. likes — 点赞表

**SQL DDL（实际）：**

```sql
CREATE TABLE IF NOT EXISTS likes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    post_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, post_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (post_id) REFERENCES posts(id)
);
```

| 字段 | 类型 | 可为空 | 说明 |
|------|------|--------|------|
| `id` | `INTEGER` PK AUTOINCREMENT | ❌ | 点赞 ID |
| `user_id` | `INTEGER` FK→users.id | ❌ | 点赞用户 ID |
| `post_id` | `INTEGER` FK→posts.id | ❌ | 被点赞文章 ID |
| `created_at` | `DATETIME` | ❌ | 点赞时间 |

**外键**：
- `user_id` → `users(id)` — 无 ON DELETE / ON UPDATE 子句
- `post_id` → `posts(id)` — 无 ON DELETE / ON UPDATE 子句

**唯一约束**：`UNIQUE(user_id, post_id)` — 同一用户对同一文章只能点赞一次

> **注意**：此表**没有 Rust Entity**（`src/models/entity/` 下无对应文件）。有独立的自增 `id` 主键。

---

## 17. _migrations — 迁移追踪表

**SQL DDL（实际）：**

```sql
CREATE TABLE IF NOT EXISTS _migrations (
    name TEXT PRIMARY KEY,
    executed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

| 字段 | 类型 | 可为空 | 说明 |
|------|------|--------|------|
| `name` | `TEXT` PK | ❌ | 迁移文件名 |
| `executed_at` | `DATETIME` | ❌ | 执行时间 |

> 此表由 `migrations.rs` 管理，用于追踪增量迁移，无 Rust Entity。

---

## 18. 外键约束汇总

| 表 | 列 | 引用 | ON DELETE CASCADE? |
|----|-----|------|--------------------|
| `posts` | `user_id` | `users(id)` | ❌ |
| `posts` | `category_id` | `categories(id)` | ❌ |
| `post_tags` | `post_id` | `posts(id)` | ❌ |
| `post_tags` | `tag_id` | `tags(id)` | ❌ |
| `refresh_tokens` | `user_id` | `users(id)` | ❌ |
| `comments` | `post_id` | `posts(id)` | ❌ |
| `comments` | `user_id` | `users(id)` | ❌ |
| `comments` | `parent_id` | `comments(id)` | ❌ |
| `author_applications` | `user_id` | `users(id)` | ✅ **唯一的 CASCADE** |
| `read_logs` | `post_id` | `posts(id)` | ❌ |
| `read_logs` | `user_id` | `users(id)` | ❌ |
| `likes` | `user_id` | `users(id)` | ❌ |
| `likes` | `post_id` | `posts(id)` | ❌ |

**REFERENCES（非独立 FOREIGN KEY 子句）：**

| 表 | 列 | 引用 |
|----|-----|------|
| `categories` | `user_id` | `users(id)` |
| `tags` | `user_id` | `users(id)` |

**无 FK 的 ID 列**：`files.user_id`、`author_applications.reviewed_by`、`login_logs.user_id`

---

## 19. Entity 对应说明

### 所有 ID 类型为 `i32`

所有 Rust Entity 中的 `id`、`user_id`、`post_id` 等外键列类型均为 `i32`（与 SQLite `INTEGER` 匹配），**不是 `i64`**。

### Nullable 映射

SQL 中允许 NULL 的列在 Rust 中对应 `Option<T>`：
- `display_name`, `avatar_url`, `bio`, `last_login_at`, `deleted_at` → `Option<...>`
- `summary`, `content`, `content_html`, `cover_image`, `published_at` → `Option<...>`
- `storage_path`, `url`, `md5_hash` → `Option<...>`

### 无 CHECK 约束

实际 DDL 没有任何 `CHECK(...)` 约束。枚举值（如 status、role）仅为 VARCHAR + DEFAULT，由应用层逻辑保证。

### 无 ON DELETE / ON UPDATE 子句

除 `author_applications.user_id`（`ON DELETE CASCADE`）外，所有 FOREIGN KEY 均无 ON DELETE / ON UPDATE 子句。

### 无 Entity 的表

`likes` 在 SQL 中存在但**尚未在 `src/models/entity/` 中定义 Rust Entity**。如需通过 SeaORM 操作此表，需要创建对应的 Entity 文件。

---

## 20. 软删除策略

### 采用软删除的表（有 `deleted_at` 列）

| 表 | 说明 |
|----|------|
| `users` | 用户数据，保留审计轨迹 |
| `posts` | 核心内容，误删可恢复 |
| `categories` | 分类，影响文章归属 |
| `tags` | 标签，防误删 |
| `files` | 文件资源，需延迟清理 |
| `comments` | 用户生成内容，支持审核恢复 |

### 采用硬删除的表（无 `deleted_at` 列）

| 表 | 说明 |
|----|------|
| `post_tags` | 关联记录，直接删除 |
| `refresh_tokens` | 安全敏感，过期/吊销后直接删除 |
| `settings` | 配置项直接 UPDATE/DELETE |
| `author_applications` | 审批流程记录 |
| `likes` | 点赞操作频繁，直接删除 |
| `read_logs` | 统计数据，按时间清理 |
| `login_logs` | 安全审计日志，按时间清理 |
| `network_resources` | 资源引用，允许直接删除 |
| `changelog` | 版本记录，允许直接删除 |

### 软删除查询要求

所有涉及软删除表的查询需添加 `WHERE deleted_at IS NULL` 条件。

---

## 附录：完整 SQLite DDL

以下为 `migrations/0000000000_init_schema.sql` 的完整建表与索引内容：

```sql
-- users
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
    is_active BOOLEAN NOT NULL DEFAULT 1,
    last_login_at DATETIME,
    deleted_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- categories
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL,
    slug VARCHAR NOT NULL UNIQUE,
    description TEXT,
    parent_id INTEGER,
    sort_order INTEGER NOT NULL DEFAULT 0,
    image_url VARCHAR,
    image_filename TEXT,
    user_id INTEGER REFERENCES users(id),
    deleted_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- tags
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL,
    slug VARCHAR NOT NULL UNIQUE,
    user_id INTEGER REFERENCES users(id),
    deleted_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- posts
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
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (category_id) REFERENCES categories(id)
);

-- post_tags
CREATE TABLE IF NOT EXISTS post_tags (
    post_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (post_id, tag_id),
    FOREIGN KEY (post_id) REFERENCES posts(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);

-- files
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

-- settings
CREATE TABLE IF NOT EXISTS settings (
    key VARCHAR PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- refresh_tokens
CREATE TABLE IF NOT EXISTS refresh_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    token VARCHAR NOT NULL UNIQUE,
    expires_at DATETIME NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- network_resources
CREATE TABLE IF NOT EXISTS network_resources (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    url         TEXT NOT NULL,
    label       TEXT,
    source_type VARCHAR NOT NULL DEFAULT 'image',
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_network_resources_url ON network_resources(url);

-- login_logs
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

-- read_logs
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

-- changelog
CREATE TABLE IF NOT EXISTS changelog (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    version TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- likes
CREATE TABLE IF NOT EXISTS likes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    post_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, post_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (post_id) REFERENCES posts(id)
);

-- comments
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

-- author_applications
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

-- _migrations
CREATE TABLE IF NOT EXISTS _migrations (
    name TEXT PRIMARY KEY,
    executed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 索引
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
```
