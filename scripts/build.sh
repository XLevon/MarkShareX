#!/bin/bash
set -e

# ==============================================================================
# MarkShareX Docker 镜像构建脚本
#
# 使用方式:
#   ./scripts/build.sh              # 构建应用镜像（从 Docker Hub 拉取基础镜像）
#   ./scripts/build.sh --local      # 先构建本地基础镜像，再构建应用镜像（快速重建）
#   ./scripts/build.sh --base-only  # 仅构建本地基础镜像
# ==============================================================================

PROJECT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$PROJECT_DIR"

IMAGE_NAME="${IMAGE_NAME:-marksharex}"
IMAGE_TAG="${IMAGE_TAG:-latest}"

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
die() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# 构建基础镜像（只需执行一次，之后 rebuild 跳过 registry 拉取）
build_base_images() {
    info "构建本地基础镜像..."

    for target in frontend-base backend-base runtime-base; do
        local tag="marksharex-${target}"
        info "  → ${tag}"
        docker build \
            -f docker/Dockerfile.base \
            --target "$target" \
            -t "$tag" \
            . || die "基础镜像构建失败: $target"
    done

    success "基础镜像构建完成"
}

# 构建应用镜像
build_app() {
    local use_local="${1:-false}"

    info "构建应用镜像: ${IMAGE_NAME}:${IMAGE_TAG}"

    if [ "$use_local" = "true" ]; then
        info "使用本地基础镜像（跳过 registry 拉取）"
        docker build \
            --build-arg FRONTEND_BASE=marksharex-frontend-base \
            --build-arg BACKEND_BASE=marksharex-backend-base \
            --build-arg RUNTIME_BASE=marksharex-runtime-base \
            -t "${IMAGE_NAME}:${IMAGE_TAG}" \
            . || die "应用镜像构建失败"
    else
        docker build \
            -t "${IMAGE_NAME}:${IMAGE_TAG}" \
            . || die "应用镜像构建失败"
    fi

    success "镜像构建完成: ${IMAGE_NAME}:${IMAGE_TAG}"
}

case "${1:-}" in
    --base-only)
        build_base_images
        ;;
    --local)
        build_base_images
        build_app true
        ;;
    *)
        build_app
        ;;
esac

echo ""
echo "运行方式:"
echo "  docker compose up -d"
echo ""
echo "手动运行:"
echo "  docker run -d --name marksharex -p 5023:5023 -v marksharex_data:/data ${IMAGE_NAME}:${IMAGE_TAG}"
