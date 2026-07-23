# MarkShareX 系统全貌

> 基于 MarkShareX v0.4.2 当前源码、README 与 `docs/` 现有专题文档综合整理。
> 本文面向使用者、开发者与运维人员，说明系统定位、总体架构、功能模块、数据流、权限边界、AI 能力、运行机制及部署方式。若本文与旧文档存在差异，以当前源码和迁移文件为准。

---

## 1. 系统定位

MarkShareX 是一个面向个人技术创作者和小型内容团队的轻量级、自托管 Markdown 内容平台。它处在静态博客与重量级 CMS 之间：既保留 Markdown 写作、slug 固定链接和低运维成本，又提供在线管理后台、用户权限、全文搜索、评论互动、资源管理、数据分析以及 AI 自动采集和写作能力。

系统的核心目标是：

1. **数据自主**：文章、资讯、用户、配置和资源全部保存在自有数据库与文件目录中。
2. **轻量部署**：Rust 后端承担 API、SSR、静态资源和后台任务；默认使用 SQLite，不依赖外部数据库或搜索服务。
3. **完整创作链路**：从 Markdown 编辑、图片管理、分类标签到发布、搜索、评论和统计形成闭环。
4. **搜索与 SEO 友好**：Tantivy 提供中文全文搜索，Rust/Tera 为公开页面生成可抓取 HTML 和独立 metadata。
5. **AI 原生扩展**：将模型供应商、模型、Agent、技能、工具、任务和执行日志建模为可管理对象，使 AI 能读取站内数据、搜索网络并创建文章或资讯。
6. **人机协作**：AI 可以完成信息采集和初稿生成，最终内容仍可在管理后台审核、编辑和发布。

### 1.1 适用场景

- 个人技术博客与知识库
- 小团队内部知识沉淀
- Markdown 在线创作与多作者协作
- 带全文搜索和管理后台的轻量内容站
- AI Agent 自动采集、整理和发布内容的实验平台
- 希望使用单一应用进程和 SQLite 降低运维复杂度的自托管站点

### 1.2 当前技术规模

按 v0.4.2 源码快照统计，排除 `.git`、`node_modules`、`target` 和构建产物后：

- Rust 源文件：82 个，约 2.5 万行代码
- Vue 单文件组件：55 个，约 2.1 万行代码
- TypeScript：41 个文件，约 0.4 万行代码
- 后端控制器模块：23 个
- `/api/v1/*` API operation：162 个，由 `src/api_endpoints.rs` 权威目录统一生成
- 当前版本仅支持 SQLite；PostgreSQL 和 MySQL 计划在后续版本支持。
- 初始化 Schema 当前包含 29 张应用表和 27 个应用索引。其中包含 `_migrations` 迁移追踪表。

这些数字用于描述当前规模，不作为稳定 API 承诺。

---

## 2. 总体架构

```text
┌────────────────────────────────────────────────────────────┐
│                         Browser                            │
│                                                            │
│  公开前台 Vue SPA             管理后台 Vue SPA             │
│  首页/知识库/文章/搜索/评论     内容/资源/用户/AI/设置        │
└───────────────┬──────────────────────────┬─────────────────┘
                │ HTML / assets            │ REST / JSON
                ▼                          ▼
┌────────────────────────────────────────────────────────────┐
│                    Rust + Axum 0.7                         │
│                                                            │
│  页面层          API 控制器       中间件         后台任务    │
│  Tera SSR        /api/v1/*        认证/IP/安全头  AI cron    │
│                                                            │
│  服务层：文章、认证、文件、搜索、AI、日志、执行追踪          │
└───────────────┬──────────────┬──────────────┬──────────────┘
                │              │              │
                ▼              ▼              ▼
          SeaORM / SQL     Tantivy 索引     本地文件系统
          SQLite           data/search_index data/uploads
                │
                ▼
      文章、资讯、用户、配置、AI 元数据与日志
```

### 2.1 前后端协作方式

MarkShareX 不是“纯 SSR”或“纯 SPA”，而是两者协作：

- **Rust 页面路由**负责公开页面的初始 HTML、SEO metadata、Open Graph、Twitter Card、JSON-LD、robots 和 sitemap。
- **Vue Router**接管浏览器中的前台和管理后台交互，实现无刷新导航、异步加载、状态管理和复杂编辑界面。
- **Axum REST API**统一提供数据读写，前端 Axios 基础路径为 `/api/v1`。
- **Vite 开发服务器**运行在 `5173`，将 `/api`、`/uploads`、`/scalar` 代理到 Rust 后端；生产构建输出到 `static/frontend/`，由 Rust 提供资源服务。

### 2.2 后端分层

```text
controllers/  HTTP 参数、响应、权限入口、路由处理
services/     可复用业务逻辑、Markdown、搜索、AI、日志
models/       SeaORM Entity、数据库连接与初始化迁移
middleware/   身份认证、IP 访问控制、压缩、安全头、缓存
config/       config.toml 与受支持的环境变量覆盖
utils/        统一响应、错误、AppState、客户端/IP/时间工具
migrations/   初始化 Schema 及后续 SQL 增量迁移
```

控制器与服务并非绝对隔离：部分业务仍直接位于控制器中，但总体上遵循“路由处理—业务服务—数据模型”的组织方式。

`src/api_endpoints.rs` 是 162 个 `/api/v1/*` operation 的权威目录。每条声明同时包含 HTTP method、Axum path、handler、Utoipa path metadata 和是否需要认证；宏展开从同一目录生成 production Router、OpenAPI paths 与 `GET /api/v1/` discovery metadata。测试逐端点把 Axum 的 `:id` 规范化为 OpenAPI 的 `{id}`，校验 method/path 完全一致、无重复且双向无遗漏；测试期还从真实 handler 参数签名独立核对必选认证 extractor，并用迁移前的 162-operation fingerprint 防止重构漏项或错绑。受保护的 OpenAPI operation 必须精确声明 `bearerAuth OR apiKeyAuth`（`X-API-Key`）、使用空 scope、引用已注册 scheme；公开 operation 不得带有效 security requirement。端点发现通过独立 baseline fixture 逐字保留重构前 76 条中文描述；shared-path Router 测试还验证 POST+DELETE、GET+POST、GET+PUT+DELETE 的 405 与精确 `Allow` 集合。`GET /api/v1/openapi.json` 返回的版本从 Cargo package version 派生，`/scalar` 继续只允许管理员访问。

---

## 3. 项目目录

