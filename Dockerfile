# ==============================================================================
# MarkShareX Docker 构建文件
#
# 默认从 Docker Hub 拉取基础镜像，也可指定本地镜像加速：
#   docker build -t marksharex .
#
# 使用本地基础镜像（跳过拉取）：
#   docker build \
#     --build-arg FRONTEND_BASE=marksharex-frontend-base \
#     --build-arg BACKEND_BASE=marksharex-backend-base \
#     --build-arg RUNTIME_BASE=marksharex-runtime-base \
#     -t marksharex .
# ==============================================================================

ARG FRONTEND_BASE=node:20-alpine
ARG BACKEND_BASE=rust:1.95-slim
ARG RUNTIME_BASE=debian:bookworm-slim

# ------------------------------ 阶段 1: 构建前端 ------------------------------
FROM ${FRONTEND_BASE} AS frontend-builder

WORKDIR /app

# 复制前端代码
COPY frontend/package*.json ./
COPY frontend/vite.config.ts ./
COPY frontend/tsconfig.json ./
COPY frontend/tsconfig.node.json ./
COPY frontend/src ./src
COPY frontend/public ./public
COPY frontend/index.html ./

# 安装依赖并构建（覆盖 outDir，容器内不使用 ../static/frontend）
RUN npm ci && npm run build -- --outDir dist

# ------------------------------ 阶段 2: 构建后端 ------------------------------
FROM ${BACKEND_BASE} AS backend-builder

WORKDIR /app

# 安装构建依赖（使用本地基础镜像时已预装，但仍保留确保兼容）
RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# 复制 Rust 项目文件
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY build.rs ./

# 复制前端构建产物
COPY --from=frontend-builder /app/dist ./static/frontend

# 构建 Release 版本
RUN cargo build --release --locked

# ------------------------------ 阶段 3: 生产镜像 ------------------------------
FROM ${RUNTIME_BASE}

WORKDIR /app

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 创建非特权用户
RUN useradd -m marksharex

# 复制构建产物
COPY --from=backend-builder /app/target/release/marksharex ./marksharex
COPY --from=frontend-builder /app/dist ./static/frontend
COPY config.toml ./

# 设置权限
RUN chown -R marksharex:marksharex /app
RUN mkdir -p /data && chown -R marksharex:marksharex /data

# 切换到非特权用户
USER marksharex

# 暴露端口
EXPOSE 5023

# 启动命令
CMD ["./marksharex"]

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s \
    CMD curl -f http://localhost:5023/api/v1/health || exit 1
