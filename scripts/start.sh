#!/bin/bash
set -e

# ==============================================================================
# MarkShareX 启动脚本（构建 + 启动，一体）
#
# 使用方式:
#   ./scripts/start.sh                 # 智能构建 + 启动
#   ./scripts/start.sh --base-only     # 仅预缓存 base 镜像（不构建应用）
#   ./scripts/start.sh --rebuild       # 强制重建（跳过缓存检测）
#   ./scripts/start.sh prod            # 生产环境
#   ./scripts/start.sh prod --rebuild  # 组合
#
# 智能构建三级降级:
#   1. marksharex:latest 已存在 → 跳过构建，直接启动
#   2. marksharex-backend-base 存在 → 用本地缓存构建
#   3. 都没有 → 从 Docker Hub 完整构建
# ==============================================================================

PROJECT_DIR="${MARKSHAREX_PROJECT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
cd "$PROJECT_DIR"

IMAGE_NAME="${IMAGE_NAME:-marksharex}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
IMAGE="${IMAGE_NAME}:${IMAGE_TAG}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info()    { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $1"; }
die()     { echo -e "${RED}[FATAL]${NC} $1"; exit 1; }

# ═══════════════════════════════════════════════════
# 参数解析
# ═══════════════════════════════════════════════════

ENV="dev"
FORCE_REBUILD=false
BASE_ONLY=false

for arg in "$@"; do
    case "$arg" in
        dev|prod)  ENV="$arg" ;;
        --rebuild) FORCE_REBUILD=true ;;
        --base-only) BASE_ONLY=true ;;
        *)         die "无效参数: $arg（可用: dev | prod | --rebuild | --base-only）" ;;
    esac
done

# ═══════════════════════════════════════════════════
# 配置初始化
# ═══════════════════════════════════════════════════

ensure_env_secret() {
    local key="$1"
    local generated=""
    local line
    local found=false
    local tmp

    [ -f .env ] || die ".env 不存在，且未找到 .env.example"
    tmp=$(mktemp "${TMPDIR:-/tmp}/marksharex-env.XXXXXX")

    while IFS= read -r line || [ -n "$line" ]; do
        if [[ "$line" == "${key}="* ]]; then
            local value="${line#*=}"
            if [ -z "${value//[[:space:]]/}" ]; then
                command -v openssl &>/dev/null || die "${key} 为空，且未安装 openssl，无法生成安全随机值"
                [ -n "$generated" ] || generated=$(openssl rand -hex 32)
                printf '%s=%s\n' "$key" "$generated" >> "$tmp"
            else
                printf '%s\n' "$line" >> "$tmp"
            fi
            found=true
        else
            printf '%s\n' "$line" >> "$tmp"
        fi
    done < .env

    if ! $found; then
        command -v openssl &>/dev/null || die "未配置 ${key}，且未安装 openssl，无法生成安全随机值"
        generated=$(openssl rand -hex 32)
        printf '%s=%s\n' "$key" "$generated" >> "$tmp"
    fi

    mv "$tmp" .env
    [ -z "$generated" ] || info "已生成并持久化 ${key}"
}

if ! $BASE_ONLY; then
    if [ ! -f .env ] && [ -f .env.example ]; then
        info "首次运行，从 .env.example 创建 .env"
        cp .env.example .env
        success ".env 已创建"
    fi

    ensure_env_secret "MARKSHAREX_AUTH_JWT_SECRET"
    ensure_env_secret "MARKSHAREX_AUTH_ENCRYPT_KEY"
    chmod 600 .env
fi

# 仅供配置契约测试和自动化初始化使用；不会访问 Docker daemon。
if [ "${MARKSHAREX_INIT_ENV_ONLY:-0}" = "1" ]; then
    exit 0
fi

# ═══════════════════════════════════════════════════
# 预检
# ═══════════════════════════════════════════════════

command -v docker &>/dev/null       || die "Docker 未安装"
docker compose version &>/dev/null || die "Docker Compose 未安装"

if [ "$ENV" = "prod" ]; then
    info "生产环境模式"
fi

# ═══════════════════════════════════════════════════
# build_base_images — 预缓存 3 个基础镜像到本地
# ═══════════════════════════════════════════════════

build_base_images() {
    info "构建本地 base 缓存..."
    for target in frontend-base backend-base runtime-base; do
        local tag="marksharex-${target}"
        info "  → ${tag}"
        docker build \
            -f docker/Dockerfile.base \
            --target "$target" \
            -t "$tag" \
            . || die "base 镜像构建失败: $target"
    done
    success "base 缓存构建完成"
}

# ═══════════════════════════════════════════════════
# build_app — 构建应用镜像
# ═══════════════════════════════════════════════════

build_app() {
    local use_local="${1:-false}"

    if [ "$use_local" = "true" ]; then
        info "使用本地 base 缓存构建 $IMAGE ..."
        docker build \
            --build-arg FRONTEND_BASE=marksharex-frontend-base \
            --build-arg BACKEND_BASE=marksharex-backend-base \
            --build-arg RUNTIME_BASE=marksharex-runtime-base \
            -t "$IMAGE" \
            . || die "镜像构建失败"
    else
        info "从 Docker Hub 构建 $IMAGE ..."
        docker build -t "$IMAGE" . || die "镜像构建失败"
    fi
    success "$IMAGE 构建完成"
}

# ═══════════════════════════════════════════════════
# 执行
# ═══════════════════════════════════════════════════

if $BASE_ONLY; then
    build_base_images
    echo ""
    info "下一步: ./scripts/start.sh"
    exit 0
fi

# ── 构建判定 ──

IMAGE_EXISTS=false
BASE_EXISTS=false

docker image inspect "$IMAGE" &>/dev/null            && IMAGE_EXISTS=true
docker image inspect marksharex-backend-base &>/dev/null && BASE_EXISTS=true

if $FORCE_REBUILD; then
    build_base_images
    build_app true
elif $IMAGE_EXISTS; then
    success "$IMAGE 已存在，跳过构建"
elif $BASE_EXISTS; then
    warn "未找到 $IMAGE，使用本地 base 缓存"
    build_app true
else
    warn "首次构建：先缓存 base 镜像..."
    build_base_images
    build_app true
fi

# ── 启动 ──

info "启动容器..."
docker compose up -d --no-build

# ── 获取容器实际对外端口 ──
HOST_PORT=$(docker compose port marksharex 5023 2>/dev/null | grep -o ':[0-9]\+' | tr -d ':')
PORT="${HOST_PORT:-5023}"

# ── 等待就绪 ──

info "等待服务就绪..."
for i in $(seq 1 30); do
    if curl -sf "http://localhost:$PORT/api/v1/health" &>/dev/null; then
        echo ""
        success "MarkShareX 已就绪！"
        echo ""
        info "访问地址:"
        echo "  前台:  http://localhost:$PORT"
        echo "  后台:  http://localhost:$PORT/admin"
        [ "$ENV" = "prod" ] && warn "请确保已配置域名解析和 SSL 证书"
        echo ""
        exit 0
    fi
    sleep 1
done

die "服务未在 30 秒内就绪，请检查: docker compose logs"