```text
MarkShareX/
├── src/
│   ├── main.rs                 # 启动入口与全局装配
│   ├── controllers/            # API、SSR 页面与系统路由
│   ├── services/               # 文章、认证、搜索、文件、AI、日志
│   ├── models/entity/          # SeaORM 数据实体
│   ├── middleware/             # Auth、IP Guard、压缩和安全响应头
│   ├── config/                 # 配置结构与加载逻辑
│   ├── migrations.rs           # 编译嵌入的增量迁移执行器
│   ├── api_endpoints.rs         # API Router/OpenAPI/discovery 权威端点目录
│   ├── api_doc.rs              # OpenAPI 聚合
│   ├── crypto.rs               # AI API Key 加解密
│   └── utils/                  # 错误、响应和通用工具
├── frontend/
│   ├── src/router/             # 前台、后台路由与导航守卫
│   ├── src/views/front/        # 公开阅读页面
│   ├── src/views/admin/        # 管理后台页面
│   ├── src/components/         # 布局及共享组件
│   ├── src/api/                # Axios 请求模块与类型
│   ├── src/stores/             # Pinia 登录、站点设置状态
│   ├── src/composables/        # 主题、标题和可见性逻辑
│   ├── src/utils/              # 标题、校验和内容访问工具
│   ├── src/__tests__/          # Vitest/Vue 组件、Store 与竞态行为测试
│   └── tests/                  # Node 源码契约和脚本门禁测试
├── templates/default/          # Tera SSR 模板
├── static/frontend/            # Vite 生产构建产物
├── migrations/                 # SQL 初始化与增量迁移
├── data/                       # 默认运行时数据目录
│   ├── marksharex.db           # SQLite 数据库
│   ├── uploads/                # 上传文件
│   ├── search_index/           # Tantivy 索引
│   └── templates/              # 启动时写入的内嵌模板副本
├── docs/                       # 专题文档和本文
├── scripts/                    # Docker/启动辅助脚本
├── Cargo.toml                  # Rust 依赖和版本
├── config.example.toml         # 配置示例
├── Dockerfile                  # 三阶段容器构建
└── docker-compose.yml          # 容器编排与数据卷
```

---

## 4. 应用启动与运行时生命周期

`src/main.rs` 中的启动流程如下：

1. 读取 `.env`。
2. 初始化终端标识、tracing 日志和容量为 5000 条的内存环形日志缓冲区。
3. 从 `config.toml` 加载配置，并应用代码明确支持的环境变量覆盖。
4. 将配置中的加密密钥注入加解密模块使用的环境变量。
5. 创建 `data_dir` 和上传目录。
6. 将编译时嵌入的默认 Tera 模板写入 `{data_dir}/templates/default/`。
7. 初始化 SeaORM 数据库连接。
8. 执行初始化 Schema 和增量 SQL 迁移。
9. 初始化 `{data_dir}/search_index` 中的 Tantivy 索引。
10. 比较已发布文章数与索引文档数；索引为空或数量不一致时自动重建。
11. 迁移旧 IP 设置数据格式。
12. 创建共享 `AppState`，其中包含数据库、配置、搜索引擎和日志缓冲区。
13. 后台启动 AI 定时调度器；调度器每 60 秒检查一次任务。
14. 合并 API 路由、SSR 页面路由和 SPA fallback。
15. 依次挂载压缩、安全响应头、静态资源缓存、请求体大小限制、IP Guard、HTTP trace 和 CORS。
16. 绑定配置的 host/port，启动 Axum HTTP 服务。

### 4.1 搜索索引恢复

Tantivy 索引不是数据库真相源。启动时系统会比较：

- 数据库中 `status='published'` 且未软删除的文章数
- Tantivy 当前文档数

数量不一致时从数据库重建索引。文章新增、Markdown 导入、更新、发布状态变化和删除时也会增量维护索引；published 导入完成数据库事务后会立即写入 Tantivy。若增量提交失败，会按数据库真相源执行全量重建。只有增量提交或重建至少一项成功时请求才可成功，已知的索引失败不会被静默忽略。若 published import 的增量与重建同时失败，服务会在独立事务中补偿删除已提交的文章、关系以及仅本次创建且已无引用的分类/标签；只有补偿确认成功后 controller 才删除本次导入文件。若补偿自身失败，则保留文章、关系和文件，响应以 `success=false` 报告索引异常，但 `imported_count` 按 durable state 计数并在 `persisted_with_errors` 返回已持久化 post ID，避免“文章仍引用但文件已删”或客户端按 skipped 盲目重试。未显式提供 slug 的导入使用标题的确定性规范化 base，并在整个 posts 唯一域（包括软删除保留行）中依次选择 `base`、`base-2`……；非 ASCII 标题规范化为空时使用 `post` base。

### 4.2 AI 调度恢复

AI Scheduler 启动时会把上次异常退出遗留的 `running` 日志标记为失败，避免任务永久卡在执行中。每个任务在进程内通过 `running` 集合防止同一任务并发重入。

---

## 5. 功能模块全景

### 5.1 文章与知识库

文章是系统的核心内容实体，支持：

- Markdown 原文与净化后的 HTML 双份存储
- 草稿与发布状态；文章删除写入 `deleted_at` 软删除并保留关联数据
- 独立 slug 固定链接
- 作者、树形分类和多标签
- 封面、本地文件或网络资源引用
- 置顶、排序和前后篇导航
- 评论开关、点赞计数、评论计数和阅读统计
- 文章类型与文章状态字典
- 管理端搜索、状态筛选、批量删除和置顶排序
- ZIP 导入导出及 YAML Front Matter 解析；多行 `tags:` 后的 `status`、`slug` 等字段仍会在同一轮状态切换中解析，不依赖字段顺序；显式 slug 的重复检查和自动 slug 碰撞递增都覆盖软删除保留行，与数据库全表唯一约束一致

文章写权限按角色和资源所有者共同约束：author 可创建文章、更新本人文章和删除本人草稿，但不能转移作者、设置置顶或删除已发布文章；admin、sub_admin 可全局管理文章、作者归属、置顶和批量删除。非特权用户的后台文章列表始终按当前用户 ID 过滤，调用方传入的 `author_id` 不能越权查询他人草稿。

公开读取只暴露 `published` 文章。按 ID、slug、SSR 详情、相邻文章、评论、点赞状态和阅读日志等关联入口复用同一发布状态边界；未发布文章对匿名访客和非所有者统一表现为 404。文章所有者可在认证后的详情 API 中读取本人草稿，admin、sub_admin 可读取全局草稿。

文章类型和状态并不是写死在前端的简单枚举，而是由 `article_types`、`article_statuses` 两张字典表维护；文章保存其 code，公开页面可按类型或状态聚合浏览。公开类型/状态列表的 `post_count` 仅统计已发布且未软删除文章，管理端计数仍覆盖全部引用，确保删除约束不会忽略草稿或历史行。

#### Markdown 渲染管道

```text
Markdown 原文
  → 解析 nr:{id} 网络资源引用
  → comrak 生成 GFM HTML
  → ammonia 白名单净化
  → 外链图片属性增强
  → content_html 持久化
  → SSR / SPA 展示
```

