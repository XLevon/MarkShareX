# MarkShareX 系统功能说明书 v0.2.0

> 完整功能清单与架构说明 — 上次更新 2026-05-26

---

## 一、项目概览

MarkShareX 是一个面向技术创作者的轻量级自托管 Markdown 博客平台。核心特点：

- **双栏编辑**：Vditor IR 即时渲染 + Live Preview，粘贴图片自动上传
- **网络资源引用系统**：外链图片存储为 `nr:{id}` 活引用，URL 变更自动跟随
- **RBAC 四角色**：admin / sub_admin / author / visitor
- **全文搜索**：Tantivy + CJK 中文分词
- **SSR 服务端渲染**：Tera 模板，SEO 友好
- **前后台分离**：公开浏览 + 管理面板，各自独立路由和布局
- **编译嵌入**：SQL 迁移 + 静态资源编译进二进制，单文件部署

---

## 二、技术架构

```
┌──────────────────────────────────────────────┐
│                   Browser                     │
│  Frontend (Vue 3 + NaiveUI + Vditor)         │
│  /            → 前台公开页面                   │
│  /admin/*     → 后台管理面板                  │
└──────────────┬───────────────────────────────┘
               │ REST API (/api/v1/*)
┌──────────────▼───────────────────────────────┐
│              Axum HTTP Server                 │
│  ┌──────────┬──────────┬──────────┐          │
│  │Controllers│ Services │Middleware│          │
│  │ 17 模块   │  5 服务   │  2 中间件 │          │
│  └──────────┴──────────┴──────────┘          │
│  ┌──────────┬──────────┬──────────┐          │
│  │ SeaORM   │ Tantivy  │  Tera    │          │
│  └────┬─────┴────┬─────┴────┬─────┘          │
└───────┼──────────┼──────────┼────────────────┘
        │          │          │
   SQLite DB   全文索引  SSR 模板
```

### 数据层

- **ORM**: SeaORM，支持 SQLite / PostgreSQL 切换
- **迁移系统**: 初始化 SQL + 增量迁移文件，`build.rs` 编译嵌入二进制，`_migrations` 追踪表幂等执行
- **全文搜索**: Tantivy 0.22，CJK 中文自动分词，启动时全量重建索引
- **日志缓冲**: 内存环形缓冲区 5000 条，tracing Layer 捕获，运维 API 查询

---

## 三、后端模块详解

### 3.1 控制器 (controllers/)

| 文件 | 端点数 | 说明 |
|------|--------|------|
| `mod.rs` | 0 | 路由注册聚合，列出所有 `/api/v1/*` 路由 |
| `auth.rs` | 3 | 注册、登录（邮箱/用户名）、JWT 刷新 |
| `posts.rs` | 15 | 文章 CRUD、slug 路由、搜索、点赞（toggle）、浏览去重（IP 30s）、作者列表、前后导航、阅读日志 |
| `categories.rs` | 6 | 公开/管理列表、CRUD、拖拽排序、隐藏分类过滤、封面 `nr:` 引用 |
| `tags.rs` | 4 | 含文章计数列表、CRUD |
| `comments.rs` | 5 | 发表（登录/匿名）、树形嵌套列表、管理审核、待审计数（含 scope=mine） |
| `files.rs` | 7 | 单/批量上传、MD5 去重、未引用检测、物理+DB 删除 |
| `network_resources.rs` | 8 | CRUD、URL 去重（去 fragment）、引用查询、批量解析、`nr:{id}` 302 重定向 |
| `settings.rs` | 2 | key-value 站点设置 |
| `setup.rs` | 2 | 初始化状态检查、创建管理员 |
| `profile.rs` | 6 | 个人资料、改密、API Key 管理、管理员公开信息 |
| `admin.rs` | 8 | 用户 CRUD、角色/状态变更、级联删除（文章/文件/评论/分类）、登录/阅读日志 |
| `author_applications.rs` | 5 | 提交、状态、审批、待审计数 |
| `analytics.rs` | 8 | 趋势（按天）、总计（浏览/点赞/评论）、今日增量（发布/点赞）、文章排行（分页，含作者/点赞/评论） |
| `import_export.rs` | 2 | Markdown ZIP 导入/导出，CSDN 标签提取，图片 base64/URL 处理 |
| `changelog.rs` | 6 | 公开/管理列表、最新版本、CRUD |
| `ops.rs` | 3 | 日志查询、健康检查（DB/磁盘/内存/uptime）、统计 |
| `api_doc.rs` | 0 | OpenAPI 3.0 文档聚合（utoipa 派生） |

