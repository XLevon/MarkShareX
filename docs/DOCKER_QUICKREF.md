# Docker 构建命令清单

> 最后更新：2026-07-14

---

## 一、国外（网络通畅）

```bash
git clone https://github.com/XLevon/MarkShareX.git
cd MarkShareX
docker compose up -d
```

---

## 二、国内（推荐）

```bash
git clone https://github.com/XLevon/MarkShareX.git
cd MarkShareX

# 一句话搞定（首次自动缓存 base，之后秒起）
./scripts/start.sh
```

之后每次都这一条。首次会自动从 Docker Hub 拉取并缓存 base 镜像（一次性代价），后续 rebuild 跳过所有拉取。

---

## 三、国内 · compose 文件方案

```bash
git clone https://github.com/XLevon/MarkShareX.git
cd MarkShareX

# ① 一次性：缓存 base 镜像
./scripts/start.sh --base-only

# ② 构建 + 启动
docker compose -f docker-compose.local.yml up -d
```

---

## 四、国内 · 纯 Docker 方案（不依赖脚本）

```bash
# ① 一次性：缓存 base 镜像
docker build -f docker/Dockerfile.base --target frontend-base -t marksharex-frontend-base .
docker build -f docker/Dockerfile.base --target backend-base  -t marksharex-backend-base  .
docker build -f docker/Dockerfile.base --target runtime-base  -t marksharex-runtime-base  .

# ② 构建应用镜像
docker build \
    --build-arg FRONTEND_BASE=marksharex-frontend-base \
    --build-arg BACKEND_BASE=marksharex-backend-base \
    --build-arg RUNTIME_BASE=marksharex-runtime-base \
    -t marksharex:latest \
    .

# ③ 启动
docker compose up -d --no-build
```

之后每次改动重复 ②③。

---

## 五、运维

```bash
docker compose ps                   # 查看状态
docker compose logs -f              # 查看日志
docker compose restart              # 重启
docker compose down                 # 停止（保留数据）
docker compose down -v              # 停止+删除数据 ⚠️
curl localhost:5023/api/v1/health   # 健康检查
```

---

## 对照

| | 脚本 | compose 文件 | 纯 Docker |
|--|------|-------------|----------|
| 首次 | `./scripts/start.sh` | `start.sh --base-only` + `docker compose -f local.yml up -d` | 3 + 1 + 1 条 |
| 之后 | `./scripts/start.sh` | `docker compose -f docker-compose.local.yml up -d` | 2 条 |
| 依赖 | 项目自带脚本 | 项目自带 yml | 无 |
| 特点 | 全自动，首次自动建 base | 显式声明，简洁 | 全手动，不依赖任何文件 |

---

## 内置优化

| 优化 | 效果 |
|------|------|
| backend apt-get | 注释掉（base 已预装） |
| runtime apt-get | 注释掉（base 已预装） |
| 模板 | `include_str!` 编译期嵌入 |
| glibc | runtime 用 `ubuntu:24.04`（glibc 2.39），匹配 ARM64 |
| HEALTHCHECK | `curl` 内置在 runtime-base |