文章可见 H1 和 JSON-LD `headline` 保留完整标题；网页 `<title>`、OG 和 Twitter metadata 使用运行时宽度控制规则，避免中文标题在搜索结果中被不合理截断。

### 5.2 分类、标签、作者与筛选

- 分类支持父子层级、显示/隐藏、排序、封面和创建者归属。
- 标签通过 `post_tags` 与文章建立多对多关系。
- 公开页面支持分类、标签、作者、文章类型和文章状态五类聚合入口。
- 管理员和子管理员可查看全局内容；作者的写操作按资源所有者进行约束。

### 5.3 资讯模块

资讯与知识文章分表管理，适用于时效性内容：

- 草稿/发布状态
- 标题、摘要、Markdown 正文与 HTML
- 原始来源 URL
- 题材分类：时政、财经、科技、社会、文娱、体育、国际、法治、教育
- 搜索、筛选、排序、批量删除和发布管理
- AI 工具可自动创建资讯
- AI 创建前按来源 URL 和近期标题相似度进行去重

公开首页可将资讯作为“每日简讯”展示，而知识文章仍进入知识库和 Tantivy 索引，两类内容职责分离。

### 5.4 评论、留言、点赞与阅读日志

#### 评论

- 支持登录用户和匿名访客发表评论
- `parent_id` 构成嵌套回复树
- 状态包含 pending、approved、deleted
- 是否进入待审由站点设置决定
- 管理端提供列表、筛选、审核和待审数量
- 公共评论列表和评论创建只接受已发布文章；admin、sub_admin 使用明确的管理视图时仍可检查草稿下的历史评论

#### 留言板

留言板独立于文章评论，支持：

- 访客昵称、邮箱和正文
- 登录用户关联
- 管理员回复与删除
- 全站启用开关
- 访客复制能力开关

#### 点赞与阅读

- `likes` 记录用户与文章的唯一点赞关系，API 提供 toggle 和状态查询。
- `read_logs` 记录文章、用户/IP、User-Agent、设备、来源和阅读时长。
- 文章阅读统计和管理端趋势分析主要基于阅读日志聚合。
- 点赞、点赞状态、相邻文章和阅读日志接口都会先确认目标文章已发布，拒绝对草稿泄露元数据或产生互动写入。
- 前端文章页对主文章、相邻文章、评论和点赞请求设置请求身份校验，避免快速切换路由时旧响应覆盖新文章状态。

### 5.5 本地文件资源

文件模块提供：

- 单文件和批量上传
- MIME 类型与大小限制
- MD5 去重
- 重名处理
- 数据库记录和磁盘文件同步删除
- 未引用文件检测
- 网格/列表视图及批量操作

引用检测会检查文章正文、封面、分类图片和站点 Logo 等位置，降低误删正在使用资源的风险。

### 5.6 网络资源引用

网络资源模块解决外链图片 URL 变化或散落在文章中的问题。

```text
外部 URL
  → network_resources 注册并去重
  → 获得资源 ID
  → 内容中保存 nr:{id}
  → API 批量解析或 /resolve 302
  → 渲染时还原当前 URL
```

主要能力包括：

- URL 规范化与去重
- `nr:{id}` 活引用
- 查询某资源被哪些文章、封面、分类或设置引用
- 批量解析 ID 到 URL
- URL 更新后引用自动跟随
- 删除前检查引用关系

该设计将“内容表达”与“外部地址”解耦，是 MarkShareX 区别于普通 Markdown 博客的重要特性。

### 5.7 用户、作者申请与个人资料

用户具有角色和状态两个维度：

- 角色：`admin`、`sub_admin`、`author`、`visitor`
- 状态：如 `active`、`muted`、`banned`

系统支持：

- 注册、登录和 Token 刷新
- 个人资料、头像、简介和头衔
- 修改密码
- 每用户独立 API Key
- 登录日志和阅读日志
- visitor 提交作者申请
- 管理员/子管理员审批并填写备注

### 5.8 数据分析与运维

管理后台提供：

- 已发布文章、草稿、浏览、点赞和评论统计
- 今日文章和今日点赞增量
- 阅读趋势
- 文章阅读排行及详细日志
- 点赞记录
- 评论审核统计
- 登录日志
- 内存环形运行日志
- 数据库、磁盘、内存和进程 uptime 健康信息

公开健康检查为：

```text
GET /api/v1/health → OK
```

管理运维接口位于 `/api/v1/admin/logs`、`/health`、`/stats`。

### 5.9 站点设置与更新日志

`settings` 是 key-value 表，保存站点标题、副标题、描述、Logo、友情链接、评论审核、侧栏行为、留言板开关和列表加载数量等运行时设置。

更新日志模块独立维护版本号、内容、发布状态和时间，公开页面展示已发布记录，管理端支持 CRUD。

### 5.10 资源删除语义

删除操作按资源的数据价值和存储约束明确区分：

| 资源 | 删除语义 | 关联数据行为 |
|---|---|---|
| 文章 | 软删除（写入 `deleted_at`） | 保留标签关系、评论、点赞和阅读日志；立即从公开查询、相邻文章、sitemap 与 Tantivy 索引中隐藏；删除后拒绝发布、取消发布、置顶、取消置顶和置顶排序 |
| 评论 | 软删除（写入 `deleted_at`） | 保留正文和审核记录，公开及后台默认列表排除 |
| 分类 | 未被任何文章引用时硬删除 | 有活动或软删除文章引用都拒绝；子分类解除父级关系与父分类删除处于同一事务，任一步失败整体回滚 |
| 标签 | 未被任何文章引用时硬删除 | 有 `post_tags` 关系则拒绝，禁止通过删除标签隐式改写文章 |
| 本地文件 | 事务性硬删除 | 数据库记录与物理文件必须一起删除；失败时恢复两者 |
| 资讯 | 硬删除 | 资讯不进入文章关系链，保持明确的永久删除行为 |

软删除文章不会批量清理历史互动数据；数据库是内容真相源，搜索索引属于可重建派生数据。

---

## 6. AI 子系统

AI 子系统不是单一“聊天接口”，而是一套可配置的 Agent 运行环境。

### 6.1 核心对象

| 对象 | 作用 |
|---|---|
| AI Provider | 保存 OpenAI 兼容供应商类型、Base URL 和加密 API Key |
| AI Model | 归属于 Provider 的模型名称及默认标记 |
| Agent Config | 系统提示词、用户提示词、默认模型和默认标记 |
| AI Tool | function name、描述、JSON Schema、启用状态和配置 |
| AI Skill | 可复用任务模板、说明、输出格式和参数模板 |
| AI Task | 技能、模型、Agent、cron、参数、最大轮次和启用状态 |
| Task Log | 执行状态、工具步骤、最终回复和错误 |
| Chat Session / Message | 后台多轮聊天会话与消息历史 |

