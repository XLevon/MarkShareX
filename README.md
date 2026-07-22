# MarkShareX

> 轻量自托管 Markdown 博客系统 — Rust + Vue 3 + SQLite

[![Rust](https://img.shields.io/badge/Rust-1.95+-orange.svg)](https://www.rust-lang.org)
[![Vue](https://img.shields.io/badge/Vue-3.x-4fc08d.svg)](https://vuejs.org)
[![Version](https://img.shields.io/badge/version-0.4.0-blue.svg)](https://github.com/XLevon/MarkShareX)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

MarkShareX 是一个面向技术创作者的轻量级自托管博客平台。高性能 Rust 后端配合现代化 Vue 3 管理后台，内置 AI 自动写作与资讯采集能力，数据完全由你掌控。

---

## 功能特性

### 🤖 AI 写作与调度
- 🧠 **多模型支持** — OpenAI 兼容 / DeepSeek / 硅基流动，统一管理 API 提供商
- 🛠️ **工具系统** — 内置 web_search、web_extract、create_post、api_request 等工具，支持自定义扩展
- 📋 **技能编排** — 可复用的技能描述模板，定义采集流程与创作规范
- ⏰ **定时调度** — 内置 cron 调度器，自动执行资讯采集、知识创作等定时任务
- 📊 **执行追踪** — 手动执行任务实时显示每轮 LLM 对话与工具调用详情
- 💬 **AI 对话** — 管理后台内置 AI 聊天面板，支持多轮对话
- 🔑 **API Key 托管** — AES-256-GCM 加密存储，Web 后台管理

### 🔍 多源搜索
- 🔗 **搜索降级链** — Tavily → Firecrawl → DuckDuckGo 三级自动降级
- 🌐 **web_search / web_extract** — AI 工具调用搜索，网页正文提取
- 📰 **资讯采集** — 自动搜索今日热点 + 去重 + 抓取全文 + 创建资讯草稿

### 内容创作
- ✍️ **Vditor 编辑器** — IR 即时渲染模式，支持 Markdown 快捷输入、工具栏、全屏写作、暗色模式
- 📝 **知识管理** — 文章草稿/发布状态、二级分类归属、标签关联、封面图设置、文章类型（原创/AI整理/教程/转载/翻译/随笔）
- 📎 **本地资源** — 图片/文件上传、MD5 去重、批量操作、未引用文件检测清理
- 🖼️ **网络资源** — 外链图片入库管理，URL 变更后全站引用自动同步，杜绝图片失效
- 📥 **批导管理** — Markdown + 图片 ZIP 导入导出，兼容 CSDN 博客格式，YAML Front Matter 解析
- 📢 **资讯管理** — 独立资讯模块（与文章分离），题材分类（时政/财经/科技等 9 类），筛选搜索、批量发布/撤回

### 阅读体验
- 🔍 **全文搜索** — Tantivy 搜索引擎，CJK 中文分词，统一搜索文章/标签/作者
- 🌐 **SEO 友好** — SSR 服务端渲染（Tera 模板），每篇文章独立 slug 路由，OG 标签
- 🏷️ **多维度浏览** — 按分类（树形结构）、标签、作者筛选文章，热门文章/标签云/友链侧边栏
- 💬 **评论系统** — 支持匿名/登录评论、嵌套回复、评论审核（pending/approved/deleted）
- ❤️ **互动功能** — 文章点赞（toggle）、浏览统计、阅读日志记录、前后篇导航
- 📋 **代码增强** — 自动代码复制按钮，登录保护
- 📖 **留言板** — 访客留言 + 管理员回复，前台简洁展示

### 管理后台
- 📊 **仪表盘** — 已发布/草稿/阅读量/点赞量/评论数统计卡片，每日增量角标，快捷入口，存储用量，阅读趋势图
- 📝 **知识库** — 文章/分类/标签管理，状态筛选、搜索、分页，角色权限控制
- 📂 **资源库** — 本地资源/网络资源管理，拖拽上传，网格/列表视图，类型筛选，未用检查，批量删除
- 👥 **用户管理** — RBAC 四角色（admin/sub_admin/author/visitor），状态管控（active/muted/banned），登录日志
- 📝 **作者申请** — 访客提交申请，管理员审核通过/拒绝（附备注）
- 📈 **阅读统计** — 文章阅读榜（含作者/点赞/评论），点击查看详细阅读日志，点赞记录列表
- 📊 **评论管理** — 状态 Tab 筛选，审核操作，详情弹窗
- 🎨 **明暗主题** — CSS 变量驱动，Vditor 编辑器跟随
- ⚙️ **系统设置** — 站点Logo/标题/副标题/描述、友链 JSON、评论审核开关、侧栏分类折叠
- 📋 **版本说明** — 更新日志增删改维护
- 🔑 **API Key** — 每个用户独立 API Key，支持 X-API-Key 认证

### 工程品质
- 🔒 **安全认证** — JWT 双 Token（Access + Refresh），bcrypt 密码哈希，X-API-Key 双认证
- 🔄 **Token 自动刷新** — 401 响应自动 refresh token 续期
- 🛡️ **XSS 防护** — ammonia 净化 HTML，comrak 安全 Markdown 渲染
- 🔄 **CORS 支持** — 跨域访问控制
- 📦 **前后台分离** — 前台公开浏览 + 后台管理面板，各自独立路由和布局
- 🗜️ **编译嵌入** — SQL 迁移 + 静态资源编译进二进制
- 📚 **OpenAPI 文档** — Scalar UI 交互式 API 文档，150+ 端点完整覆盖

---

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端框架 | Rust 2021 + Axum 0.7 |
| ORM | SeaORM |
| 数据库 | SQLite（可选 PostgreSQL） |
| 搜索引擎 | Tantivy（CJK 中文分词） |
| SSR 模板 | Tera |
| Markdown | comrak + ammonia |
| 前端框架 | Vue 3 + TypeScript |
| UI 组件 | Naive UI |
| 编辑器 | Vditor |
| 样式 | Tailwind CSS v4 |
| 认证 | JWT + bcrypt + X-API-Key |
| 部署 | Docker / 单二进制 |

---

## 快速开始

### Docker 部署

```bash
git clone https://github.com/XLevon/MarkShareX.git
cd MarkShareX
cp .env.example .env
# 编辑 .env 设置 JWT_SECRET 等

# 国外直接从 Docker Hub 拉取基础镜像进行构建并启动容器
docker compose up -d

# 国内推荐分步构建并启动容器
./scripts/start.sh

# 从本地基础镜像进行构建并启动容器
docker compose -f docker-compose.local.yml up -d

# 删除容器
docker compose down            # 停止并删除容器（数据保留，volume 还在）
docker compose down -v         # 停止 + 删除容器 + 删除数据卷（⚠️ 清空数据库/上传文件）
```

访问 `http://localhost:5023`，首次启动进入安装向导创建管理员账号。

### 单二进制部署

```bash
# 构建前端
cd frontend && npm install && npm run build && cd ..

# 构建后端
cargo build --release

# 配置文件
cp config.example.toml config.toml

# 运行
./target/release/marksharex
```

首次启动自动创建 SQLite 数据库并执行迁移，无需手动初始化。

### 本地开发

```bash
# 后端
cargo run

# 前端（另一个终端）
cd frontend
npm install
npm run dev
```

前端 `http://localhost:5173`，API 代理到后端 `http://localhost:5023`。

配置详情见 [docs/CONFIG.md](docs/CONFIG.md)。

---

## API 端点

共 150+ 个 REST API 端点，完整覆盖前后台功能。在线文档：启动后访问 `/scalar`。

| 模块 | 路径前缀 | 端点数 | 说明 |
|------|---------|--------|------|
| 认证 | `/api/v1/auth` | 3 | 注册、登录、Token 刷新 |
| 文章 | `/api/v1/posts` | 15 | CRUD、slug、搜索、点赞、浏览、作者、前后导航、阅读日志 |
| 分类 | `/api/v1/categories` | 6 | 公开/管理列表、CRUD、拖拽排序 |
| 标签 | `/api/v1/tags` | 4 | 含文章计数的列表、CRUD |
| 文件 | `/api/v1/files` | 7 | 上传、批量上传、MD5 检查、列表、未引用、删除 |
| 网络资源 | `/api/v1/network-resources` | 8 | CRUD、引用查询、批量解析、URL 确保 |
| 评论 | `/api/v1/posts/:id/comments` | 5 | 发表、列表（树形嵌套）、管理、审核、待审计数 |
| 留言板 | `/api/v1/guestbook` | 4 | 列表、发表、管理员回复、删除 |
| 资讯 | `/api/v1/news` | 8 | 公开/管理列表、CRUD、题材列表、搜索筛选 |
| 用户管理 | `/api/v1/admin/users` | 8 | 列表、创建、更新、角色/状态变更、删除、登录日志、阅读日志 |
| 个人资料 | `/api/v1/profile` | 6 | 查看、更新、改密、API Key 管理、管理员信息 |
| 作者申请 | `/api/v1/apply` | 5 | 提交、状态查询、审批通过/拒绝、待审计数 |
| 数据分析 | `/api/v1/analytics` | 8 | 趋势、总计、今日增量、文章排行、点赞记录 |
| 导入导出 | `/api/v1/(import\\|export)` | 2 | Markdown ZIP 导入/导出 |
| 设置 | `/api/v1/settings` | 2 | 站点设置获取/更新 |
| 初始化 | `/api/v1/setup` | 2 | 状态检查、创建管理员 |
| 更新日志 | `/api/v1/changelogs` | 6 | 公开列表、最新版本、管理端 CRUD |
| AI 对话 | `/api/v1/ai` / `/api/v1/admin/ai` | 30 | 聊天、提供商、模型、工具、技能、任务、Agent、会话 |
| 运维 | `/api/v1/admin` | 3 | 运行日志、健康检查、系统统计 |
| 系统 | `/api/v1` | 4 | 端点发现、OpenAPI JSON、健康检查、版本 |

---

## 角色权限体系

| 角色 | 权限范围 |
|------|---------|
| **admin** | 完全权限：管理用户、资源、内容，审核申请，系统设置，AI 调度 |
| **sub_admin** | 管理权限：管理用户、资源、内容，审核申请 |
| **author** | 创作权限：创建、编辑自己的文章和分类，管理自己文章的评论 |
| **visitor** | 只读权限：浏览、评论，申请成为作者 |

---

## 数据库模型

共 25 张表：

### 内容相关
| 表 | 说明 |
|----|------|
| `posts` | 文章（title/slug/content/html/status/封面/计数/软删除/post_type/article_type） |
| `categories` | 分类（树形 parent_id、封面、排序） |
| `tags` | 标签（name/slug） |
| `post_tags` | 文章-标签多对多关联 |
| `comments` | 评论（树形 parent_id、状态、点赞计数） |
| `guestbook` | 留言板（昵称/内容/邮箱/管理员回复） |
| `news` | 资讯（标题/摘要/题材/状态/排序） |
| `changelog` | 更新日志（版本号/内容） |

### 资源相关
| 表 | 说明 |
|----|------|
| `files` | 文件（MD5/路径/类型/大小） |
| `network_resources` | 网络资源（URL/标签/类型/引用计数） |

### 用户相关
| 表 | 说明 |
|----|------|
| `users` | 用户（username/email/password_hash/role/status/api_key） |
| `refresh_tokens` | 刷新令牌 |
| `author_applications` | 作者申请（理由/状态/审批备注） |
| `login_logs` | 登录日志（IP/设备/方式/成功标记） |
| `read_logs` | 阅读日志（文章/用户/IP/时长/来源） |

### AI 相关
| 表 | 说明 |
|----|------|
| `ai_providers` | AI 提供商（类型/API URL/API Key） |
| `ai_models` | AI 模型（名称/标识/输入输出价格） |
| `ai_tools` | AI 工具（函数名/描述/参数 Schema/执行器） |
| `ai_skills` | AI 技能（名称/提示词模板） |
| `ai_tasks` | 定时任务（cron/关联技能/提供商/最大轮次） |
| `ai_task_logs` | 任务执行日志（追踪步骤/结果/错误） |
| `ai_agent_config` | Agent 配置（系统提示词/默认模型/温度） |
| `ai_chat_sessions` | AI 对话会话 |
| `ai_chat_messages` | AI 对话消息历史 |

### 系统相关
| 表 | 说明 |
|----|------|
| `settings` | 站点设置（key-value） |
| `article_types` | 文章类型字典（原创/AI整理/教程等） |
| `article_statuses` | 文章状态字典（规划中/施工中/已完成等） |

---

## 配置说明

配置文件 `config.toml`，支持环境变量覆盖（格式 `MARKSHAREX_<SECTION>_<KEY>`）。

```toml
[server]
host = "0.0.0.0"
port = 5023

[database]
url = "sqlite://./data/marksharex.db?mode=rwc"

[auth]
jwt_secret = ""  # 必填；部署时生成独立随机值
encrypt_key = "" # 必填；部署后必须长期固定

[ai.search]
provider = "tavily"         # 主搜索: tavily | firecrawl
api_key = ""
fallback_provider = "firecrawl"  # 降级搜索
fallback_api_key = ""
# duckduckgo 免费兜底，无需 API Key

[ai]
max_tool_rounds = 8         # AI 工具调用最大轮次
```

环境变量覆盖示例：
```bash
export MARKSHAREX_SERVER_PORT=8080
export MARKSHAREX_AI_MAX_TOOL_ROUNDS=15
export MARKSHAREX_AUTH_JWT_SECRET="$(openssl rand -base64 64)"
export MARKSHAREX_AUTH_ENCRYPT_KEY="$(openssl rand -base64 32)"
```

完整字段、全部环境变量、覆盖优先级和校验规则见
[`docs/CONFIG.md`](docs/CONFIG.md)、`.env.example` 和 `config.example.toml`。

---

## 部署方式

| 方式 | 适用场景 | 说明 |
|------|----------|------|
| Docker Compose | 快速体验 / 中小规模 | 一键启动，数据卷持久化 |
| 单二进制 + systemd | 生产环境 | 最小依赖，systemd 守护 |

---

## License

MIT © 2026 [XLevon](mailto:408251965@qq.com)
