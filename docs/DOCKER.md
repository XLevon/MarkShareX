# Docker 部署指南

## 目录

- [快速开始](#快速开始)
- [镜像分层](#镜像分层)
- [国内加速部署](#国内加速部署)
- [文件清单](#文件清单)
- [配置说明](#配置说明)

## 快速开始

### 国外 / 网络通畅

```bash
git clone https://github.com/XLevon/MarkShareX.git
cd MarkShareX
docker compose up -d
```

### 国内 / 网络受限

```bash
git clone https://github.com/XLevon/MarkShareX.git
cd MarkShareX

# 一句话搞定（首次自动缓存 base，之后每次秒起）
./scripts/start.sh
```

### 运维

```bash
docker compose ps                      # 查看状态
docker compose logs -f                 # 查看日志
docker compose restart                 # 重启
docker compose down                    # 停止（数据保留在 volume）
docker compose down -v                 # 停止并清空数据 ⚠️
curl localhost:5023/api/v1/health      # 健康检查
```

---

## 镜像分层

```
┌────────────────────────────────────────────┐
│ 阶段 1: frontend-builder (node:20-alpine)  │
│ npm ci → vite build → /app/dist/           │
└──────────────────┬─────────────────────────┘
                   │ COPY /app/dist
┌──────────────────▼─────────────────────────┐
│ 阶段 2: backend-builder (rust:1.95-slim)   │
│ include_str! 嵌入模板 → cargo build --rel   │
│ COPY 前端 dist → static/frontend/           │
└──────────────────┬─────────────────────────┘
                   │ COPY 二进制 + static/
┌──────────────────▼─────────────────────────┐
│ 阶段 3: runtime (ubuntu:24.04)             │
│ libssl3 + ca-certificates + curl            │
│ 非特权用户 marksharex                       │
│ 最终镜像 ≈ 80MB                             │
└────────────────────────────────────────────┘
```

> **模板**：`include_str!("../templates/...")` 编译期嵌入二进制，首次启动提取到 `/data/templates/`，运行时 Tera 从文件系统加载（方便修改）。
>
> **glibc**：ARM64 版 `rust:1.95-slim` 链接 glibc 2.39，运行时选用 `ubuntu:24.04`（glibc 2.39）而非 `debian:bookworm`（glibc 2.36），避免 `GLIBC_2.39 not found`。

---

## 国内加速部署

### 原理

主 `Dockerfile` 用 `ARG` 声明基础镜像，默认指向 Docker Hub，可通过 `--build-arg` 切换到本地：

```dockerfile
ARG FRONTEND_BASE=node:20-alpine
ARG BACKEND_BASE=rust:1.95-slim
ARG RUNTIME_BASE=ubuntu:24.04

FROM ${FRONTEND_BASE} AS frontend-builder
FROM ${BACKEND_BASE} AS backend-builder
FROM ${RUNTIME_BASE}
```

`docker/Dockerfile.base` 把三个基础镜像的拉取 + apt-get 依赖安装**一次性完成**并缓存到本地。`start.sh` 首次运行时自动执行这一步：

```
全新克隆
  └─ ./scripts/start.sh
       ├─ 检测：无镜像、无 base 缓存
       ├─ build_base_images   ← 一次性（拉 Docker Hub + apt-get）
       │   → marksharex-frontend-base
       │   → marksharex-backend-base
       │   → marksharex-runtime-base
       └─ build_app true      ← 用本地 base 构建应用镜像

之后每次
  └─ ./scripts/start.sh
       ├─ 检测：base 缓存存在
       └─ build_app true      ← 秒级（跳过 Docker Hub + apt-get）
```

**关键优化**：主 Dockerfile 中 backend 和 runtime 的 `apt-get` 已被注释，因为这些依赖已在 base 镜像预装。之后每次构建只需 `npm ci + vite build` 和 `cargo build --release`，零冗余。

### 其他选项

```bash
./scripts/start.sh --base-only      # 仅缓存 base（不构建应用、不启动）
./scripts/start.sh --rebuild        # 强制重建（先刷新 base 缓存）
./scripts/start.sh prod             # 生产环境（校验 BASE_URL）
```

也可以跳过脚本，直接用专用 compose 文件：

```bash
./scripts/start.sh --base-only      # 先缓存 base
docker compose -f docker-compose.local.yml up -d   # 构建 + 启动
```

---

## 文件清单

```
项目根目录/
├── Dockerfile                  # 主构建文件（多阶段）
├── docker-compose.yml          # 编排（默认：Docker Hub）
├── docker-compose.local.yml    # 编排（本地 base 缓存）
├── docker/
│   └── Dockerfile.base         # 基础镜像预构建
├── scripts/
│   ├── start.sh                # 启动脚本（智能构建 + 启动）
│   └── migrate_storage.sh      # 数据迁移
├── .env.example                # 环境变量模板（可选）
└── config.toml                 # 默认配置（镜像内置）
```

### 三个 compose 文件的关系

| 文件 | 基础镜像来源 | 适用场景 |
|------|-------------|---------|
| `docker-compose.yml` | Docker Hub | 国外 / 首次 |
| `docker-compose.local.yml` | 本地 `marksharex-*-base` | 国内 / 已有缓存 |
| （`start.sh` 内部调用） | 自动检测选择 | 推荐 |

`docker-compose.local.yml` 的唯一区别是多了一行 `args:`：

```yaml
build:
  args:
    FRONTEND_BASE: marksharex-frontend-base
    BACKEND_BASE: marksharex-backend-base
    RUNTIME_BASE: marksharex-runtime-base
```

---

## 配置说明

### 优先级

```
环境变量（docker-compose.yml / .env）  ← 最高
    ↓ 覆盖
config.toml（镜像内置）                ← 兜底
```

### docker-compose.yml 已声明的变量

| 环境变量 | 容器内值 | 说明 |
|----------|---------|------|
| `MARKSHAREX_DATA_DIR` | `/data` | 覆盖 config.toml 的相对路径 |
| `MARKSHAREX_STORAGE_UPLOAD_DIR` | `/data/uploads` | 同上 |
| `MARKSHAREX_DATABASE_URL` | `sqlite:///data/marksharex.db?mode=rwc` | 同上 |
| `MARKSHAREX_SERVER_HOST` | `0.0.0.0` | 容器内必须绑 0.0.0.0 |
| `MARKSHAREX_SERVER_PORT` | `5023` | 端口变更时覆盖 |

其余（数据库连接数、AI 参数、JWT 过期等）走 `config.toml`。密钥通过 `.env` 注入：

```bash
cp .env.example .env
vi .env   # 修改 MARKSHAREX_ENCRYPT_KEY
```

### 容器内文件布局

```
/app/marksharex               ← cargo build --release
/app/config.toml              ← 镜像内置
/app/static/frontend/         ← vite build
/data/templates/              ← 首次启动从二进制提取
/data/marksharex.db           ← 自动创建
/data/uploads/                ← 自动创建
/data/search_index/           ← 自动创建
```