API Key 使用 AES-256-GCM 加密后存入数据库；加密密钥必须在生产环境固定保存，变更后旧密文将无法解密。

### 6.2 Function Calling 循环

```text
用户消息 / 定时任务
  → 选择 Agent、Provider 和 Model
  → 从数据库加载启用工具
  → 调用 OpenAI-compatible /chat/completions
  → LLM 返回 tool_calls
  → Rust ToolRegistry 执行工具
  → 工具结果追加到消息上下文
  → 继续下一轮，直到最终文本或达到轮次上限
  → 保存 trace、最终回复和错误
```

任务级 `max_tool_rounds` 优先于全局配置；全局未设置时默认 8 轮。执行追踪记录每一轮 LLM 内容、工具名、参数和结果预览，手动任务可在前端轮询查看过程。

### 6.3 内置工具

当前内置能力包括：

- `get_current_datetime`：获取服务器本地时间
- `api_request`：在受控范围内调用站内 API
- `web_search`：网络搜索
- `web_extract`：抓取网页正文
- `create_news`：创建资讯
- `create_post`：创建知识文章

工具是否暴露给模型由数据库中的 `ai_tools` 配置控制，工具元数据与 Rust 实际执行器通过 `function_name` 对应。

### 6.4 搜索与抓取降级链

网络搜索支持：

```text
主提供商（通常 Tavily）
  → 备用提供商（通常 Firecrawl）
  → 可选 SearXNG
  → DuckDuckGo 兜底
```

网页抓取优先使用配置的商业提供商，最终可降级为直接 HTTP GET。工具对结果数量、抓取 URL 数量和正文长度进行限制，避免单次上下文无限增长。

### 6.5 定时调度

- Scheduler 每 60 秒读取启用任务。
- cron 表达式会先标准化再匹配本地时间。
- 同一任务在单进程内禁止并发重入。
- 任务执行不会阻塞下一次调度 tick。
- 成功后更新 `last_run_at` 和 `run_count`。
- trace 持久化到 `ai_task_logs`，异常退出后的僵尸任务会在启动时清理。

### 6.6 AI 权限边界

- Provider、Model、Agent Config、Skill、Tool、Task、Task Log、供应商测试和任务执行等 AI 管理 handler 统一使用 `AdminUser`，仅当前数据库角色为 active admin 的用户可调用。
- `AdminUser` 不只信任 JWT 中的历史角色声明；敏感操作前会回查用户当前角色、状态和删除标记，因此降权、禁用或删除会立即影响管理权限。
- Chat Session 和 Message 属于用户私有资源。会话列表固定按当前 `user_id` 过滤，详情、删除、普通聊天和 slash command 都要求会话所有者匹配。
- AI 会话没有管理员跨用户例外；包括 admin、sub_admin 在内的所有角色都只能访问自己的会话，跨用户访问统一返回 404 以减少资源枚举。

---

## 7. 搜索与 SEO

### 7.1 站内全文搜索

Tantivy 索引字段包括：

- `title`
- `body`
- `post_id`

系统对中日韩字符以及 CJK 与字母的边界插入分隔，使 SimpleTokenizer 能处理中文查询。统一搜索接口还会补充标签和作者的数据库匹配结果。

索引目录位于：

```text
{data_dir}/search_index/
```

数据库始终是内容真相源，索引可以重建。

### 7.2 SSR 与 SPA 的 SEO 分工

Rust 直接处理：

- 首页和知识库
- 分类、标签、作者、文章类型、文章状态聚合页
- 推荐文章和更新日志入口
- 文章详情
- `robots.txt`
- `sitemap.xml`
- favicon 和默认 OG 图片

SSR 输出包含：

- `<title>` 和 description
- canonical
- robots 指令
- Open Graph / Twitter metadata
- WebSite、CollectionPage 或 Article JSON-LD
- 文章或聚合页的预渲染可读内容

Vue 在客户端路由切换时同步 `document.title`：

- `App.vue` 管理静态路由标题
- 动态页面组件管理文章、分类、标签、作者、类型和状态标题
- 前后端统一使用站点标题显示规则与 fallback

文章 title 使用中英文加权宽度控制：ASCII 等半角字符计 1，中文、全角字符和 Emoji 等计 2；超限时优先移除站点后缀，再对文章标题进行安全截断。description 也采用加权宽度并优先在自然标点处截断。

---

## 8. 前端架构与页面

### 8.1 技术栈

- Vue 3.5 + Composition API
- TypeScript 5.7
- Vue Router 4.5
- Pinia 3
- Axios
- Naive UI
- Tailwind CSS 4
- Vditor
- Vite 6
- marked、JSZip、file-saver、dayjs 等工具库

### 8.2 公开前台

| 路径 | 页面能力 |
|---|---|
| `/` | Hero、每日简讯、文章列表、侧栏信息 |
| `/knowledge-base` | 知识文章浏览 |
| `/post/:slug` | 文章正文、目录、前后篇、点赞、评论、阅读记录 |
| `/categories`、`/category/:slug` | 分类总览与分类文章 |
| `/tags`、`/tag/:slug` | 标签总览与标签文章 |
| `/authors`、`/author/:id` | 作者列表与作者文章 |
| `/types`、`/type/:code` | 文章类型筛选 |
| `/statuses`、`/status/:code` | 文章状态筛选 |
| `/pinned` | 推荐/置顶文章 |
| `/search` | 统一搜索结果 |
| `/changelog` | 版本更新记录 |
| `/guestbook` | 留言板 |
| `/login`、`/register` | 登录与注册 |
| `/apply` | 作者申请 |

`FrontLayout` 负责导航、全局搜索、用户菜单、侧栏与主题体验。Vue Router 保存滚动位置，并在异步内容渲染后恢复浏览器前进/后退位置。

### 8.3 管理后台

| 路径 | 模块 |
|---|---|
| `/admin/dashboard` | 仪表盘与运行概况 |
| `/admin/posts` | 文章列表 |
| `/admin/posts/new`、`/:id` | Vditor 新建/编辑文章 |
| `/admin/categories` | 分类管理 |
| `/admin/tags` | 标签管理 |
| `/admin/files` | 本地与网络资源库 |
| `/admin/analytics/views` | 阅读分析 |
| `/admin/analytics/comments` | 评论审核 |
| `/admin/likes` | 点赞记录 |
| `/admin/import` | 导入导出 |
| `/admin/users` | 用户与作者申请 |
| `/admin/settings` | 站点设置与更新日志 |
| `/admin/guestbook` | 留言管理 |
| `/admin/news` | 资讯管理 |
| `/admin/ai` | Provider、Model、Agent、Skill、Tool、Task、Chat 与日志 |
| `/admin/setup` | 首次初始化 |

