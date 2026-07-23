# MarkShareX

> 轻量、自托管的 Markdown 博客与知识管理系统 — Rust + Vue 3 + SQLite

[![Rust](https://img.shields.io/badge/Rust-1.95+-orange.svg)](https://www.rust-lang.org)
[![Vue](https://img.shields.io/badge/Vue-3.x-4fc08d.svg)](https://vuejs.org)
[![Version](https://img.shields.io/badge/version-0.4.2-blue.svg)](CHANGELOG.md)
[![CI](https://github.com/XLevon/MarkShareX/actions/workflows/ci.yml/badge.svg)](https://github.com/XLevon/MarkShareX/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

MarkShareX 面向需要自行掌控内容、附件和运行数据的技术创作者。项目提供公开博客、管理后台、Markdown 编辑与导入导出、全文搜索、资源管理、评论与留言、访问统计、AI 对话和定时任务等能力。

- GitHub：<https://github.com/XLevon/MarkShareX>
- 配置参考：[docs/CONFIG.md](docs/CONFIG.md)
- 系统全貌：[docs/MarkShareX系统全貌.md](docs/MarkShareX系统全貌.md)
- 更新记录：[CHANGELOG.md](CHANGELOG.md)

## 主要能力

### 内容与阅读

- Vditor Markdown 编辑器，支持草稿、发布、分类、标签、封面、文章类型和状态。
- Markdown 与图片 ZIP 导入导出，支持 YAML Front Matter 和 CSDN 格式兼容。
- Tantivy 全文搜索，支持文章、标签和作者检索。
- 文章详情 SEO 服务端渲染，包含独立 slug、Canonical、Open Graph 等元数据。
- 评论审核、嵌套回复、点赞、阅读日志、前后篇导航和访客留言板。
- 独立资讯模块，可按题材维护、筛选、发布和撤回。

### 资源与管理

- 本地文件上传、MD5 去重、批量操作和未引用文件检查。
- 网络资源统一管理和引用溯源。
- 四类角色：`admin`、`sub_admin`、`author`、`visitor`；后端按角色、资源所有权和操作类型执行权限检查。
- 用户状态、作者申请、登录日志、阅读统计、站点设置和版本记录管理。

### AI 与自动化

- OpenAI 兼容 API Provider、模型、工具、技能、Agent、会话和定时任务管理。
- 管理后台 AI 对话和任务执行追踪。
- 可配置 Tavily、Firecrawl、SearXNG 或 DuckDuckGo 搜索 Provider 与降级 Provider。
- API Key 使用应用加密密钥加密保存；密钥必须由部署者生成并长期固定。

### 安全与工程

- JWT Access/Refresh Token、bcrypt 密码哈希和 `X-API-Key` 认证。
- Markdown/HTML 清理、请求体限制、CORS、可信代理、IP 规则和安全响应头。
- SQLite 自动初始化和增量迁移；启动时检查并按需重建 Tantivy 索引。
- Rust、前端、文档契约和容器构建由 GitHub Actions 持续验证。

## 技术栈

| 层级 | 技术 |
|---|---|
| 后端 | Rust 2021、Axum 0.7、Tokio |
| 数据与 ORM | SQLite、SeaORM |
| 搜索 | Tantivy（CJK 分词） |
| Markdown / SSR | comrak、ammonia、Tera |
| 前端 | Vue 3、TypeScript、Vite 6、Pinia |
| UI / 编辑器 | Naive UI、Vditor、Tailwind CSS 4 |
| 认证 | JWT、bcrypt、`X-API-Key` |
| 部署 | Docker Compose 或原生构建产物 |

当前版本仅支持 SQLite；PostgreSQL 和 MySQL 计划在后续版本支持。

## 快速开始

### 方式一：下载 GitHub Release

每个正式版本都会在 [GitHub Releases](https://github.com/XLevon/MarkShareX/releases) 提供已经组合好的后端、前端和示例配置：

| 平台 | 架构 | 文件后缀 |
|---|---|---|
| Linux | x86_64、ARM64 | `.tar.gz` |
| macOS | Intel x86_64、Apple Silicon ARM64 | `.tar.gz` |
| Windows | x86_64 | `.zip` |

Linux 发布包分别以 `x86_64-unknown-linux-gnu` 和 `aarch64-unknown-linux-gnu` 为目标，并在 Ubuntu 22.04 Runner 上构建；需要兼容的 glibc/GNU 用户空间，不支持 Alpine 等 musl 系统。musl 静态包将在 SQLite、Tantivy 等原生依赖完成独立验证后再提供。

下载与操作系统、架构匹配的压缩包及同名 `.sha256` 文件。Linux/macOS 示例：

```bash
sha256sum -c marksharex-v0.4.2-linux-x86_64.tar.gz.sha256
tar -xzf marksharex-v0.4.2-linux-x86_64.tar.gz
cd marksharex-v0.4.2-linux-x86_64
cp config.example.toml config.toml
openssl rand -hex 32  # 填入 config.toml 的 auth.jwt_secret
openssl rand -hex 32  # 填入 config.toml 的 auth.encrypt_key
./start.sh
```

macOS 可使用 `shasum -a 256` 对照 `.sha256` 中的摘要。Windows 可使用 `Get-FileHash -Algorithm SHA256`，解压后复制并填写 `config.toml`，然后运行 `start.cmd`。

Release 包必须从解压后的根目录启动，因为程序运行时会读取相对路径下的 `config.toml`、`static/frontend` 和 `data`。当前发布物暂未进行 Apple notarization 或 Windows Authenticode 签名；请只从本项目 GitHub Release 页面下载并核对 SHA-256。

首次启动会创建 SQLite 数据库、执行迁移并初始化搜索索引。升级已有站点前请先阅读[备份与升级](#备份与升级)。

### 方式二：Docker Compose

前置条件：Git、Docker 和 Docker Compose v2。

```bash
git clone https://github.com/XLevon/MarkShareX.git
cd MarkShareX
./scripts/start.sh
```

首次运行时，脚本会：

1. 从 `.env.example` 创建本地 `.env`；
2. 为 JWT secret 和 encryption key 生成随机值，并将 `.env` 权限设置为 `0600`；
3. 构建镜像、启动容器并等待健康检查通过。

访问 `http://localhost:5023`，按照初始化页面创建管理员。

代码更新后强制重建：

```bash
./scripts/start.sh --rebuild
```

停止服务：

```bash
docker compose down
```

数据保存在 `marksharex_data` volume。以下命令会同时删除数据库、上传文件和其他持久化数据，请谨慎使用：

```bash
docker compose down -v
```

### 方式三：原生构建

前置条件：Rust `1.95`、Node.js `22.14+`、npm、OpenSSL 和 SQLite 运行环境。

```bash
git clone https://github.com/XLevon/MarkShareX.git
cd MarkShareX

# 构建前端；产物写入 static/frontend
cd frontend
npm ci
npm run build
cd ..

# 构建后端
cargo build --release --locked

# 初始化运行配置
cp config.example.toml config.toml
cp .env.example .env
openssl rand -hex 32  # 填入 .env 的 MARKSHAREX_AUTH_JWT_SECRET
openssl rand -hex 32  # 填入 .env 的 MARKSHAREX_AUTH_ENCRYPT_KEY

# 必须从包含 config.toml 和 static/frontend 的项目目录运行
./target/release/marksharex
```

首次启动会自动创建 SQLite 数据库、执行迁移并初始化搜索索引。

### 本地开发

先按上面的方式创建 `config.toml` 和 `.env`，然后分别启动后端和 Vite：

```bash
# 终端一：后端 http://127.0.0.1:5023
cargo run

# 终端二：前端 http://127.0.0.1:5173
cd frontend
npm ci
npm run dev
```

Vite 开发服务器会把 API 请求代理到后端。完整配置字段、环境变量优先级和校验规则见 [docs/CONFIG.md](docs/CONFIG.md)。

## 配置与数据

配置优先级为：

```text
环境变量 > config.toml
```

必须为每个部署生成独立随机值：

```bash
MARKSHAREX_AUTH_JWT_SECRET="$(openssl rand -hex 32)"
MARKSHAREX_AUTH_ENCRYPT_KEY="$(openssl rand -hex 32)"
```

`MARKSHAREX_AUTH_ENCRYPT_KEY` 用于加密已保存的 API Key；部署后不得随意更换，否则既有密文将无法解密。

不要提交 `.env`、`config.toml`、数据库、上传目录、日志、搜索索引或备份。仓库中的 `.env.example` 和 `config.example.toml` 仅是无凭据模板。

### 备份与升级

升级前至少备份 SQLite 数据库、上传目录、`.env` 和实际使用的 `config.toml`。必须保留原 encryption key，否则已加密保存的 API Key 无法恢复。Docker 部署应备份 `marksharex_data` volume；Tantivy 搜索索引可以从数据库重建，但数据库和上传文件不可替代。

升级后启动新版本时会自动执行尚未应用的数据库迁移。建议先在备份副本或测试环境验证升级与回滚流程。

## API 与接口文档

共 162 个 REST API operation。`src/api_endpoints.rs` 是路由、OpenAPI paths 和端点发现信息的权威目录。

服务启动后访问：

- Scalar UI（管理员认证后）：`/scalar`
- OpenAPI JSON：`/api/v1/openapi.json`
- 健康检查：`/api/v1/health`

主要模块：

| 模块 | 路径 |
|---|---|
| 认证 | `/api/v1/auth/*` |
| 文章、分类、标签 | `/api/v1/posts/*`、`/api/v1/categories/*`、`/api/v1/tags/*` |
| 文件与网络资源 | `/api/v1/files/*`、`/api/v1/network-resources/*` |
| 评论、留言、资讯 | `/api/v1/posts/:id/comments/*`、`/api/v1/guestbook/*`、`/api/v1/news/*` |
| 用户、统计、设置 | `/api/v1/admin/*`、`/api/v1/analytics/*`、`/api/v1/settings` |
| 导入导出 | `/api/v1/import`、`/api/v1/export` |
| 更新日志 | `/api/v1/changelogs/*` |
| AI | `/api/v1/ai/*`、`/api/v1/admin/ai/*` |

公开前端路由不代表授权边界；所有敏感操作均由后端认证和授权检查保护。

## 数据库模型

共 29 张应用表：

- 内容：`posts`、`categories`、`tags`、`post_tags`、`likes`、`comments`、`guestbook`、`news`、`changelog`
- 资源：`files`、`network_resources`
- 用户：`users`、`refresh_tokens`、`author_applications`、`login_logs`、`read_logs`
- AI：`ai_providers`、`ai_models`、`ai_tools`、`ai_skills`、`ai_tasks`、`ai_task_logs`、`ai_agent_config`、`ai_chat_sessions`、`ai_chat_messages`
- 系统：`settings`、`article_types`、`article_statuses`、`_migrations`

初始 Schema 与后续增量迁移位于 `migrations/`；应用启动时自动执行尚未应用的迁移。生产升级前仍应先备份数据库和上传目录。

## 质量检查

### 后端

```bash
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --all-features --locked -- \
  -D clippy::correctness -D clippy::suspicious -A clippy::misnamed_getters
cargo test --all-targets --locked --no-fail-fast
```

### 前端

```bash
cd frontend
npm ci
npm run lint
npm test
npm run build
npm audit --audit-level=high
```

### 文档契约

```bash
python3 -m unittest discover -s scripts/tests -p 'test_*.py' -v
python3 scripts/check_doc_sync.py
```

## 参与项目

- 提交 Bug 或功能建议前请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。
- 安全漏洞必须按 [SECURITY.md](SECURITY.md) 私下报告，不要公开提交真实利用细节。
- 面向用户的版本变化记录在 [CHANGELOG.md](CHANGELOG.md)。
- GitHub 提交入口由 [Issue 模板](.github/ISSUE_TEMPLATE/) 和 [PR 模板](.github/pull_request_template.md) 提供。

## License

MarkShareX 采用 [MIT License](LICENSE)。

Copyright © 2026 XLevon.