### 3.2 服务层 (services/)

| 服务 | 功能 |
|------|------|
| `auth.rs` | JWT 生成/验证（Access Token 短期 + Refresh Token 长期），bcrypt 密码哈希 |
| `posts.rs` | 文章分页/查询/删除（含级联）、标签解析（按名查找或创建）、前后篇 SQL、Markdown 渲染（comrak + ammonia + referrerpolicy 注入）、slug 生成 |
| `search.rs` | Tantivy 索引管理（初始化/索引/删除/搜索），CJK 预处理，启动重建 |
| `files.rs` | 上传（MD5 去重/重名处理/覆写）、引用检查（文章内容+封面+分类+Logo） |
| `logs.rs` | 环形日志缓冲区 5000 条、LogCaptureLayer、条件查询 |

### 3.3 中间件 (middleware/)

| 中间件 | 功能 |
|--------|------|
| CORS | 允许所有来源/方法/头 |
| Auth | `AuthUser` 提取器：JWT (Bearer) + X-API-Key 双认证，登录日志记录，`is_admin()`/`is_privileged()` 权限判断 |

### 3.4 数据模型 (models/entity/)

14 张表，支持软删除（deleted_at）统一 pattern。

| 表 | 关键字段 |
|----|---------|
| `users` | id, username, email, password_hash, display_name, avatar_url, role, status, api_key, bio, last_login_at |
| `posts` | id, user_id, category_id, title, slug, content, content_html, cover_image/cover_image_url/cover_network_id, status, is_pinned, view_count, like_count, comment_count, published_at |
| `categories` | id, name, slug, parent_id, is_visible, sort_order, image_url/network_resource_id |
| `tags` | id, name, slug |
| `post_tags` | post_id, tag_id (联合主键) |
| `comments` | id, post_id, user_id, parent_id, author_name, content, status (pending/approved/deleted), ip_address |
| `files` | id, filename, original_name, mime_type, size, storage_path, url, md5_hash |
| `network_resources` | id, url, label, source_type, reference_count |
| `settings` | key (PK), value |
| `refresh_tokens` | id, user_id, token, expires_at, revoked |
| `author_applications` | id, user_id, reason, content_description, status, admin_remark |
| `changelog` | id, version, content |
| `login_logs` | id, user_id, username, ip_address, user_agent, device_type, login_method, success |
| `read_logs` | id, post_id, user_id, ip_address, user_agent, duration_seconds, referrer |

---

## 四、前端模块详解

### 4.1 路由结构

#### 前台路由 (FrontLayout)
| 路径 | 组件 | 说明 |
|------|------|------|
| `/` | Home.vue | Hero 区 + 按分类分组文章 + 侧边栏（热门/TagCloud/友链/站长） |
| `/post/:id` | PostDetail.vue | TOC 目录 + 内容 + 代码复制 + 评论（嵌套）+ 点赞/分享 |
| `/tags` | TagsAll.vue | 标签云搜索页 |
| `/tag/:slug` | TagPosts.vue | 标签文章列表 |
| `/categories` | CategoriesAll.vue | 分类列表 |
| `/category/:slug` | CategoryPosts.vue | 分类文章列表 |
| `/authors` | AuthorsList.vue | 作者列表（搜索/Top10） |
| `/author/:id` | AuthorPosts.vue | 作者文章列表 |
| `/search` | SearchResults.vue | 全局搜索（文章/标签/作者分类展示） |
| `/login` | Login.vue | 登录弹窗（记住我） |
| `/register` | Register.vue | 注册弹窗 |
| `/apply` | ApplyAuthor.vue | 申请成为作者 |
| `/changelog` | Changelog.vue | 版本更新说明 |

#### 后台路由 (AdminLayout, requiresAuth)
| 路径 | 组件 | 说明 |
|------|------|------|
| `/admin/dashboard` | Dashboard.vue | 5 统计卡片 + 趋势 SVG + 存储用量 + 最近文章 |
| `/admin/posts` | PostList.vue | 三合一 Tab：文章列表 / 分类管理 / 标签管理 |
| `/admin/posts/new` | PostEdit.vue | 新建文章编辑器 |
| `/admin/posts/:id` | PostEdit.vue | 编辑文章编辑器 |
| `/admin/files` | Files.vue | 双 Tab 资源库：本地资源 / 网络资源 |
| `/admin/analytics/views` | ViewsAnalytics.vue | 阅读统计排行 + 详细日志弹窗 |
| `/admin/analytics/comments` | CommentsAdmin.vue | 评论审核管理 |
| `/admin/likes` | Likes.vue | 点赞记录列表 |
| `/admin/import` | Import.vue | 批量导入/导出面板 |
| `/admin/users` | AdminUsers.vue | 用户管理 + 申请审批 |
| `/admin/settings` | Settings.vue | 双 Tab：站点设置 / 版本说明 |
| `/admin/setup` | Setup.vue | 系统初始化（免认证） |