前端路由守卫允许 admin、sub_admin、author 进入后台框架，仅 admin 可进入 AI 管理页面。它只负责用户体验，真正的安全边界仍是后端权限检查。

认证信息被视为不可拆分的 Access Token、Refresh Token 与用户元组，用户对象必须至少包含合法的数字 ID 和角色：勾选“记住登录”时只写入 `localStorage`，否则只写入 `sessionStorage`；每次登录切换和退出都会先清理两处。Axios 请求、自动刷新、路由守卫和 Pinia Store 统一通过同一存储模块读取，发现历史冲突、残缺或畸形凭据时会 fail closed 清空并要求重新登录。每个请求在发出时绑定所属会话，401 刷新锁按原会话隔离；同一会话的并发或错峰 401 只刷新一次并沿 token 轮换关系重放。Refresh 响应的用户 ID 必须与原会话身份一致，角色等资料可更新但不得切换用户。登录请求使用 generation 防止 logout 后晚响应复活会话或旧登录覆盖新登录；跨标签页的 Storage identity replacement/logout 会同步 Pinia，并使本地未完成的旧登录失效。旧请求、旧 refresh 或重试响应即使晚于新登录或 logout 返回，也不得借用、覆盖或清除新会话。

### 8.4 状态管理

当前全局 Pinia Store 保持精简：

- `authStore`：Access Token、Refresh Token、用户和登录状态
- `settingsStore`：站点设置、网络资源 URL 缓存和解析后的 Logo

大部分页面状态保留在组件内部，避免将所有业务都堆入全局 Store。

### 8.5 API 请求与 Token 刷新

Axios 请求拦截器自动附加：

```http
Authorization: Bearer <access-token>
```

出现 401 时：

1. 查找 Refresh Token。
2. 使用全局共享 Promise 发起一次刷新，防止并发 401 重复刷新。
3. 写回新 Token 和用户信息。
4. 重放原请求。
5. 刷新失败时清空两种 Storage 中的认证信息，并通知路由系统。

管理页面收到 `auth:expired` 后跳转登录页；公开页面不会因过期 Token 被强制中断。

---

## 9. API 组织

所有业务 API 使用 `/api/v1` 前缀。主要模块如下：

| 模块 | 典型路径 | 说明 |
|---|---|---|
| 端点发现 | `/api/v1/` | 返回可供工具自举的端点元数据 |
| 健康与版本 | `/health`、`/version` | 服务状态与版本 |
| 认证 | `/auth/*` | 注册、登录、刷新 |
| 文章 | `/posts/*` | CRUD、slug、相邻文章、点赞 |
| 搜索 | `/search` | 文章、标签、作者统一搜索 |
| 分类/标签 | `/categories/*`、`/tags/*` | 公开列表与管理操作 |
| 类型/状态 | `/article-types`、`/article-statuses` | 字典与筛选 |
| 评论 | `/posts/:id/comments`、`/admin/comments` | 已发布文章的公开评论与后台审核 |
| 文件 | `/files/*` | 上传、MD5、未引用和删除 |
| 网络资源 | `/network-resources/*` | 活引用、解析和引用检查 |
| 用户/资料 | `/admin/users/*`、`/profile/*` | 用户管理与个人资料 |
| 作者申请 | `/apply/*`、`/admin/applications/*` | 申请与审批 |
| 分析 | `/analytics/*` | 趋势、总计和排行 |
| 资讯 | `/news/*`、`/admin/news/*` | 公开读取和管理 CRUD |
| 留言 | `/guestbook`、`/admin/guestbook/*` | 留言与回复 |
| 更新日志 | `/changelogs/*` | 公开列表、最新版本和管理 CRUD |
| 导入导出 | `/import/posts`、`/export/posts` | Markdown ZIP 数据交换 |
| AI | `/ai/*`、`/admin/ai/*` | admin-only 管理资源与 owner-only 私有会话 |
| 运维 | `/admin/logs`、`/health`、`/stats` | 运行诊断 |
| OpenAPI | `/openapi.json`、`/scalar` | 机器可读规范与交互文档 |

`/scalar` 本身由 admin 中间件保护，可接受后台设置的 `scalar_token` Cookie 或 Bearer Token。

`GET /api/v1/` 的端点发现列表、OpenAPI paths 和 production Router 均由 `src/api_endpoints.rs` 的同一目录生成。`auth_required` 会同步写入 discovery，并映射为 OpenAPI 的 Bearer/API Key security requirement；测试再与真实 handler 的必选认证 extractor 独立核对，防止 catalog 自我对照产生假阳性。点赞及点赞状态接口当前都明确要求登录。

---

## 10. 身份认证与权限边界

### 10.1 两种认证方式

#### JWT

- Access Token：短期访问
- Refresh Token：长期续期并在数据库中可撤销
- 密码使用 bcrypt 哈希
- 浏览器默认通过 Bearer Token 调用 API

#### X-API-Key

- 管理员用户可生成 API Key
- 请求头为 `X-API-Key`
- 后端查询处于 active 状态且 Key 匹配的用户
- 用于 CLI、脚本和外部 AI Agent 集成
- 成功和失败均可写入登录审计日志

认证提取器优先检查 X-API-Key，再回退到 JWT。公开但可向所有者展示额外数据的接口使用 `OptionalAuthUser`：完全没有凭据时按匿名处理，调用方一旦提交无效 Bearer Token 或 API Key 则返回认证错误，不会静默降级为匿名身份。

### 10.2 角色边界

| 角色 | 典型能力 |
|---|---|
| admin | 全局内容、用户、设置、资源、运维、Scalar、AI 管理 |
| sub_admin | 全局内容与审核类管理，不拥有仅 admin 的系统/AI 权限 |
| author | 创建和维护本人内容及允许的相关资源 |
| visitor | 公开浏览、互动和申请作者 |

具体权限由后端 extractor、控制器策略函数、当前角色和资源所有者共同决定。前端隐藏按钮和路由守卫只改善交互体验，不能作为安全边界。

### 10.3 统一权限提取器

本批权限封口引入并统一使用以下后端 extractor：

| Extractor | 允许范围 | 关键行为 |
|---|---|---|
| `AuthUser` | 任意已认证用户 | 解析 JWT 或 X-API-Key 身份 |
| `OptionalAuthUser` | 匿名或合法认证用户 | 无凭据时匿名；无效凭据不会静默放行 |
| `AdminUser` | active admin | 敏感操作前回查数据库当前角色、状态和删除标记 |
| `PrivilegedUser` | active admin、sub_admin | 用于全局内容、置顶和批量管理操作，并实时回查数据库 |

