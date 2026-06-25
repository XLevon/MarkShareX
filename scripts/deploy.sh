#!/bin/bash
set -e

# ==============================================================================
# MarkShareX 部署脚本
# 使用方式:
#   开发环境: ./scripts/deploy.sh dev
#   生产环境: ./scripts/deploy.sh prod
# ==============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PROJECT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$PROJECT_DIR"

ENV="${1:-dev}"
PORT="${MARKSHAREX_SERVER_PORT:-5023}"

info()    { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
warning() { echo -e "${YELLOW}[WARN]${NC} $1"; }
die()     { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

check_dependencies() {
    info "检查系统依赖..."
    command -v docker &>/dev/null || die "Docker 未安装"
    docker compose version &>/dev/null || die "Docker Compose 未安装"
    success "依赖检查通过"
}

ensure_env() {
    if [ ! -f .env ]; then
        if [ -f .env.example ]; then
            info "从 .env.example 创建 .env"
            cp .env.example .env
        else
            die ".env.example 不存在，无法创建配置"
        fi
    fi
    # 如果 JWT 密钥还是示例值，生成随机密钥
    if grep -q 'change.*cret' .env 2>/dev/null; then
        warning "检测到示例 JWT 密钥，生成随机密钥..."
        local new_secret=$(openssl rand -hex 32 2>/dev/null || python3 -c "import secrets; print(secrets.token_hex(32))")
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/^MARKSHAREX_AUTH_JWT_SECRET=.*/MARKSHAREX_AUTH_JWT_SECRET=${new_secret}/" .env
        else
            sed -i "s/^MARKSHAREX_AUTH_JWT_SECRET=.*/MARKSHAREX_AUTH_JWT_SECRET=${new_secret}/" .env
        fi
        success "JWT 密钥已更新"
    fi
}

deploy() {
    info "构建并启动容器..."
    docker compose up --build -d

    info "等待服务就绪..."
    for i in $(seq 1 30); do
        if curl -s "http://localhost:$PORT/api/v1/health" | grep -q "OK"; then
            break
        fi
        sleep 1
    done

    if curl -s "http://localhost:$PORT/api/v1/health" | grep -q "OK"; then
        success "部署成功！"
        echo ""
        info "访问地址:"
        echo "  http://localhost:$PORT"
        echo "  http://localhost:$PORT/admin"
    else
        die "服务未就绪，请检查: docker compose logs"
    fi
}

prod_check() {
    if [ -z "$MARKSHAREX_SERVER_BASE_URL" ]; then
        die "请设置 MARKSHAREX_SERVER_BASE_URL（如: export MARKSHAREX_SERVER_BASE_URL=https://your-domain.com）"
    fi
}

main() {
    check_dependencies
    ensure_env

    case "$ENV" in
        dev)
            info "部署开发环境..."
            deploy
            ;;
        prod)
            info "部署生产环境..."
            prod_check
            deploy
            warning "请确保已配置域名解析和 SSL 证书"
            ;;
        *)
            die "无效参数: $ENV. 请使用 'dev' 或 'prod'"
            ;;
    esac
}

main "$@"
