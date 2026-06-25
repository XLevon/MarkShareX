# Markdown Blog System (MarkShareX) — 完整需求文档

> **版本**: v1.0  
> **日期**: 2026-05-10  
> **状态**: 需求定义阶段

---

## 目录

1. [项目概述与目标](#1-项目概述与目标)
2. [整体架构设计](#2-整体架构设计)
3. [技术方案选型](#3-技术方案选型)
4. [功能模块详细设计](#4-功能模块详细设计)
5. [组件清单与依赖](#5-组件清单与依赖)
6. [安全性设计](#6-安全性设计)
7. [用户体验设计](#7-用户体验设计)
8. [数据模型设计](#8-数据模型设计)
9. [API 接口设计](#9-api-接口设计)
10. [部署方案](#10-部署方案)
11. [开发阶段与里程碑](#11-开发阶段与里程碑)
12. [风险与应对](#12-风险与应对)

---

## 1. 项目概述与目标

### 1.1 项目背景

现有 Markdown 博客系统存在明显的市场空白：

| 类型 | 代表项目 | 缺陷 |
|------|---------|------|
| 静态生成器 | Hexo, Jekyll, Zola | 无在线管理，需命令行操作 |
| 极简单文件 | markdown-blog | 无后台管理，功能匮乏 |
| 重型 CMS | Halo, Ghost, WordPress | 部署复杂，资源占用大，门槛高 |

**目标**：填补"轻量自托管 + Web 管理后台 + 文档上传"的中间地带。

### 1.2 项目定位

**MarkShareX** — 一款轻量级、自托管的 Web Markdown 博客系统，核心特性：

- **Markdown 原生**：以 MD 文件为内容源，所见即所得编辑
- **文档上传**：支持拖拽上传 .md 文件，自动解析导入
- **在线管理**：完整的 Web 管理后台，无需命令行
- **轻量部署**：单二进制 + Docker 双模式，最低 1C512M 即可运行
- **跨平台**：Web 形式，任意操作系统浏览器访问

### 1.3 目标用户

| 用户画像 | 核心需求 |
|---------|---------|
| 个人开发者 | 低成本自建博客，MD 写作习惯，不想折腾部署 |
| 技术团队 | 团队知识库，多人协作，权限管理 |
| 内容创作者 | 在线编辑发布，文档批量导入，无需技术背景 |

### 1.4 项目目标

| 维度 | 目标 |
|------|------|
| 性能 | 单服务器支撑 1000+ 日活，P99 响应 < 200ms |
| 部署 | Docker 一行命令启动，小白可用 |
| 体验 | 首屏加载 < 1.5s，编辑器零延迟感知 |
| 安全 | 通过 OWASP Top 10 全部检查项 |
| 扩展 | 插件化主题系统，API 开放 |

---

## 2. 整体架构设计

### 2.1 系统架构图

```
                         ┌──────────────────┐
                         │    用户浏览器      │
                         └────────┬─────────┘
                                  │ HTTPS
                    ┌─────────────▼──────────────┐
                    │         Nginx/Caddy         │
                    │   (反向代理 + SSL + 缓存)    │
                    └──────┬──────────────┬───────┘
                           │              │
              /api/*       │              │  /*
                           │              │
              ┌────────────▼──┐    ┌──────▼────────┐
              │  Axum 后端服务  │    │  静态资源服务  │
              │  (Rust)       │    │  Vue SPA 构建  │
              └──┬───┬───┬────┘    └───────────────┘
                 │   │   │
        ┌────────┘   │   └────────┐
        │            │            │
  ┌─────▼────┐ ┌────▼─────┐ ┌───▼───────┐
  │ SQLite/  │ │ 文件存储   │ │ 搜索引擎   │
  │ PostgreSQL│ │ (MD/媒体) │ │ (Tantivy) │
  └──────────┘ └──────────┘ └───────────┘
```

### 2.2 架构模式

采用 **前后端分离 + BFF (Backend For Frontend)** 模式：

```
┌─────────────────────────────────────────────────┐
│                  前端 SPA                        │
│  ┌───────────┐  ┌───────────┐  ┌──────────────┐ │
│  │ 博客前台   │  │ 管理后台   │  │ 安装向导     │ │
│  │ (访客访问) │  │ (博主操作) │  │ (首次部署)   │ │
│  └───────────┘  └───────────┘  └──────────────┘ │
└─────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────┐
│                  后端 API 层                     │
│  ┌───────────┐  ┌───────────┐  ┌──────────────┐ │
│  │ 公开 API   │  │ 管理 API   │  │ 系统配置 API │ │
│  │ (文章/页面)│  │ (CRUD/上传)│  │ (设置/插件)  │ │
│  └───────────┘  └───────────┘  └──────────────┘ │
└─────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────┐
│                  核心服务层                      │
│  ┌───────────┐  ┌───────────┐  ┌──────────────┐ │
│  │ MD 渲染    │  │ 文件处理   │  │ 搜索索引     │ │
│  │ (comrak)  │  │ (上传/导入)│  │ (Tantivy)   │ │
│  └───────────┘  └───────────┘  └──────────────┘ │
│  ┌───────────┐  ┌───────────┐  ┌──────────────┐ │
│  │ 主题渲染   │  │ 媒体处理   │  │ 定时任务     │ │
│  │ (Tera)    │  │ (缩略图)   │  │ (RSS/Sitemap)│ │
│  └───────────┘  └───────────┘  └──────────────┘ │
└─────────────────────────────────────────────────┘
```

### 2.3 数据流

#### 访客阅读文章

```
浏览器请求 → Nginx → Axum 路由
  → 从 DB 查询文章元数据
  → 从文件系统读取 MD 原文
  → comrak 渲染为 HTML
  → Tera 模板套用主题
  → 返回完整页面
```

#### 博主在线编辑

```
浏览器 → Vditor 编辑器 → REST API
  → Axum 接收 MD 内容
  → 校验 + 预处理（XSS过滤、frontmatter解析）
  → 写入文件系统 + 更新 DB 元数据
  → 异步更新搜索索引
  → 返回成功响应
```

#### 文档上传导入

```
拖拽 .md 文件 → 前端解析文件名/大小
  → multipart 上传 API
  → 后端解析 frontmatter + 正文
  → 去重检查（标题/日期）
  → 批量写入 DB + 文件系统
  → 返回导入结果报告
```

---

## 3. 技术方案选型

### 3.1 后端技术栈

| 组件 | 选型 | 版本 | 选型理由 |
|------|------|------|---------|
| **语言** | Rust | 1.80+ | 内存安全、零GC、极致性能、单二进制 |
| **Web 框架** | Axum | 0.8+ | Tokio 官方出品，生态最广，macro-free |
| **异步运行时** | Tokio | 1.x | 事实标准，Axum 原生依赖 |
| **MD 解析** | comrak | 0.28+ | GFM 670/670 满分，内置 frontmatter |
| **ORM** | SeaORM | 1.x | 原生 async，Entity API，与 Axum 契合 |
| **模板引擎** | Tera | 1.x | Jinja2 风格，主题模板灵活 |
| **搜索** | Tantivy | 0.22+ | Rust 原生全文搜索，性能对标 Lucene |
| **认证** | jsonwebtoken | 9.x | JWT 签发验证，无状态认证 |
| **序列化** | serde + serde_json | 1.x | Rust 生态基石 |
| **配置** | config-rs | 0.14+ | TOML/YAML/JSON 多格式配置 |
| **日志** | tracing | 0.1+ | 结构化日志，tokio 兼容 |
| **HTTP 工具** | tower-http | 0.6+ | 中间件（CORS/压缩/限流/静态文件） |
| **文件上传** | multer | 3.x | multipart 解析 |
| **图片处理** | image | 0.25+ | 缩略图生成、格式转换 |
| **RSS 生成** |rss | 2.x | RSS/Atom feed 生成 |

### 3.2 前端技术栈

| 组件 | 选型 | 选型理由 |
|------|------|---------|
| **框架** | Vue 3 + Composition API | 轻量灵活，中文生态好 |
| **构建工具** | Vite 6 | 极速 HMR，开箱即用 |
| **UI 库** | Naive UI | Vue 3 原生，Tree-shakable，中文文档完善 |
| **MD 编辑器** | Vditor | 三模式编辑，图片上传 API，国产完善 |
| **路由** | Vue Router 4 | 官方方案 |
| **状态管理** | Pinia | Vue 3 官方推荐 |
| **HTTP 客户端** | Axios | 拦截器机制完善 |
| **CSS** | TailwindCSS 4 | 原子化 CSS，主题定制灵活 |
| **图表** | ECharts (可选) | 后台数据看板 |

### 3.3 数据库选型

| 场景 | 默认 | 可扩展 |
|------|------|--------|
| 个人/小团队 | SQLite | — |
| 中大型/多用户 | — | PostgreSQL |

**SQLite 默认策略**：
- 零配置，文件即数据库，降低部署门槛
- WAL 模式支持并发读
- 博客场景读远多于写，SQLite 完全胜任
- 提供迁移脚本，一键切换 PostgreSQL

### 3.4 为什么不用 Go？

| 维度 | Rust | Go |
|------|------|-----|
| 内存安全 | 编译期保证 | GC 管理，可能抖动 |
| 二进制体积 | ~5MB（含前端） | ~15MB（含前端） |
| 运行时内存 | 5-15MB | 20-50MB |
| 并发模型 | async/await 零成本 | goroutine 轻量但有调度开销 |
| 长期运行稳定性 | 无 GC 零停顿 | GC STW 可能性 |
| 市场差异化 | **几乎无竞品** | markdown-blog 等已有方案 |

---

## 4. 功能模块详细设计

### 4.1 模块总览

```
MarkShareX
├── 🏠 博客前台（访客端）
│   ├── 首页/文章列表
│   ├── 文章详情页
│   ├── 分类/标签页
│   ├── 归档页
│   ├── 搜索
│   ├── 关于页面
│   └── RSS/Sitemap
│
├── ✏️ 管理后台（博主端）
│   ├── 仪表盘
│   ├── 文章管理
│   ├── 页面管理
│   ├── 分类与标签
│   ├── 媒体库
│   ├── 文档导入
│   ├── 主题管理
│   └── 系统设置
│
├── 🔐 认证与权限
│   ├── 用户登录/登出
│   ├── JWT 鉴权
│   ├── 角色权限 (RBAC)
│   └── 安装向导
│
├── ⚙️ 核心服务
│   ├── MD 渲染引擎
│   ├── 文件存储服务
│   ├── 搜索索引服务
│   ├── 媒体处理服务
│   └── 定时任务服务
│
└── 🔌 扩展系统
    ├── 主题引擎
    └── 插件 API（预留）
```

### 4.2 博客前台

#### 4.2.1 首页与文章列表

| 功能 | 描述 |
|------|------|
| 文章列表 | 分页展示，支持设置每页条数（10/20/50） |
| 置顶文章 | 支持文章置顶，置顶文章优先显示 |
| 文章摘要 | 自动截取摘要（优先 frontmatter 指定，否则截取前 200 字） |
| 封面图 | 支持 frontmatter 指定封面图 URL |
| 分页 | 上一页/下一页 + 页码跳转 |

#### 4.2.2 文章详情页

| 功能 | 描述 |
|------|------|
| MD 渲染 | GFM 全特性支持：表格、任务列表、脚注、数学公式 |
| 代码高亮 | 服务端渲染，支持 100+ 语言（Syntect） |
| 目录导航 | 根据标题自动生成 TOC，侧边栏固定 |
| 上下篇 | 同分类上/下一篇文章导航 |
| 相关文章 | 基于标签的相关文章推荐 |
| 阅读统计 | 文章阅读量（防刷：IP+时间窗口去重） |

#### 4.2.3 分类与标签

| 功能 | 描述 |
|------|------|
| 分类树 | 支持多级分类（父子关系） |
| 标签云 | 按文章数量加权显示标签 |
| 筛选 | 按分类/标签筛选文章列表 |

#### 4.2.4 全文搜索

| 功能 | 描述 |
|------|------|
| 实时搜索 | 输入即搜索，无需按回车 |
| 模糊匹配 | 支持标题+正文+标签检索 |
| 高亮 | 搜索结果中关键词高亮 |
| 搜索建议 | 输入时展示搜索建议下拉 |

#### 4.2.5 RSS 与 Sitemap

| 功能 | 描述 |
|------|------|
| RSS Feed | 自动生成 RSS 2.0 / Atom feed |
| Sitemap | 自动生成 sitemap.xml，SEO 友好 |
| Web Sub | 支持 WebSub 协议通知搜索引擎 |

### 4.3 管理后台

#### 4.3.1 仪表盘

| 功能 | 描述 |
|------|------|
| 数据概览 | 文章总数、评论数、阅读总量、存储占用 |
| 近期文章 | 最近 5 篇文章快速编辑入口 |
| 访问趋势 | 7 天阅读量折线图 |
| 系统信息 | 运行时间、版本、数据库大小 |

#### 4.3.2 文章管理

| 功能 | 描述 |
|------|------|
| 文章列表 | 分页/搜索/筛选（状态/分类/标签） |
| 新建文章 | Vditor 编辑器，双模式（WYSIWYG/源码） |
| 编辑文章 | 自动保存草稿（30s 间隔） |
| 发布/下架 | 草稿 → 发布 → 下架，状态流转 |
| 删除 | 软删除，回收站 30 天恢复 |
| 批量操作 | 批量修改分类/标签/状态 |
| 预览 | 全屏预览，支持主题渲染预览 |
| Frontmatter | 可视化编辑 frontmatter 元数据 |
| 定时发布 | 设置未来时间，自动发布 |

#### 4.3.3 页面管理

| 功能 | 描述 |
|------|------|
| 自定义页面 | 关于、友链等独立页面 |
| 页面模板 | 支持自定义页面模板 |
| 排序 | 拖拽排序，控制导航显示顺序 |

#### 4.3.4 媒体库

| 功能 | 描述 |
|------|------|
| 图片上传 | 拖拽/粘贴/选择上传，支持 jpg/png/gif/webp/svg |
| 自动缩略图 | 上传自动生成缩略图（300px 宽） |
| 文件管理 | 列表/网格视图，按类型/日期筛选 |
| 图片编辑 | 基础裁剪/旋转（前端实现） |
| 批量上传 | 多文件同时上传，显示进度 |
| 引用关系 | 图片被哪篇文章引用，防止误删 |
| 存储配额 | 可配置存储限额，超限警告 |

#### 4.3.5 文档导入

| 功能 | 描述 |
|------|------|
| MD 文件上传 | 拖拽/选择 .md 文件，支持批量 |
| ZIP 包导入 | 上传包含多个 .md 文件的 ZIP 包，批量解压导入 |
| Frontmatter 解析 | 自动读取 YAML/TOML frontmatter 元数据 |
| 去重检测 | 基于标题或 slug 检测重复，提示用户选择覆盖/跳过/重命名 |
| 导入报告 | 导入完成后生成报告：成功 N 篇，跳过 N 篇，失败 N 篇 |
| HTML → MD | （V2）支持 HTML 文件导入，自动转换为 Markdown |
| 第三方导入 | （V2）从 Hexo/Jekyll/WordPress 导入 |

#### 4.3.6 主题管理

| 功能 | 描述 |
|------|------|
| 主题切换 | 内置 2-3 个默认主题，一键切换 |
| 主题配置 | 可视化配置主题参数（颜色/布局/字体） |
| 实时预览 | 修改配置即时预览效果 |
| 自定义主题 | 支持上传主题 ZIP 包（Tera 模板 + CSS/JS） |

#### 4.3.7 系统设置

| 功能 | 描述 |
|------|------|
| 基本设置 | 站点名称/描述/关键词/Logo/Favicon |
| 日期格式 | 时区选择，日期显示格式 |
| 评论设置 | 开关/审核策略/关键词过滤 |
| SEO 设置 | 全局 meta/robots/自定义 head 代码 |
| 邮件设置 | SMTP 配置（通知邮件） |
| 备份恢复 | 一键导出/导入全部数据 |
| 缓存管理 | 清除渲染缓存/搜索索引重建 |

### 4.4 认证与权限

#### 4.4.1 用户管理

| 角色 | 权限 |
|------|------|
| **管理员** | 全部权限：文章/页面/媒体/主题/设置/用户 |
| **编辑** | 文章/页面/媒体读写，无法管理设置和用户 |
| **作者** | 仅管理自己文章，可上传媒体 |
| **访客** | 仅前台浏览（未来支持注册评论时扩展） |

#### 4.4.2 安装向导

首次访问时自动进入安装向导：

```
步骤 1: 欢迎页 → 步骤 2: 创建管理员账号 → 步骤 3: 站点基本设置
→ 步骤 4: 选择默认主题 → 步骤 5: 完成安装
```

安装完成后创建 `install.lock` 文件，后续不再触发。

### 4.5 核心服务

#### 4.5.1 MD 渲染引擎

| 特性 | 实现 |
|------|------|
| GFM 全特性 | comrak 开启全部 GFM 扩展 |
| 代码高亮 | 服务端 Syntect 渲染，无需前端 JS |
| 数学公式 |KaTeX 服务端渲染或前端渲染（可配置） |
| Mermaid 图表 | 前端 mermaid.js 渲染 |
| 脚注 | comrak footnotes 扩展 |
| 自动链接 | URL 自动转为可点击链接 |
| Emoji | :emoji_name: → Unicode emoji |
| 目录生成 | 提取标题生成 TOC JSON |

#### 4.5.2 文件存储服务

| 特性 | 实现 |
|------|------|
| 存储结构 | `data/posts/{year}/{slug}.md` |
| 媒体目录 | `data/uploads/{year}/{month}/{filename}` |
| 缩略图 | `data/uploads/.thumbs/{size}/{filename}` |
| 类型限制 | 白名单扩展名：jpg/png/gif/webp/svg/mp4/pdf/zip |
| 大小限制 | 单文件 10MB（可配置） |
| 命名策略 | 原始文件名 + 时间戳哈希防冲突 |

#### 4.5.3 搜索索引服务

| 特性 | 实现 |
|------|------|
| 引擎 | Tantivy 全文搜索 |
| 索引字段 | 标题（权重 3x）、正文（权重 1x）、标签（权重 2x） |
| 增量更新 | 文章发布/更新时异步重建单条索引 |
| 全量重建 | 管理后台手动触发，后台任务执行 |
| 中文分词 | jieba-rs 分词器 |

#### 4.5.4 定时任务服务

| 任务 | 周期 |
|------|------|
| RSS/Sitemap 生成 | 文章变更时触发 |
| 草稿自动保存 | 每 30s（前端触发） |
| 回收站清理 | 每日 0 点 |
| 搜索索引优化 | 每周 |
| 访问统计归档 | 每日 2 点 |

---

## 5. 组件清单与依赖

### 5.1 后端 Rust 依赖 (Cargo.toml)

```toml
[dependencies]
# Web 框架
axum = "0.8"
axum-extra = { version = "0.10", features = ["typed-header", "cookie"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "compression-gzip", "trace", "limit", "serve-dir"] }

# Markdown
comrak = { version = "0.28", features = ["shortcodes"] }
syntect = "5"

# 数据库
sea-orm = { version = "1", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-rustls", "macros"] }
sea-orm-migration = "1"

# 模板
tera = "1"

# 搜索
tantivy = "0.22"
jieba-rs = "0.7"

# 认证
jsonwebtoken = "9"
argon2 = "0.5"

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 配置
config = "0.14"

# 文件处理
multer = "3"
image = "0.25"
zip = "2"
uuid = { version = "1", features = ["v4"] }

# 日志
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# 工具
chrono = { version = "0.4", features = ["serde"] }
url = "2"
slug = "0.1"
rss = "2"
sitemap = "0.4"

# 图片处理（缩略图）
fast-image-resize = "5"
```

### 5.2 前端依赖 (package.json)

```json
{
  "dependencies": {
    "vue": "^3.5",
    "vue-router": "^4.4",
    "pinia": "^3",
    "axios": "^1.7",
    "naive-ui": "^2.40",
    "vditor": "^3.10",
    "tailwindcss": "^4"
  },
  "devDependencies": {
    "vite": "^6",
    "@vitejs/plugin-vue": "^5",
    "typescript": "^5.5",
    "eslint": "^9",
    "prettier": "^3"
  }
}
```

### 5.3 项目目录结构

```
marksharex/
├── Cargo.toml                      # Rust 工作空间根
├── Cargo.lock
├── Dockerfile
├── docker-compose.yml
├── README.md
│
├── server/                         # 后端 Axum 服务
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                 # 入口：启动服务
│       ├── config.rs               # 配置加载
│       ├── app.rs                  # Axum 路由注册
│       ├── error.rs                # 统一错误处理
│       │
│       ├── handler/                # API 处理器
│       │   ├── mod.rs
│       │   ├── auth.rs             # 登录/登出/刷新 Token
│       │   ├── post.rs             # 文章 CRUD
│       │   ├── page.rs             # 页面 CRUD
│       │   ├── category.rs         # 分类管理
│       │   ├── tag.rs              # 标签管理
│       │   ├── media.rs            # 媒体上传/管理
│       │   ├── import.rs           # 文档导入
│       │   ├── search.rs           # 搜索
│       │   ├── theme.rs            # 主题管理
│       │   ├── setting.rs          # 系统设置
│       │   ├── user.rs             # 用户管理
│       │   └── install.rs          # 安装向导
│       │
│       ├── service/                # 业务逻辑层
│       │   ├── mod.rs
│       │   ├── post.rs
│       │   ├── page.rs
│       │   ├── media.rs
│       │   ├── import.rs
│       │   ├── search.rs
│       │   ├── render.rs           # MD 渲染
│       │   └── backup.rs
│       │
│       ├── model/                  # SeaORM 实体
│       │   ├── mod.rs
│       │   ├── user.rs
│       │   ├── post.rs
│       │   ├── page.rs
│       │   ├── category.rs
│       │   ├── tag.rs
│       │   ├── media.rs
│       │   └── setting.rs
│       │
│       ├── middleware/             # 中间件
│       │   ├── mod.rs
│       │   ├── auth.rs             # JWT 验证
│       │   ├── rbac.rs             # 权限检查
│       │   └── install_guard.rs    # 安装检测
│       │
│       ├── migration/              # 数据库迁移
│       │   ├── mod.rs
│       │   ├── m20260510_000001_init.rs
│       │   └── ...
│       │
│       └── util/                   # 工具函数
│           ├── mod.rs
│           ├── slug.rs
│           ├── pagination.rs
│           └── validator.rs
│
├── web/                            # 前端 Vue 项目
│   ├── package.json
│   ├── vite.config.ts
│   ├── tailwind.config.js
│   ├── tsconfig.json
│   │
│   ├── public/
│   │   └── favicon.ico
│   │
│   └── src/
│       ├── main.ts
│       ├── App.vue
│       │
│       ├── views/                  # 页面视图
│       │   ├── frontend/           # 博客前台
│       │   │   ├── Home.vue
│       │   │   ├── PostDetail.vue
│       │   │   ├── Category.vue
│       │   │   ├── Tag.vue
│       │   │   ├── Archive.vue
│       │   │   ├── Search.vue
│       │   │   └── About.vue
│       │   │
│       │   └── admin/              # 管理后台
│       │       ├── Dashboard.vue
│       │       ├── PostList.vue
│       │       ├── PostEditor.vue
│       │       ├── PageList.vue
│       │       ├── PageEditor.vue
│       │       ├── MediaLib.vue
│       │       ├── Importer.vue
│       │       ├── ThemeSetting.vue
│       │       ├── SystemSetting.vue
│       │       ├── UserList.vue
│       │       └── Login.vue
│       │
│       ├── components/             # 通用组件
│       │   ├── MdEditor.vue        # Vditor 封装
│       │   ├── FileUploader.vue    # 文件上传组件
│       │   ├── Pagination.vue
│       │   ├── TagInput.vue
│       │   ├── SearchBox.vue
│       │   └── ConfirmDialog.vue
│       │
│       ├── stores/                 # Pinia 状态
│       │   ├── auth.ts
│       │   ├── post.ts
│       │   └── setting.ts
│       │
│       ├── api/                    # API 调用封装
│       │   ├── request.ts          # Axios 实例 + 拦截器
│       │   ├── auth.ts
│       │   ├── post.ts
│       │   ├── media.ts
│       │   └── ...
│       │
│       ├── router/
│       │   └── index.ts
│       │
│       ├── layouts/
│       │   ├── FrontendLayout.vue
│       │   └── AdminLayout.vue
│       │
│       └── styles/
│           └── global.css
│
├── themes/                         # 主题目录
│   ├── default/
│   │   ├── theme.toml
│   │   ├── templates/
│   │   │   ├── base.html
│   │   │   ├── index.html
│   │   │   ├── post.html
│   │   │   ├── page.html
│   │   │   ├── category.html
│   │   │   ├── tag.html
│   │   │   ├── archive.html
│   │   │   └── search.html
│   │   ├── static/
│   │   │   ├── css/
│   │   │   └── js/
│   │   └── screenshot.png
│   │
│   └── minimal/                    # 极简主题
│       └── ...
│
├── data/                           # 运行时数据（.gitignore）
│   ├── marksharex.db                   # SQLite 数据库
│   ├── posts/                      # MD 文章源文件
│   ├── uploads/                    # 媒体文件
│   ├── search_index/               # Tantivy 索引
│   └── install.lock                # 安装锁
│
└── docs/                           # 项目文档
    ├── REQUIREMENTS.md             # 本文档
    ├── API.md                      # API 文档
    ├── THEME.md                    # 主题开发指南
    └── DEPLOYMENT.md               # 部署指南
```

---

## 6. 安全性设计

### 6.1 OWASP Top 10 防护

| OWASP 风险 | 防护措施 |
|------------|---------|
| **A01 权限控制失效** | JWT + RBAC，每个 API 校验角色权限；安装向导锁文件防重复创建 |
| **A02 加密失败** | 密码 Argon2id 哈希；HTTPS 强制；敏感配置环境变量注入 |
| **A03 注入** | SeaORM 参数化查询，杜绝 SQL 注入；模板引擎自动转义，防 SSTI |
| **A04 不安全设计** | 最小权限原则；默认安全配置；Rate Limiting 防暴力破解 |
| **A05 安全配置错误** | 安装向导自动生成安全配置；默认关闭调试模式；CORS 白名单 |
| **A06 易受攻击组件** | Cargo audit CI 集成；dependabot 自动更新 |
| **A07 身份认证失败** | JWT 短期(15min) + Refresh Token(7d)；登录失败限流(5次/15min) |
| **A08 数据完整性失败** | 文件上传校验 MIME type + Magic bytes；Cargo lock 锁定版本 |
| **A09 日志监控不足** | tracing 结构化日志；认证事件全量记录；异常访问告警 |
| **A10 服务端请求伪造** | 无 SSRF 场景；webhook URL 白名单；禁止内网地址请求 |

### 6.2 文件上传安全

| 风险 | 防护 |
|------|------|
| 恶意文件类型 | 双重校验：扩展名白名单 + Magic bytes 检测实际类型 |
| 文件名注入 | 统一重命名：`{timestamp}_{hash}.{ext}`，禁止保留原始文件名入文件系统 |
| 路径穿越 | 文件名过滤 `../`，存储路径不拼接用户输入 |
| 文件大小 | 全局限制 10MB，multipart 解析层硬限制 20MB |
| SVG XSS | SVG 文件强制 sanitize（xmpp-sanitize 或 html5ever 解析后过滤 script/event） |
| ZIP 炸弹 | ZIP 导入限制解压总大小 100MB，单文件数 1000 |
| 存储耗尽 | 可配置存储配额，超限拒绝上传并告警 |

### 6.3 XSS 防护

| 层级 | 措施 |
|------|------|
| **MD 渲染** | comrak 输出 HTML 经 ammonia 库过滤，白名单标签+属性 |
| **前端渲染** | Vue v-html 仅用于服务端已过滤的 HTML，不渲染用户原始输入 |
| **CSP** | Content-Security-Policy 头：禁止 inline script，限制 style-src |
| **Cookie** | HttpOnly + Secure + SameSite=Strict |

### 6.4 CSRF 防护

| 措施 | 实现 |
|------|------|
| SameSite Cookie | SameSite=Strict |
| 自定义Header | 管理API要求 `X-Requested-With: XMLHttpRequest` |
| Origin 校验 | 验证请求 Origin/Header Referer |

### 6.5 认证与会话安全

```
登录流程：
1. POST /api/auth/login { username, password }
2. 验证 Argon2id 哈希
3. 签发 access_token (15min) + refresh_token (7d)
4. 返回 HttpOnly Secure Cookie

Token 刷新：
1. access_token 过期
2. POST /api/auth/refresh { refresh_token }
3. 验证 refresh_token 有效性
4. 签发新 access_token

安全策略：
- refresh_token 一次一用，使用后轮换
- 修改密码后全部 token 失效（token_version 机制）
- 同一用户最多 5 个有效 refresh_token
- 登出时 token 加入黑名单（内存 Set，TTL = token 过期时间）
```

### 6.6 速率限制

| 接口 | 限制 | 策略 |
|------|------|------|
| 登录 | 5 次 / 15 分钟 / IP | 超限锁定 30 分钟 |
| 文件上传 | 20 次 / 小时 / 用户 | 超限返回 429 |
| API 通用 | 100 次 / 分钟 / 用户 | 超限返回 429 |
| 搜索 | 30 次 / 分钟 / IP | 超限返回 429 |
| 注册（未来） | 3 次 / 小时 / IP | 超限返回 429 |

### 6.7 数据保护

| 数据类型 | 存储方式 | 加密 |
|---------|---------|------|
| 用户密码 | Argon2id 哈希 | 单向，不可逆 |
| JWT Secret | 环境变量 | 不入库 |
| 数据库连接串 | 环境变量 | 不入代码仓库 |
| Refresh Token | 数据库 | Argon2id 哈希存储 |
| 用户邮箱 | 数据库 | 可选对称加密（AES-256-GCM） |

---

## 7. 用户体验设计

### 7.1 设计原则

| 原则 | 描述 |
|------|------|
| **零配置启动** | Docker 一行命令，首次访问自动进入安装向导 |
| **渐进式复杂** | 基础用法 3 步内完成，高级功能按需展开 |
| **即时反馈** | 每个操作 < 300ms 反馈，加载状态明确 |
| **防丢失** | 自动保存草稿，关闭页面前未保存提醒 |
| **键盘优先** | 编辑器全键盘操作，管理后台快捷键 |

### 7.2 关键交互流程

#### 7.2.1 首次安装

```
[欢迎页]                    [创建管理员]
┌──────────────────────┐    ┌──────────────────────┐
│  🎉 欢迎使用 MarkShareX    │    │  创建管理员账号        │
│                      │    │                      │
│  3 步完成安装          │    │  用户名: [________]   │
│                      │    │  邮  箱: [________]   │
│  [开始安装 →]         │    │  密  码: [________]   │
└──────────────────────┘    │  确  认: [________]   │
                            │                      │
                            │  [下一步 →]           │
                            └──────────────────────┘

[站点设置]                   [完成]
┌──────────────────────┐    ┌──────────────────────┐
│  站点名称: [________] │    │  ✅ 安装完成！        │
│  站点描述: [________] │    │                      │
│  时    区: [▼ UTC+8 ] │    │  开始写第一篇文章？    │
│                      │    │                      │
│  默认主题: [▼ 卡片 ]  │    │  [进入后台]  [写文章]  │
│                      │    │                      │
│  [完成安装]           │    └──────────────────────┘
└──────────────────────┘
```

#### 7.2.2 文章编辑

```
┌──────────────────────────────────────────────────┐
│ ← 返回  │  文章标题: [________________] │ 💾 已保存 │
├──────────┼───────────────────────────────────────┤
│          │                                       │
│ 📝 编辑  │  ┌──────────────────────────────────┐  │
│ 📊 属性  │  │                                  │  │
│ 📷 媒体  │  │     Vditor 编辑区域               │  │
│ ⚙ 设置  │  │                                  │  │
│          │  │                                  │  │
│ ─────── │  │                                  │  │
│ 📋 分类  │  └──────────────────────────────────┘  │
│ 技术随笔 │                                       │
│          ├───────────────────────────────────────┤
│ 🏷 标签  │  预览  │  源码  │  分屏               │
│ Rust     │ ┌──────────────────────────────────┐   │
│ Web      │ │  渲染后的文章预览...              │   │
│          │ └──────────────────────────────────┘   │
│ ─────── │                                       │
│ 📌 状态  │                                       │
│ ○ 草稿   │                                       │
│ ● 发布   │                                       │
│          │                                       │
│ ─────── │                                       │
│ [发布]   │                                       │
│ [预览]   │                                       │
└──────────┴───────────────────────────────────────┘
```

#### 7.2.3 文档导入

```
┌──────────────────────────────────────────────────┐
│  📂 文档导入                                      │
├──────────────────────────────────────────────────┤
│                                                  │
│  ┌──────────────────────────────────────────────┐│
│  │                                              ││
│  │     📎 拖拽 .md / .zip 文件到此处            ││
│  │        或 [点击选择文件]                       ││
│  │                                              ││
│  │     支持 .md / .zip，单次最多 50 个文件        ││
│  └──────────────────────────────────────────────┘│
│                                                  │
│  上传文件列表:                                    │
│  ┌────────────────────────────────────────────┐  │
│  │ ✅ rust-async.md      2.3KB  已解析         │  │
│  │ ✅ web-framework.md   5.1KB  已解析         │  │
│  │ ⚠️  duplicate.md      1.8KB  标题重复 ▼     │  │
│  │    → 覆盖 / 跳过 / 重命名                   │  │
│  │ ❌ broken.md          0.2KB  格式异常        │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
│  默认分类: [▼ 技术随笔]                            │
│  默认标签: [+ 添加标签]                             │
│  默认状态: ○ 草稿  ● 发布                          │
│                                                  │
│                  [导入 3 篇文章]                    │
└──────────────────────────────────────────────────┘
```

### 7.3 响应式设计

| 断点 | 设备 | 布局调整 |
|------|------|---------|
| ≥1200px | 桌面 | 侧边栏+主内容双栏 |
| 768-1199px | 平板 | 侧边栏折叠，抽屉式 |
| <768px | 手机 | 底部导航栏，全屏编辑器 |

### 7.4 性能体验

| 指标 | 目标 | 实现方式 |
|------|------|---------|
| 首屏加载 | < 1.5s | SSR 服务端渲染 + Gzip/Brotli |
| 路由切换 | < 200ms | 前端路由 + 预加载 |
| 编辑器输入 | 0 延迟感知 | Vditor 本地渲染 + 30s 自动保存 |
| 搜索响应 | < 300ms | Tantivy 内存索引 |
| 图片加载 | WebP 优先 + 懒加载 | picture 标签 + Intersection Observer |
| 管理后台 | < 2s 首屏 | SPA 懒加载路由 + Skeleton |

### 7.5 无障碍 (Accessibility)

| 要素 | 实现 |
|------|------|
| 语义化 HTML | 正确使用 header/nav/main/article/footer |
| ARIA 标签 | 动态组件添加 aria-label |
| 键盘导航 | Tab 序列合理，编辑器快捷键 |
| 对比度 | 最低 4.5:1 (WCAG AA) |
| 焦点管理 | 路由切换后焦点移至主内容区 |
| 屏幕阅读器 | alt 文本、aria-live 区域 |

### 7.6 国际化 (i18n)

| 策略 | 实现 |
|------|------|
| 前端 | vue-i18n，默认中文 + 英文 |
| 后端 | 错误码国际化，accept-language 头 |
| 主题模板 | Tera 模板 i18n 变量 |
| 时间格式 | 按用户时区格式化 |

---

## 8. 数据模型设计

### 8.1 ER 图

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│  users   │     │  posts   │     │  pages   │
├──────────┤     ├──────────┤     ├──────────┤
│ id (PK)  │←──┐│ id (PK)  │     │ id (PK)  │
│ username │   ││ author_id│──┘  │ title    │
│ email    │   ││ title    │     │ slug     │
│ password │   ││ slug     │     │ content  │
│ role     │   ││ content  │     │ status   │
│ avatar   │   ││ summary  │     │ template │
│ token_ver│   ││ cover    │     │ sort_order│
│ created  │   ││ status   │     │ created  │
│ updated  │   ││ is_top   │     │ updated  │
└──────────┘   ││ published│     └──────────┘
               ││ allow_comment│
     ┌─────────┘│ created  │
     │          │ updated  │
     │          └────┬─────┘
     │               │
     │    ┌──────────┼──────────┐
     │    │          │          │
     │  ┌─▼────┐  ┌─▼────┐  ┌──▼───────┐
     │  │post_  │  │post_  │  │post_     │
     │  │categor│  │tag   │  │metas     │
     │  │ies    │  │s     │  │(KV扩展)  │
     │  ├───────┤  ├──────┤  ├──────────┤
     │  │post_id│  │post_id│  │ post_id  │
     │  │cat_id │  │tag_id│  │ key      │
     │  └───┬───┘  └──┬───┘  │ value    │
     │      │         │      └──────────┘
     │  ┌───▼───┐  ┌──▼────┐
     │  │categor│  │ tags  │
     │  │ies    │  ├───────┤
     │  ├───────┤  │id (PK)│
     │  │id (PK)│  │name   │
     │  │name   │  │slug   │
     │  │slug   │  │count  │
     │  │parent │  └───────┘
     │  │sort   │
     │  └───────┘
     │
   ┌─▼────────┐     ┌──────────┐     ┌──────────┐
   │ categories│     │  media   │     │ settings │
   ├──────────┤     ├──────────┤     ├──────────┤
   │ id (PK)  │     │ id (PK)  │     │ key (PK) │
   │ name     │     │ filename │     │ value    │
   │ slug     │     │ path     │     │ category │
   │ parent_id│     │ mime_type│     └──────────┘
   │ sort_order│    │ size     │
   │ count    │     │ width    │
   └──────────┘     │ height   │
                    │ alt_text │
                    │ uploader │
                    │ created  │
                    └──────────┘
```

### 8.2 核心表结构

#### users

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | INTEGER | PK, AUTO | 用户 ID |
| username | VARCHAR(50) | UNIQUE, NOT NULL | 用户名 |
| email | VARCHAR(255) | UNIQUE, NOT NULL | 邮箱 |
| password_hash | VARCHAR(255) | NOT NULL | Argon2id 哈希 |
| role | VARCHAR(20) | NOT NULL, DEFAULT 'author' | 角色：admin/editor/author |
| avatar | VARCHAR(500) | | 头像 URL |
| token_version | INTEGER | DEFAULT 0 | Token 版本（密码修改时+1） |
| status | VARCHAR(20) | DEFAULT 'active' | active/disabled |
| created_at | TIMESTAMP | NOT NULL | 创建时间 |
| updated_at | TIMESTAMP | NOT NULL | 更新时间 |

#### posts

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | INTEGER | PK, AUTO | 文章 ID |
| author_id | INTEGER | FK → users.id | 作者 |
| title | VARCHAR(255) | NOT NULL | 标题 |
| slug | VARCHAR(255) | UNIQUE, NOT NULL | URL 友好标识 |
| content | TEXT | | MD 原文 |
| summary | VARCHAR(500) | | 摘要 |
| cover | VARCHAR(500) | | 封面图 URL |
| status | VARCHAR(20) | NOT NULL | draft/published/archived/trash |
| is_top | BOOLEAN | DEFAULT false | 是否置顶 |
| allow_comment | BOOLEAN | DEFAULT true | 允许评论 |
| published_at | TIMESTAMP | | 发布时间（定时发布） |
| view_count | INTEGER | DEFAULT 0 | 阅读量 |
| created_at | TIMESTAMP | NOT NULL | 创建时间 |
| updated_at | TIMESTAMP | NOT NULL | 更新时间 |
| deleted_at | TIMESTAMP | | 软删除时间 |

#### categories

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | INTEGER | PK, AUTO | 分类 ID |
| name | VARCHAR(100) | NOT NULL | 分类名 |
| slug | VARCHAR(100) | UNIQUE | URL 标识 |
| parent_id | INTEGER | FK → categories.id, NULLABLE | 父分类 |
| sort_order | INTEGER | DEFAULT 0 | 排序 |
| description | VARCHAR(500) | | 描述 |

#### tags

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | INTEGER | PK, AUTO | 标签 ID |
| name | VARCHAR(100) | UNIQUE, NOT NULL | 标签名 |
| slug | VARCHAR(100) | UNIQUE | URL 标识 |

#### media

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | INTEGER | PK, AUTO | 媒体 ID |
| filename | VARCHAR(255) | NOT NULL | 存储文件名 |
| original_name | VARCHAR(255) | | 原始文件名 |
| path | VARCHAR(500) | NOT NULL | 存储路径 |
| mime_type | VARCHAR(100) | NOT NULL | MIME 类型 |
| size | BIGINT | NOT NULL | 文件大小(bytes) |
| width | INTEGER | | 图片宽度 |
| height | INTEGER | | 图片高度 |
| alt_text | VARCHAR(255) | | 替代文本 |
| uploader_id | INTEGER | FK → users.id | 上传者 |
| created_at | TIMESTAMP | NOT NULL | 上传时间 |

#### settings

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| key | VARCHAR(100) | PK | 配置键 |
| value | TEXT | | 配置值 (JSON) |
| category | VARCHAR(50) | | 分组：general/seo/email/theme |

#### refresh_tokens

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| id | INTEGER | PK, AUTO | ID |
| user_id | INTEGER | FK → users.id | 用户 |
| token_hash | VARCHAR(255) | NOT NULL | Token 哈希 |
| expires_at | TIMESTAMP | NOT NULL | 过期时间 |
| created_at | TIMESTAMP | NOT NULL | 创建时间 |

---

## 9. API 接口设计

### 9.1 接口规范

| 规范 | 说明 |
|------|------|
| 风格 | RESTful |
| 前缀 | `/api/v1` |
| 认证 | Bearer Token (JWT) |
| 分页 | `?page=1&per_page=20` |
| 排序 | `?sort=created_at&order=desc` |
| 筛选 | `?status=published&category_id=1` |
| 搜索 | `?q=keyword` |

### 9.2 统一响应格式

```json
// 成功
{
  "code": 0,
  "message": "ok",
  "data": { ... }
}

// 列表（带分页）
{
  "code": 0,
  "message": "ok",
  "data": {
    "items": [ ... ],
    "total": 100,
    "page": 1,
    "per_page": 20
  }
}

// 错误
{
  "code": 40001,
  "message": "参数校验失败",
  "errors": [
    { "field": "title", "message": "标题不能为空" }
  ]
}
```

### 9.3 核心接口一览

#### 认证

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| POST | /api/v1/auth/login | 登录 | ❌ |
| POST | /api/v1/auth/logout | 登出 | ✅ |
| POST | /api/v1/auth/refresh | 刷新 Token | ❌ (需 refresh_token) |
| GET | /api/v1/auth/me | 当前用户信息 | ✅ |
| PUT | /api/v1/auth/password | 修改密码 | ✅ |

#### 文章

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | /api/v1/posts | 文章列表（公开，仅 published） | ❌ |
| GET | /api/v1/posts/:slug | 文章详情 | ❌ |
| GET | /api/v1/admin/posts | 全部文章（含草稿） | ✅ admin/editor |
| POST | /api/v1/admin/posts | 创建文章 | ✅ |
| PUT | /api/v1/admin/posts/:id | 更新文章 | ✅ |
| DELETE | /api/v1/admin/posts/:id | 删除文章（软删除） | ✅ |
| POST | /api/v1/admin/posts/batch | 批量操作 | ✅ admin |

#### 文档导入

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| POST | /api/v1/admin/import/md | 上传 MD 文件导入 | ✅ |
| POST | /api/v1/admin/import/zip | 上传 ZIP 包导入 | ✅ |
| GET | /api/v1/admin/import/status/:task_id | 导入进度查询 | ✅ |

#### 媒体

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | /api/v1/admin/media | 媒体列表 | ✅ |
| POST | /api/v1/admin/media/upload | 上传文件 | ✅ |
| DELETE | /api/v1/admin/media/:id | 删除文件 | ✅ admin |

#### 搜索

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | /api/v1/search?q=keyword | 全文搜索 | ❌ |
| GET | /api/v1/search/suggest?q=prefix | 搜索建议 | ❌ |

#### 分类/标签

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | /api/v1/categories | 分类列表 | ❌ |
| GET | /api/v1/tags | 标签列表 | ❌ |
| POST | /api/v1/admin/categories | 创建分类 | ✅ admin |
| PUT | /api/v1/admin/categories/:id | 更新分类 | ✅ admin |
| DELETE | /api/v1/admin/categories/:id | 删除分类 | ✅ admin |
| POST | /api/v1/admin/tags | 创建标签 | ✅ |
| DELETE | /api/v1/admin/tags/:id | 删除标签 | ✅ admin |

#### 系统设置

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | /api/v1/settings | 公开设置 | ❌ |
| GET | /api/v1/admin/settings | 全部设置 | ✅ admin |
| PUT | /api/v1/admin/settings | 更新设置 | ✅ admin |
| POST | /api/v1/admin/backup/export | 导出备份 | ✅ admin |
| POST | /api/v1/admin/backup/import | 导入备份 | ✅ admin |
| POST | /api/v1/admin/cache/clear | 清除缓存 | ✅ admin |

#### 安装

| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | /api/v1/install/check | 检查是否已安装 | ❌ |
| POST | /api/v1/install | 执行安装 | ❌ (仅首次) |

---

## 10. 部署方案

### 10.1 Docker 部署（推荐）

```yaml
# docker-compose.yml
version: '3.8'
services:
  marksharex:
    image: marksharex/marksharex:latest
    container_name: marksharex
    ports:
      - "8080:8080"
    volumes:
      - marksharex-data:/app/data
    environment:
      - MARKSHAREX_JWT_SECRET=${JWT_SECRET}
      - MARKSHAREX_DB_URL=sqlite:///app/data/marksharex.db
      - RUST_LOG=marksharex=info
    restart: unless-stopped

volumes:
  marksharex-data:
```

一键启动：
```bash
curl -sL https:// marksharex.dev/install.sh | bash
# 或
docker compose up -d
```

### 10.2 二进制部署

```bash
# 下载
wget https://github.com/user/marksharex/releases/latest/download/marksharex-linux-amd64
chmod +x marksharex-linux-amd64
mv marksharex-linux-amd64 /usr/local/bin/marksharex

# 运行
export MARKSHAREX_JWT_SECRET=$(openssl rand -hex 32)
marksharex serve --port 8080 --data-dir /var/lib/marksharex
```

### 10.3 系统要求

| 配置 | 最低 | 推荐 |
|------|------|------|
| CPU | 1 核 | 2 核 |
| 内存 | 512MB | 1GB |
| 磁盘 | 1GB | 5GB+ |
| 系统 | Linux x86_64/ARM64 | Ubuntu 22.04+ |
| Docker | 20.10+ | 24.0+ |

---

## 11. 开发阶段与里程碑

### Phase 0: 基础搭建（1 周）

| 任务 | 产出 |
|------|------|
| 初始化 Rust workspace + Vue 项目 | 项目骨架可编译 |
| SeaORM 实体定义 + 数据库迁移 | 数据库可初始化 |
| Axum 基础路由 + 中间件 | 健康检查接口可用 |
| 前端 Vue3 + Naive UI + Vite 脚手架 | 前端可启动 |

### Phase 1: MVP 核心功能（4-6 周）

| 模块 | 功能 | 优先级 |
|------|------|--------|
| 认证 | 安装向导 + 登录/登出 + JWT | 🔴 P0 |
| 文章 | CRUD + MD 渲染 + 发布 | 🔴 P0 |
| 编辑器 | Vditor 集成 + 自动保存 | 🔴 P0 |
| 前台 | 首页列表 + 文章详情 + 分页 | 🔴 P0 |
| 分类标签 | 分类树 + 标签管理 | 🟡 P1 |
| 媒体 | 图片上传 + 缩略图 | 🟡 P1 |

### Phase 2: 完善体验（3-4 周）

| 模块 | 功能 | 优先级 |
|------|------|--------|
| 文档导入 | MD/ZIP 上传 + 解析 + 去重 | 🟡 P1 |
| 搜索 | Tantivy 全文搜索 + 中文分词 | 🟡 P1 |
| 主题 | Tera 模板引擎 + 1-2 个默认主题 | 🟡 P1 |
| 用户管理 | RBAC 权限 + 用户列表 | 🟢 P2 |
| RSS/Sitemap | 自动生成 | 🟢 P2 |

### Phase 3: 高级特性（4-6 周）

| 模块 | 功能 | 优先级 |
|------|------|--------|
| AI 辅助写作 | LLM API → 续写/润色/摘要 | 🟢 P2 |
| Git 同步 | 双向同步 GitHub 仓库 | 🟢 P2 |
| 主题市场 | 主题上传/分享/在线切换 | 🔵 P3 |
| 第三方导入 | Hexo/Jekyll/WordPress 导入 | 🔵 P3 |
| 评论系统 | 内置评论 + 垃圾过滤 | 🔵 P3 |
| 插件 API | 插件钩子机制 | 🔵 P3 |

### 里程碑时间线

```
Week 1    ═══════ Phase 0 基础搭建
Week 2-7  ═══════════════════════ Phase 1 MVP
Week 8-11 ═══════════════ Phase 2 体验完善
Week 12-17════════════════════════ Phase 3 高级特性
                          ▲
                     MVP 可用 (Week 7)
```

---

## 12. 风险与应对

### 12.1 技术风险

| 风险 | 可能性 | 影响 | 应对 |
|------|--------|------|------|
| Rust 开发速度慢 | 高 | 工期延后 | 严格控制 MVP 范围；优先用成熟 crate |
| comrak 渲染差异 | 中 | 表现不符合预期 | 提前建立 GFM 合规测试集 |
| Tantivy 中文分词不理想 | 中 | 搜索体验差 | 备选方案：外挂 MeiliSearch |
| Vditor 与 Vue3 集成问题 | 中 | 编辑器体验打折 | 提前做 PoC 验证；备选 Milkdown |
| 主题系统复杂度膨胀 | 高 | 工期不可控 | V1 只做配置式主题，不做模板上传 |

### 12.2 产品风险

| 风险 | 可能性 | 影响 | 应对 |
|------|--------|------|------|
| 与现有方案差异不够大 | 中 | 用户吸引力不足 | 聚焦"文档导入 + 轻量管理"差异化 |
| Rust 门槛劝退贡献者 | 高 | 社区建设难 | 提供完善的 API 文档和插件 SDK（未来 JS 插件） |
| 用户需求偏离 | 中 | 做了没人用 | MVP 后快速收集反馈，迭代调整 |

### 12.3 运维风险

| 风险 | 可能性 | 影响 | 应对 |
|------|--------|------|------|
| SQLite 数据损坏 | 低 | 数据丢失 | WAL 模式 + 定时自动备份 + 备份校验 |
| 存储耗尽 | 中 | 服务不可用 | 配额限制 + 告警 + 清理策略 |
| 安全漏洞 | 低 | 数据泄露 | Cargo audit + 定期安全扫描 + 负责任的披露机制 |

---

## 附录

### A. 术语表

| 术语 | 含义 |
|------|------|
| GFM | GitHub Flavored Markdown |
| Frontmatter | MD 文件头部的 YAML/TOML 元数据区 |
| SSG | Static Site Generator，静态站点生成器 |
| SSR | Server-Side Rendering，服务端渲染 |
| RBAC | Role-Based Access Control，基于角色的访问控制 |
| SPA | Single Page Application，单页应用 |
| TDD | Test-Driven Development，测试驱动开发 |

### B. 参考项目

| 项目 | URL | 参考点 |
|------|-----|--------|
| markdown-blog | github.com/gaowei-space/markdown-blog | Go 轻量实现参考 |
| Halo | github.com/halo-dev/halo | 功能完整性参考 |
| TinaCMS | github.com/tinacms/tinacms | Git-backed CMS 参考 |
| Vrite | github.com/vriteio/vrite | AI 集成参考 |
| Zola | github.com/getzola/zola | Rust SSG 参考 |

### C. 变更日志

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-05-10 | v1.0 | 初版需求文档 |