后端仍未给所有 `/api/v1/admin/*` 路由机械地套同一层全局中间件；权限由具体 handler 按资源类型选择 extractor。因此 URL 中出现 `admin` 不能单独作为权限事实来源，仍应以路由注册和 handler 签名为准。AI 管理的 30 个管理 handler 已统一为 `AdminUser`，未再依赖调用方自行判断角色。

### 10.4 文章权限矩阵

| 操作 | visitor | author | admin/sub_admin |
|---|---|---|---|
| 创建文章 | 拒绝 | 允许 | 允许 |
| 更新文章 | 拒绝 | 仅本人 | 全局 |
| 删除草稿 | 拒绝 | 仅本人 | 全局 |
| 删除已发布文章 | 拒绝 | 拒绝 | 全局 |
| 修改作者归属 | 拒绝 | 拒绝 | 允许 |
| 设置置顶或调整顺序 | 拒绝 | 拒绝 | 允许 |
| 批量删除 | 拒绝 | 拒绝 | 允许 |
| 读取本人草稿 | 拒绝 | 允许 | 允许 |
| 读取他人草稿 | 拒绝 | 拒绝 | 允许 |

公开文章列表无论调用方传入何种 `status` 都固定查询 `published`。按 ID、slug 和 SSR 路径读取未发布文章时，匿名用户和非所有者得到 404；后台文章列表对非特权用户无条件覆盖 `author_id` 为当前用户，避免通过查询参数读取他人草稿元数据。作者在创建和更新路径都不能绕过置顶限制，也不能转移文章所有权。

文章关联公共操作同样执行发布状态检查：草稿的相邻文章、点赞、点赞状态、阅读日志、评论列表和评论创建均返回 404，且拒绝请求不会产生互动数据。admin、sub_admin 仍可通过明确的评论管理视图检查草稿下已有评论。

### 10.5 AI 管理与会话所有权

AI 管理资源与用户会话采用两条独立边界：

- Provider、Model、Agent Config、Skill、Tool、Task、Task Log、供应商测试和任务执行仅允许当前数据库角色为 active admin 的用户。
- `AdminUser` 和 `PrivilegedUser` 会回查数据库，因此 JWT 中过期的高权限角色声明不能继续授权；用户被降权、禁用或删除后，敏感接口立即拒绝。
- AI 会话列表按数据库当前角色分流：admin 可查看全员会话并获得用户名显示字段；sub_admin、author、visitor 只查看本人会话且不显示用户名前缀。
- 会话详情和删除允许 owner 或数据库当前角色为 admin 的用户；sub_admin 及其他角色访问他人会话返回 404。聊天续接与 slash command 对所有角色（包括 admin）都保持严格 owner-only，禁止继续他人的对话。

### 10.6 状态与访问控制

除角色外，用户状态会参与认证和业务判断。敏感角色 extractor 只接受当前数据库中 active 且未删除的用户。系统还提供 IP Guard，对配置的 IP 规则进行请求级限制。

---

## 11. 数据库全貌

当前版本仅支持 SQLite；PostgreSQL 和 MySQL 计划在后续版本支持。Cargo 当前只启用 SeaORM 的 `sqlx-sqlite`，迁移、PRAGMA、Docker 默认配置和持久化验收均以 SQLite 为正式边界；未来数据库支持必须先补齐对应驱动、迁移方言、事务语义和完整回归测试。

### 11.1 表分组

#### 内容与互动

- `posts`
- `categories`
- `tags`
- `post_tags`
- `article_types`
- `article_statuses`
- `comments`
- `likes`
- `guestbook`
- `news`
- `changelog`

#### 用户与审计

- `users`
- `refresh_tokens`
- `author_applications`
- `login_logs`
- `read_logs`

#### 资源与配置

- `files`
- `network_resources`
- `settings`

#### AI

- `ai_providers`
- `ai_models`
- `ai_tools`
- `ai_agent_config`
- `ai_skills`
- `ai_tasks`
- `ai_task_logs`
- `ai_chat_sessions`
- `ai_chat_messages`

#### 迁移内部表

- `_migrations`

共 29 张应用表：28 张业务及系统表，加 1 张 `_migrations` 迁移追踪表。`likes` 目前通过 SQL 使用，没有对应 SeaORM Entity；其余主要业务表均有 Entity。

### 11.2 关系摘要

```text
users 1 ── N posts
users 1 ── N files
users 1 ── N read_logs / login_logs
posts N ── 1 categories
posts N ── N tags              (post_tags)
posts 1 ── N comments
posts 1 ── N likes
comments 1 ── N comments       (parent_id)

ai_providers 1 ── N ai_models
ai_models 1 ── N ai_agent_config
ai_skills 1 ── N ai_tasks
ai_tasks 1 ── N ai_task_logs
users 1 ── N ai_chat_sessions
ai_chat_sessions 1 ── N ai_chat_messages
```

### 11.3 迁移机制

系统有两层迁移：

1. `0000000000_init_schema.sql` 由模型初始化逻辑作为完整 SQLite batch 在事务内执行；任意非幂等错误都会回滚并阻止启动。fresh database 成功后会在同一事务中记录初始化基线与全部已嵌入增量迁移。测试逐个验证 29 张应用表和 27 个应用索引，并在排除 SQLite 内部对象后精确断言总数，因此额外对象也会导致失败；排序、规范化后的完整 `sqlite_master(type,name,sql)` 另有固定 fingerprint，覆盖 table/index/view/trigger 及全部列、类型、NULL/default、PK/UNIQUE/FK 与索引定义。固定 seed 另有独立全量 fingerprint，精确覆盖 article types/statuses、AI tools（含描述、parameters schema、enabled/config）、默认 agent config 和完整 settings 的全部稳定字段与行集合。历史升级 fixture 是剥离九个 incremental delta 后的完整 29-table 应用 schema，保留所有无关表、seed、约束和索引；升级后独立 fingerprint 与 `ai_tasks` 的 provider/skill/agent/model 四个 FK、provider `NOT NULL` 及 legacy provider value 精确断言证明原有 schema 未被改变。
2. 九个后续 SQL 文件（`0000000001`–`0000000004`、`0000000007`–`0000000011`）由 `build.rs` 按固定文件名顺序嵌入二进制。每个文件作为完整 SQLite batch 在独立事务内执行并记录到 `_migrations`；失败时 schema 与迁移记录共同回滚，重复启动只跳过已记录文件。执行器支持 trigger body、字符串内分号和嵌套 `CASE ... END`，并拒绝迁移自行发出顶层事务控制语句。

因此发布新二进制时不需要额外携带迁移工具，但部署前仍应备份数据库。

### 11.4 软删除与统计

文章、用户、分类、标签、文件和评论等核心实体普遍保留 `deleted_at`，但删除策略按资源明确区分：文章和评论软删除并保留关联数据；已被文章引用的分类/标签拒绝删除、未引用项硬删除；文件和资讯硬删除。阅读量主要从 `read_logs` 聚合，`posts.view_count` 作为历史兼容字段保留。