### 4.2 组件清单

| 组件 | 类型 | 功能 |
|------|------|------|
| FrontLayout | 布局 | 前台导航 + 搜索 + 用户下拉菜单 |
| AdminLayout | 布局 | 后台导航 + keep-alive 缓存 |
| NavBar | 共享 | 通用导航栏 + 深色切换 + 用户头像 |
| PostCard | 共享 | 文章卡片（双模式 front/admin） |
| CodeCopyWrapper | 共享 | 代码块自动复制按钮 |
| ImageSelector | 共享 | 统一图片选择器（本地/网络/上传） |
| VditorEditor | 编辑 | Vditor 封装（粘贴上传、暗色模式、外链图片 referrerpolicy） |
| ActionBar | 前台 | 前后篇导航 + 点赞分享 |
| SidebarCard | 前台 | 侧边栏卡片容器 |
| TagCloud | 前台 | 标签云（按计数算字体/颜色/旋转） |
| FriendLinks | 前台 | 友链列表 |
| WebmasterInfo | 前台 | 站长信息展示 |

### 4.3 状态管理 (Pinia Store)

| Store | 核心状态 | 说明 |
|-------|---------|------|
| authStore | token, refreshToken, user, isAuthenticated | 登录/登出/Token 管理，401 自动刷新 |
| settingsStore | settings, networkUrlCache, resolvedLogoUrl | 站点设置 + nr: URL 缓存。`batch_load_size`/`scroll_load_size` 控制全站列表首次及滚动加载数量 |

### 4.4 API 模块 (前端)

共 12 个 API 文件覆盖约 60 个端点调用：

| 文件 | 主要端点 |
|------|---------|
| `index.ts` | axios 实例（baseURL, Token 拦截器, 401 刷新, 公共类型） |
| `auth.ts` | register, login, refresh |
| `posts.ts` | posts CRUD, slug, search, like, view, adjacent |
| `categories.ts` | categories CRUD, reorder |
| `tags.ts` | tags list, create, delete |
| `comments.ts` | comments list, create, admin CRUD |
| `files.ts` | upload, batch, MD5 check, unreferenced |
| `settings.ts` | fetch/update settings |
| `setup.ts` | setup status, create admin |
| `changelog.ts` | CRUD, latest, public list |
| `importExport.ts` | export ZIP, import markdown |
| `admin.ts` | users CRUD, profile, API key, applications, network resources, logs |

### 4.5 Composables

| 文件 | 功能 |
|------|------|
| useDarkMode | 明暗主题切换（localStorage 持久化，class 切换） |
| useTitleParts | 站点标题分色渲染 |
| useSearchVisibility | 导航栏搜索框可见性控制（IntersectionObserver） |

---

## 五、关键架构设计

### 5.1 网络资源引用系统 (`nr:{id}`)

外部 URL 图片统一管理，避免外链失效：

1. **注册**: 图片 URL 存入 `network_resources` 表，分配唯一 ID
2. **引用**: 文章内容/封面存储 `nr:{id}` 代替原始 URL
3. **渲染**: `resolve_nr_in_content()` 将 `nr:{id}` 还原为真实 URL
4. **保护**: 被引用的资源不可删除
5. **批量解析**: `batch-resolve` 一次性获取 ID→URL 映射

### 5.2 Markdown 渲染管道

```
raw Markdown
  → comrak (CommonMark + GFM + 表格)
  → ammonia (XSS 白名单清洗)
  → referrerpolicy 注入 (外链图片 no-referrer)
  → content_html 存储
```

### 5.3 全文搜索

- **引擎**: Tantivy 0.22
- **分词**: CJK 中文自动分词（cjk_to_token）
- **索引字段**: title, body, post_id
- **更新策略**: 启动全量重建 + 文章变更时增量索引
- **统一搜索**: 同时搜索文章（全文）、标签（LIKE）、作者（LIKE）

### 5.4 权限判断

```rust
AuthUser::is_admin()      → role == Admin
AuthUser::is_privileged() → role in (Admin, SubAdmin)
```

