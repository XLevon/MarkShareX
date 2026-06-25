#!/bin/bash
set -e

# ==============================================================================
# MarkShareX 存储迁移脚本
# 将 files 表中的旧路径格式迁移为仅存储 filename
# 使用方式: ./scripts/migrate_storage.sh
# ==============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PROJECT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$PROJECT_DIR"

info()    { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
warning() { echo -e "${YELLOW}[WARN]${NC} $1"; }
die()     { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# 从 .env 或 config.toml 获取数据库路径
get_db_path() {
    if [ -f .env ]; then
        local raw=$(grep -E '^MARKSHAREX_DATABASE_URL=' .env | cut -d '=' -f 2-)
        # 处理 sqlite://./relative/path?mode=rwc 格式
        if echo "$raw" | grep -q 'sqlite://\./'; then
            DB_PATH=$(echo "$raw" | sed -E 's|sqlite://\./([^?]*).*|\1|')
            DB_PATH="$PROJECT_DIR/$DB_PATH"
        # 处理 sqlite:///absolute/path?mode=rwc 格式
        elif echo "$raw" | grep -q 'sqlite:///'; then
            DB_PATH=$(echo "$raw" | sed -E 's|sqlite://(/[^?]*).*|\1|')
        else
            DB_PATH="$raw"
        fi
    fi

    if [ -z "$DB_PATH" ]; then
        DB_PATH="$PROJECT_DIR/data/marksharex.db"
    fi

    if ! echo "$DB_PATH" | grep -q '^/'; then
        DB_PATH="$PROJECT_DIR/$DB_PATH"
    fi

    [ -f "$DB_PATH" ] || die "数据库不存在: $DB_PATH"
    echo "$DB_PATH"
}

# 从 .env 或 config.toml 获取上传目录
get_upload_dir() {
    if [ -f .env ]; then
        local dir=$(grep -E '^MARKSHAREX_STORAGE_UPLOAD_DIR=' .env | cut -d '=' -f 2-)
        if [ -n "$dir" ]; then
            # 相对路径转绝对
            if ! echo "$dir" | grep -q '^/'; then
                dir="$PROJECT_DIR/$dir"
            fi
            echo "$dir"
            return
        fi
    fi
    echo "$PROJECT_DIR/data/uploads"
}

check_dependencies() {
    info "检查依赖..."
    command -v sqlite3 &>/dev/null || die "sqlite3 未安装"
    success "依赖检查通过"
}

backup() {
    local db="$1"
    local backup="${db}.backup.$(date +%Y%m%d_%H%M%S)"
    info "备份数据库: $backup"
    cp "$db" "$backup"
    success "备份完成"
}

migrate() {
    local db="$1"
    local upload_dir="$2"

    info "上传目录: $upload_dir"
    info "开始迁移..."

    local count=0
    while IFS='|' read -r id filename storage_path url; do
        [ -z "$id" ] && continue
        ((count++))

        local new_name=$(basename "$filename")
        if [ "$new_name" != "$filename" ]; then
            info "  #$id: $filename → $new_name"
            sqlite3 "$db" "UPDATE files SET filename='$new_name' WHERE id=$id;"
            filename="$new_name"
        fi

        # 移动旧路径的文件到 upload_dir
        if [ -n "$storage_path" ] && [ -f "$storage_path" ] && [ ! -f "$upload_dir/$filename" ]; then
            info "  移动: $storage_path → $upload_dir/$filename"
            mkdir -p "$upload_dir"
            cp "$storage_path" "$upload_dir/$filename"
        fi

    done < <(sqlite3 "$db" "SELECT id, filename, storage_path, url FROM files WHERE deleted_at IS NULL;")

    success "处理完成 ($count 条记录)"
}

verify() {
    local db="$1"
    info "验证..."

    local bad_name=$(sqlite3 "$db" "SELECT COUNT(*) FROM files WHERE filename LIKE '%/%' AND deleted_at IS NULL;")
    local has_storage=$(sqlite3 "$db" "SELECT COUNT(*) FROM files WHERE storage_path IS NOT NULL AND deleted_at IS NULL;")

    [ "$bad_name" -ne 0 ] && warning "$bad_name 条记录的 filename 仍含路径"
    [ "$has_storage" -ne 0 ] && warning "$has_storage 条记录的 storage_path 不为 NULL"
    [ "$bad_name" -eq 0 ] && [ "$has_storage" -eq 0 ] && success "数据已是最新格式"
}

main() {
    info "=== MarkShareX 存储迁移 ==="
    check_dependencies

    local db=$(get_db_path)
    local upload_dir=$(get_upload_dir)

    info "数据库: $db"
    info "上传目录: $upload_dir"

    backup "$db"
    migrate "$db" "$upload_dir"
    verify "$db"

    echo ""
    success "迁移完成！重启 MarkShareX 后生效"
}

main "$@"