---

## 12. 配置系统

主配置文件为 `config.toml`，参考模板是 `config.example.toml`。

### 12.1 配置分区

```toml
data_dir = "./data"

[server]
host = "0.0.0.0"
port = 5023

[database]
url = "sqlite://./data/marksharex.db?mode=rwc"
max_connections = 10
min_connections = 1

[auth]
jwt_secret = "..."
jwt_expire_seconds = 3600
refresh_expire_seconds = 604800
encrypt_key = "..."

[storage]
upload_dir = "./data/uploads"
max_file_size = 20971520
allowed_types = ["..."]

[ai]
max_tool_rounds = 8

[ai.search]
provider = "tavily"
fallback_provider = "firecrawl"
searxng_url = ""
duckduckgo_url = ""
```

### 12.2 环境变量覆盖契约

配置优先级为“环境变量 > `config.toml`”。源码中的 `ENVIRONMENT_BINDINGS`、
`.env.example` 和 `docs/CONFIG.md` 由自动化测试要求精确一致；整数解析失败、未知或废弃
TOML 字段都会阻止启动，不再静默退回默认值。列表变量使用英文逗号分隔。

完整的 24 个环境变量、字段映射、类型、校验和兼容别名见
[`docs/CONFIG.md`](CONFIG.md)。新部署使用 `MARKSHAREX_AUTH_ENCRYPT_KEY`；
旧 `MARKSHAREX_ENCRYPT_KEY` 仅作为兼容别名保留。已删除的 `server.base_url` 不再出现在
公开示例或启动脚本中。

### 12.3 生产安全要求

- 示例不提供固定 JWT secret；启动前必须生成独立随机值。
- 必须设置并长期保存固定的加密密钥；空 secret/key 会直接阻止启动。
- 不要将真实 API Key、数据库凭据或 Token 提交到仓库。
- SQLite 数据库、上传目录和搜索索引都应置于持久卷或可靠数据目录。
- 搜索索引可以重建；数据库和上传文件必须纳入备份。

---

## 13. 构建、部署与运维

### 13.1 本地开发

```bash
# 后端
cargo run

# 前端（另一个终端）
cd frontend
npm install
npm run dev
```

默认：

- Rust 后端：`http://localhost:5023`
- Vite：`http://localhost:5173`

### 13.2 单机二进制部署

```bash
cd frontend
npm ci
npm run build
cd ..

cargo build --release --locked
./target/release/marksharex
```

生产环境通常配合 systemd 管理进程，并可在前面增加 Nginx/Caddy 处理 TLS、域名、访问日志和额外缓存策略。

严格来说，运行时除二进制外还需要：

- `config.toml`
- `static/frontend/` 前端构建产物
- 可写的数据和上传目录

Tera 模板与 SQL 迁移会编译进二进制；启动时模板会写入数据目录。前端静态文件在当前构建方式下作为独立目录复制和服务，并非全部嵌入 Rust 可执行文件。

### 13.3 Docker

Dockerfile 使用三阶段构建：

1. Node 20 Alpine 构建 Vue 前端。
2. Rust 1.95 构建 release 二进制。
3. Ubuntu 24.04 运行镜像，以非特权 `marksharex` 用户启动。

Compose 默认：

- 暴露 `5023:5023`
- 挂载命名卷 `marksharex_data:/data`
- 将数据库、上传文件、索引和运行时模板持久化到 `/data`
- 设置数据库与目录的容器内路径

健康检查访问 `/api/v1/health`。

当前 Dockerfile 更偏向已经预构建本地 base 镜像的工作流：官方 Rust/Ubuntu 基础镜像的系统依赖安装步骤被注释，但构建涉及 `magic/libmagic`，运行时健康检查又调用 `curl`。因此直接使用官方 base 执行 `docker compose up -d` 前，应实际验证依赖是否齐全；不能仅根据旧文档承诺所有环境下一定可直接构建。

### 13.4 备份与恢复

最小备份集：

```text
config.toml
{data_dir}/marksharex.db
{data_dir}/uploads/
```

可选备份：

```text
{data_dir}/search_index/
{data_dir}/templates/
```

后两者可以由程序重建。恢复后首次启动会自动执行尚未运行的迁移，并在需要时重建搜索索引。

### 13.5 可观测性

- tracing 输出终端日志
- 内存保留最近 5000 条日志供管理 API 查询
- `TraceLayer` 记录 HTTP 请求
- 管理健康接口查询数据库、资源和 uptime
- systemd/Docker 负责进程退出后的拉起

---

## 14. 安全机制与边界

当前实现包含：

- bcrypt 密码哈希
- JWT Access + Refresh Token
- X-API-Key 认证
- AES-256-GCM 加密 AI API Key
- ammonia 净化文章 Markdown HTML
- 文件 MIME/大小限制与 MD5 处理
- IP Guard
- 请求体大小限制
- gzip/Brotli 压缩
- `X-Content-Type-Options: nosniff`
- `Referrer-Policy: strict-origin-when-cross-origin`
- `X-Frame-Options: SAMEORIGIN`
- 仅采信 `trusted_proxies` 转发的 HTTPS 标记，并在所有 HTTPS 响应（包括错误响应）返回 `max-age=31536000` HSTS；默认不扩展到子域。反向代理必须覆盖客户端协议头，例如 Nginx 使用 `proxy_set_header X-Forwarded-Proto $scheme;`。
- 默认禁止跨域、可配置精确 origin allowlist 的 CORS；预检仅允许明确方法和请求头
- CSP Report-Only（含 `object-src 'none'`、`base-uri`、`frame-ancestors` 等约束）及 16 KiB 违规报告上限
- 哈希静态资源一年 immutable 缓存
- Scalar admin 保护

需要理解的边界：