**角色权限矩阵：**

| 操作 | admin | sub_admin | author | visitor |
|------|-------|-----------|--------|---------|
| 管理用户 (CRUD) | ✓ | ✓ | ✗ | ✗ |
| 管理所有文章 | ✓ | ✓ | ✗ | ✗ |
| 管理自己的文章 | ✓ | ✓ | ✓ | ✗ |
| 管理分类/标签 | ✓ | ✓ | ✓(自己创建的) | ✗ |
| 上传文件 | ✓ | ✗ | ✗ | ✗ |
| 修改系统设置 | ✓ | ✗ | ✗ | ✗ |
| 管理主题 | ✓ | ✗ | ✗ | ✗ |
| 管理评论状态 | ✓ | ✓ | ✗ | ✗ |
| 发表评论 | ✓ | ✓ | ✓ | ✓ |
| 查看分析数据 | ✓(全局) | ✓(全局) | ✓(自己) | ✗ |
| 导入/导出文章 | ✓ | ✓ | ✓(自己) | ✗ |
| 审批作者申请 | ✓ | ✓ | ✗ | ✗ |
| 提交作者申请 | ✗ | ✗ | ✗ | ✓ |

### 5.5 Token 双认证

- **JWT Bearer Token**: 标准认证，用于浏览器端
- **X-API-Key Header**: 静态 API Key，用于 CLI/第三方集成
- **401 自动刷新**: 前端拦截器捕获 401 → 静默 refresh → 重放请求

### 5.6 文件管理

- **MD5 去重**: 上传时计算 hash，相同文件共享存储
- **文件名安全**: 自动处理空格、括号等特殊字符
- **未引用检测**: 检查文章正文 `![alt](url)` + 封面 + 分类图片 + 站点 Logo
- **物理删除**: DB 记录 + 磁盘文件同步删除

---

## 六、部署配置

### 环境变量 (`.env` / `.env.docker`)

| 变量 | 说明 |
|------|------|
| `DATABASE_URL` | SQLite 路径或 PostgreSQL 连接串 |
| `JWT_SECRET` | JWT 签名密钥 |
| `JWT_REFRESH_SECRET` | Refresh Token 签名密钥 |
| `SERVER_HOST` | 绑定地址（默认 0.0.0.0） |
| `SERVER_PORT` | 服务端口（默认 5023） |
| `UPLOAD_DIR` | 上传目录 |
| `STATIC_DIR` | 前端静态资源目录 |
| `SITE_URL` | 站点完整 URL（SSR 用） |

### Docker Compose

```yaml
services:
  marksharex:
    image: marksharex:latest
    ports: ["5023:5023"]
    volumes:
      - ./data:/data
    environment: ...
```

### 单二进制

```bash
# 前端构建 → 后端构建 → 单文件运行
cd frontend && npm run build && cd ..
cargo build --release
cp config.toml config.toml
./target/release/marksharex
```

---

## 七、开发备忘

### 编译检查
- `cargo check` — 后端类型检查（快）
- `cargo build` — 完整编译
- `npm run build` — 前端生产构建

### 前后端联调
- 前端 Vite dev: `http://localhost:5173`，proxy → `http://localhost:5023`
- 后端: `cargo run`
- 修改 `.vue` 文件热更新，修改 `.rs` 文件需 ctrl+c 重启

### Git 工作流
1. 本地改代码 → `cargo check` / `npm run build` 验证
2. 用户确认通过 → `git push`
3. 服务器 `git pull` 部署

---

## 八、版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.3.1 | 2026-05-29 | 修复 `skip_deserializing` 导致 admin 列表 ID 归零/编辑 404、首页"技术文章"统计=0、路由统一 `:id`→`:slug`、首页文章卡片加分类标签、PostEdit 错误诊断日志 |
| v0.3.0 | 2026-05-30 | 前端列表分批加载系统设置（`batch_load_size`/`scroll_load_size`）、迁移合并、文档同步、删除 API.md（改用 Scalar） |
| v0.2.2 | 2026-05-28 | 网络资源引用系统、阅读日志、登录日志、分析面板完善、OpenAPI 文档补全 |
| v0.2.0 | 2026-05-26 | 评论审核系统、SSR slug 路由 |
| v0.1.1 | 2026-05-23 | 项目更名 MarkShareX |
| v0.1.0 | 2026-05-18 | 初始版本：文章/分类/标签/文件 CRUD、Vditor 编辑器、RBAC |
