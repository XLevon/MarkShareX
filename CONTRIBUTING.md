# 贡献指南

感谢你关注 MarkShareX。提交代码前，请先阅读本指南和 [安全政策](SECURITY.md)。

## 先沟通再实现

- Bug：先搜索现有 Issue；不存在时使用 Bug 模板提交可复现信息。
- 新功能或较大重构：建议先创建 Feature Request，确认范围和设计方向。
- 安全漏洞：不要公开提交，请按 [SECURITY.md](SECURITY.md) 私下报告。

## 开发环境

建议使用：

- Rust `1.95`
- Node.js `22.14+`
- npm（使用仓库中的 `frontend/package-lock.json`）
- Python `3.11+`（仅文档同步检查脚本使用标准库）
- SQLite

### 1. 获取源码

```bash
git clone https://github.com/XLevon/MarkShareX.git
cd MarkShareX
```

### 2. 初始化配置

```bash
cp config.example.toml config.toml
cp .env.example .env
openssl rand -hex 32  # 填入 MARKSHAREX_AUTH_JWT_SECRET
openssl rand -hex 32  # 填入 MARKSHAREX_AUTH_ENCRYPT_KEY
```

`.env`、`config.toml`、数据库、上传目录、日志和真实密钥不得提交。配置字段和环境变量说明见 [docs/CONFIG.md](docs/CONFIG.md)。

### 3. 安装前端依赖

```bash
cd frontend
npm ci
cd ..
```

### 4. 启动开发服务

终端一：

```bash
cargo run
```

终端二：

```bash
cd frontend
npm run dev
```

默认地址：

- 后端：`http://127.0.0.1:5023`
- Vite 前端：`http://127.0.0.1:5173`

首次访问时按照初始化页面创建管理员。不要把测试账号或真实部署数据提交到仓库。

## 质量门禁

提交 PR 前，请根据改动范围运行以下检查。

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

`npm run build` 已包含 `vue-tsc --noEmit` 类型检查。

### 文档一致性

```bash
python3 -m unittest discover -s scripts/tests -p 'test_*.py' -v
python3 scripts/check_doc_sync.py
```

### 容器、Compose 或启动脚本

修改部署文件时还应运行：

```bash
bash -n scripts/start.sh scripts/migrate_storage.sh
docker compose config --quiet
docker compose -f docker-compose.local.yml config --quiet
docker build --check .
docker build --tag marksharex:local-check .
git diff --check
```

`.github/workflows/ci.yml` 是 CI 命令的最终事实来源；新增或调整质量门禁时应同步更新本节。

如果修改了版本号、API、配置绑定、数据库 Schema 或 CI，请同步更新相应文档和契约测试。

## 提交规范

建议使用 Conventional Commits：

```text
feat(scope): add capability
fix(scope): correct behavior
docs: update documentation
test(scope): add regression coverage
refactor(scope): simplify implementation
chore: maintain tooling
```

要求：

- 一个提交聚焦一个逻辑主题；
- 不混入格式化噪声、生成缓存、运行数据或无关重构；
- Bug 修复应先增加能复现问题的测试，再实现修复；
- 提交信息说明“为什么”，而不仅是“改了什么”；
- 不提交凭据、生产配置、数据库、上传文件、日志或备份。

## Pull Request 规范

PR 应包含：

- 背景、问题和目标；
- 实现方法及关键权衡；
- 测试命令与真实结果；
- 数据库、配置、API、权限和兼容性影响；
- UI 改动的桌面端和移动端截图；
- 关联的 Issue（如 `Closes #123`）。

请保持 PR 尺寸可审查。审查过程中新增改动后，应重新运行受影响的质量门禁。所有 CI 必须通过后才能合并。

## 文档与许可证

新增面向用户的功能、配置项或破坏性变更时，请同步更新 README、配置文档和 CHANGELOG。提交代码即表示你同意该贡献按仓库的 [MIT License](LICENSE) 发布。