- CORS 默认不返回跨域允许头；确有独立前端域名时，通过 `server.cors_allowed_origins` 或 `MARKSHAREX_SERVER_CORS_ALLOWED_ORIGINS` 配置精确 `http(s)://host[:port]`，不支持 `*`、路径、查询或片段。
- 前端路由守卫不是安全控制，后端权限判断才是最终边界。
- CSP 当前处于 Report-Only 观察期：已移除 `unsafe-eval`，但为兼容现有内联初始化脚本和样式暂时保留 `unsafe-inline`。连续至少 14 天没有需保留的违规后，应先移除内联依赖及 `unsafe-inline`，再将同一策略切换为 enforcing；切换前必须完成前台、后台、登录、编辑器和 AI 对话回归。
- `/api/v1/csp-report` 请求体限制为 16 KiB，日志字段去除控制字符并限制长度；URI 仅记录规范化 HTTP(S) 地址或 `inline`/`eval`/`wasm-eval`，其他格式统一脱敏，且 WARN 日志最多每进程每分钟 20 条，避免无限请求体、敏感参数泄露、日志注入和日志放大。
- 自定义 Markdown/HTML 展示点都应经过明确净化，不能因为内容来自管理后台就默认可信。
- 上传入口联合校验文件签名、扩展名和声明 MIME，拒绝截断 magic、伪造内容、危险历史文件名、路径分隔符和 `..`；SVG 默认 fail closed，图片在无界解码前检查尺寸上限。文件系统与数据库更新按可恢复顺序执行，并有失败回滚测试。
- AI `web_extract`、Provider 测试和聊天出站请求统一拒绝环回、私网、链路本地、云元数据及其他保留地址；DNS 结果在连接时固定，重定向目标逐跳重新校验，只有显式 allowlist 可放行指定私网地址或 CIDR。
- 用户或 LLM 生成的 Markdown 通过共享 `renderMarkdown()` 管道执行 `marked` 后再由 DOMPurify 净化；新增 `v-html` 展示点必须复用该边界，不能直接渲染未净化字符串。固定代码生成的导航图标不属于不可信 Markdown。
- 反向代理必须正确传递 `X-Forwarded-Proto` 和可信客户端 IP 头，HSTS 与 IP 审计才能按预期工作。

---

## 15. 关键业务数据流

### 15.1 发布文章

```text
作者在 Vditor 编辑 Markdown
  → 选择分类、标签、封面、类型和状态
  → POST/PUT /api/v1/posts
  → 后端解析 AuthUser，并按角色、所有者、作者转移和置顶策略授权
  → 解析 nr:{id}
  → comrak 渲染 + ammonia 净化
  → SeaORM 保存 Markdown 与 HTML
  → 维护标签关联
  → 更新 Tantivy 索引
  → published 可由 SSR/SPA 公开读取；draft 仅所有者或特权用户可读取
```

### 15.2 公开阅读

```text
浏览器请求 /post/:slug
  → Rust 强制查询已发布文章；草稿及其公共关联接口统一返回 404
  → Tera 输出完整 SSR、metadata 和 JSON-LD
  → 浏览器直接可读、爬虫可索引
  → 若从 Vue SPA 内导航，PostDetail 通过 API 异步加载
  → 仅对已发布文章记录阅读日志/时长
  → 用户可对已发布文章点赞、评论或跳转前后篇
```

### 15.3 AI 定时采集资讯

```text
Scheduler 匹配 cron
  → 加载 Task + Skill + Agent + Model
  → 创建 ToolRegistry
  → LLM 调用 web_search
  → 必要时调用 web_extract
  → LLM 整理结果
  → create_news 检查 URL 和标题重复
  → 保存草稿或已发布资讯
  → 保存逐轮 trace 和最终结果
  → 更新任务运行次数
```

### 15.4 Token 自动续期

```text
API 返回 401
  → Axios 检查 Refresh Token
  → 并发请求共享一个 refresh Promise
  → 获取新 Access/Refresh Token
  → 更新 Storage 与 Pinia
  → 重放原请求
  → 失败则清理认证并通知路由
```

---

## 16. 测试与质量保障

### 后端

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets --no-fail-fast
cargo build
```

使用 `axum-test`、Tokio 测试及模块内单元测试，覆盖 Router/OpenAPI/端点发现一致性、实时数据库角色、文章与 AI 会话权限、公开内容可见性、上传与删除补偿、迁移可靠性、SSRF、安全响应头和搜索索引一致性。测试数量会随功能演进，不在文档中固化易漂移的通过数；发布结论以当前命令的真实输出为准。

### 前端

```bash
cd frontend
npm test
npm run type-check
npm run build
npm run lint
```

`npm test` 串行执行 Node 源码/脚本契约测试和非 watch Vitest；Vitest 包含 Vue 挂载级行为测试，覆盖认证存储与 refresh、ArticleFilter/PostDetail 路由竞态、组件卸载以及评论/回复晚响应。`npm run build` 固定先执行 `vue-tsc --noEmit`，类型检查失败时不会启动 Vite。

### 文档同步门禁

```bash
python3 -m unittest discover -s scripts/tests -p 'test_*.py' -v
python3 scripts/check_doc_sync.py
```

`scripts/check_doc_sync.py` 从 Cargo、前端 package、API 权威目录、端点描述、配置绑定表和初始化 SQLite migration 读取事实，核对 README 与系统文档中的版本、数据库支持边界、API operation 数、环境变量映射以及表/索引摘要。`.github/workflows/documentation-sync.yml` 在 push 和 pull request 中执行同一检查，并运行相关 Rust 契约测试。

### 发布前建议

1. 前端测试和生产构建通过。
2. Rust 测试、格式和 release 构建通过。
3. `git diff --check` 通过。
4. 文档同步门禁通过。
5. 使用真实浏览器验证首页、文章、搜索、登录和后台核心流程。
6. 备份数据库和上传文件。
7. 部署后检查 systemd/Docker 状态、端口、健康接口和首页 HTTP 状态。

---

## 17. 系统特点总结

MarkShareX 的整体价值并不只在“能发布 Markdown”，而在于它把以下能力整合在一个轻量自托管系统中：

1. **内容平台**：文章、资讯、分类、标签、作者和更新日志。
2. **创作工作台**：在线 Markdown 编辑、本地及网络资源、导入导出。
3. **阅读站点**：SSR、SPA、全文搜索、评论、点赞、留言和阅读统计。
4. **协作后台**：四角色权限、作者申请、审核、用户与系统设置。
5. **AI 运行环境**：模型供应商、Agent、技能、工具、任务、聊天和执行追踪。
6. **轻量基础设施**：Rust/Axum、SQLite、Tantivy、本地文件和可选 Docker/systemd。

从系统边界看，SQLite 是结构化数据真相源，上传目录是二进制资源真相源，Tantivy 是可重建派生索引，Vue 是交互层，Rust 同时承担 API、SSR、安全边界和后台任务。理解这几个边界，就能快速定位绝大多数开发、排障和部署问题。

---

## 18. 快速导航

- 启动配置：`config.example.toml`
- API 在线文档：运行后访问 `/scalar`
- OpenAPI JSON：`/api/v1/openapi.json`
- 健康检查：`/api/v1/health`
- 数据库定义：`migrations/0000000000_init_schema.sql`
- 后端 API 权威目录：`src/api_endpoints.rs`
- 启动装配：`src/main.rs`
- 前端路由：`frontend/src/router/index.ts`
- 配置说明：`docs/CONFIG.md`

---

*本文档按 MarkShareX v0.4.2 当前源码整理。系统继续演进时，应优先同步版本、路由、权限边界、数据表、配置覆盖、AI 工具和部署边界。*
