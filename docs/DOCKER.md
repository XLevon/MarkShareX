# Docker 部署指南

## 目录

- [快速开始](#快速开始)
- [构建流程](#构建流程)
- [基础镜像加速](#基础镜像加速)
- [目录结构](#目录结构)
- [配置说明](#配置说明)

## 快速开始

```bash
# 1. 准备配置
cp config.example.toml config.toml    # 编辑端口/域名等
cp .env.example .env                  # 修改 JWT 密钥

# 2. 构建并启动
docker compose up -d

# 3. 访问
# http://localhost:5023        # 博客前台
# http://localhost:5023/admin  # 管理后台
```

首次启动时程序会自动初始化数据库、提取内置主题和模板到 `/data/` 目录。

## 构建流程

### 镜像分层

```
┌─────────────────────────────────────────────┐
│  阶段 1: frontend-builder (node:20-alpine)  │  ← 构建 Vue 前端
│  npm ci → vite build → /app/dist/           │
└──────────────────────┬──────────────────────┘
                       │ COPY /app/dist
┌──────────────────────▼──────────────────────┐
│  阶段 2: backend-builder (rust:1.95-slim)   │  ← 编译 Rust 后端
│  cargo build --release → marksharex 二进制   │     前端产物嵌入 static/frontend/
└──────────────────────┬──────────────────────┘
                       │ COPY 二进制 + dist
┌──────────────────────▼──────────────────────┐
│  阶段 3: runtime (debian:bookworm-slim)     │  ← 最小运行镜像
│  libssl3 + ca-certificates + 非特权用户      │     ≈ 80MB
└─────────────────────────────────────────────┘
```

### 构建命令

```bash
# 标准构建（从 Docker Hub 拉取基础镜像）
./scripts/build.sh

# 或手动
docker build -t marksharex:latest .
```

## 基础镜像加速

每次构建都会拉取 `node:20-alpine`、`rust:1.95-slim`、`debian:bookworm-slim` 的基础镜像 metadata，国内网络较慢。可以预先构建本地基础镜像，后续重建秒级跳过。

### 一次性准备

```bash
# 构建本地基础镜像（只需执行一次，后续无需拉取 registry）
./scripts/build.sh --base-only
```

这会生成三个本地镜像：

| 镜像 | 内容 |
|---|---|
| `marksharex-frontend-base` | node:20-alpine |
| `marksharex-backend-base` | rust:1.95-slim + build-essential + libssl-dev + pkg-config |
| `marksharex-runtime-base` | debian:bookworm-slim + libssl3 + ca-certificates |

### 快速重建

```bash
# 后续开发迭代时快速构建（跳过 registry 拉取）
./scripts/build.sh --local
```

### 原理

主 Dockerfile 使用 `ARG` 声明基础镜像，默认指向 Docker Hub，可通过 `--build-arg` 切换到本地镜像：

```dockerfile
ARG FRONTEND_BASE=node:20-alpine
ARG BACKEND_BASE=rust:1.95-slim
ARG RUNTIME_BASE=debian:bookworm-slim

FROM ${FRONTEND_BASE} AS frontend-builder
FROM ${BACKEND_BASE} AS backend-builder
FROM ${RUNTIME_BASE}
```

```bash
# 默认（拉取 registry）
docker build -t marksharex .

# 使用本地基础镜像（跳过拉取）
docker build \
  --build-arg FRONTEND_BASE=marksharex-frontend-base \
  --build-arg BACKEND_BASE=marksharex-backend-base \
  --build-arg RUNTIME_BASE=marksharex-runtime-base \
  -t marksharex .
```

### 配合镜像加速器

在国内服务器上进一步加速，配置 Docker 镜像加速器：

```json
// ~/.docker/daemon.json
{
  "registry-mirrors": [
    "https://docker.1ms.run",
    "https://docker.xuanyuan.me"
  ]
}
```

重启 Docker：`sudo systemctl restart docker`

## 目录结构

```
项目根目录/
├── Dockerfile              # 主构建文件
├── docker-compose.yml      # 容器编排
├── docker/
│   └── Dockerfile.base     # 本地基础镜像构建文件
├── scripts/
│   ├── build.sh            # 镜像构建脚本
│   ├── deploy.sh           # 部署脚本（构建 + 启动）
│   └── migrate_storage.sh  # 数据迁移脚本
├── .env.example            # 环境变量模板
└── config.example.toml     # 配置文件模板
```

## 配置说明

Docker 容器通过环境变量覆盖配置，无需修改 `config.toml`：

| 环境变量 | 默认值 | 说明 |
|---|---|---|
| `MARKSHAREX_SERVER_PORT` | `5023` | 监听端口 |
| `MARKSHAREX_SERVER_HOST` | `0.0.0.0` | 监听地址 |
| `MARKSHAREX_DATA_DIR` | `/data` | 数据目录（挂载 volume） |
| `MARKSHAREX_STORAGE_UPLOAD_DIR` | `/data/uploads` | 上传目录 |
| `MARKSHAREX_DATABASE_URL` | `sqlite:///data/marksharex.db?mode=rwc` | 数据库路径 |

容器内目录映射：

```
容器内                         来源
/app/marksharex               ← 编译产物
/app/config.toml              ← 镜像内置（可挂载覆盖）
/app/static/frontend/         ← 前端构建产物
/data/templates/              ← 首次启动自动提取
/data/themes/                 ← 首次启动自动提取
/data/marksharex.db           ← 自动创建
/data/uploads/                ← 自动创建
```
