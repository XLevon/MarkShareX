# MarkShareX 配置参考

MarkShareX 从项目工作目录中的 `config.toml` 加载基础配置，然后使用环境变量覆盖对应字段。启动时的有效优先级为：

> 环境变量 > `config.toml`

程序会先读取 `.env`（如果存在）；操作系统中已经存在的同名变量不会被 `.env` 覆盖。环境变量中的整数如果无法解析，程序会直接返回包含变量名和原始值的错误，不再静默忽略后退到 TOML 值。

所有配置结构都拒绝未知字段。拼写错误或已删除字段（例如旧 `server.base_url`）会阻止启动，避免配置看似生效、实际上被静默忽略。

## 快速开始

Docker 启动脚本会在首次运行时复制 `.env.example`，为两个为空或仅含空白的 secret 分别生成 64 位十六进制随机值，并以 `0600` 权限持久化；后续启动不会轮换已有值：

```bash
cp config.example.toml config.toml
./scripts/start.sh
```

原生启动时，先复制示例并将两个随机值分别填入 `.env`：

```bash
cp config.example.toml config.toml
cp .env.example .env
openssl rand -hex 32  # 填入 MARKSHAREX_AUTH_JWT_SECRET
openssl rand -hex 32  # 填入 MARKSHAREX_AUTH_ENCRYPT_KEY
cargo run
```

公开示例故意不携带可复用的认证密钥。`auth.jwt_secret` 和 `auth.encrypt_key` 为空时，程序会拒绝启动。

### 从旧配置升级

在替换并重启新二进制之前，先检查服务器现有 `config.toml`：删除旧 `server.base_url` 和已无运行时消费者的 `ai.search.api_url`，并确认 `auth.jwt_secret` 与 `auth.encrypt_key` 非空；也可以改用对应规范环境变量。新版本会拒绝未知字段和空 secret，因此这一步必须在服务重启前完成。该迁移只修改配置文件，不修改数据库。

## 基础 TOML 结构

`config.example.toml` 是完整、可反序列化的配置模板，字段分为：

- `data_dir`：数据库外的数据、模板和搜索索引根目录。
- `[server]`：监听地址、端口、可信代理和 CORS allowlist。
- `[database]`：SQLite URL 和连接池大小。
- `[auth]`：JWT、刷新令牌和 API Key 加密配置。
- `[storage]`：上传目录、单文件大小限制和允许的 MIME 类型。
- `[ai]` / `[ai.search]`：工具调用轮次、Provider 网络 allowlist 和搜索降级链。

## 环境变量契约

列表类型使用英文逗号分隔；每项会去除首尾空白，空项会被忽略。字符串按原值覆盖。整数必须是目标 Rust 类型可接受的十进制值。

### 全局与服务器

| 环境变量 | TOML 字段 | 类型/说明 |
|---|---|---|
| `MARKSHAREX_DATA_DIR` | `data_dir` | 字符串 |
| `MARKSHAREX_SERVER_HOST` | `server.host` | 字符串 |
| `MARKSHAREX_SERVER_PORT` | `server.port` | `u16`，必须大于 0 |
| `MARKSHAREX_SERVER_TRUSTED_PROXIES` | `server.trusted_proxies` | 逗号分隔的精确代理 IP |
| `MARKSHAREX_SERVER_CORS_ALLOWED_ORIGINS` | `server.cors_allowed_origins` | 逗号分隔的精确 HTTP(S) origin |

### 数据库

| 环境变量 | TOML 字段 | 类型/说明 |
|---|---|---|
| `MARKSHAREX_DATABASE_URL` | `database.url` | SQLite URL |
| `MARKSHAREX_DATABASE_MAX_CONNECTIONS` | `database.max_connections` | `u32`，必须大于 0 |
| `MARKSHAREX_DATABASE_MIN_CONNECTIONS` | `database.min_connections` | `u32`，不得大于最大连接数 |

### 认证

| 环境变量 | TOML 字段 | 类型/说明 |
|---|---|---|
| `MARKSHAREX_AUTH_JWT_SECRET` | `auth.jwt_secret` | 必填、非空；建议 `openssl rand -base64 64` |
| `MARKSHAREX_AUTH_JWT_EXPIRE_SECONDS` | `auth.jwt_expire_seconds` | `i64`，必须大于 0 |
| `MARKSHAREX_AUTH_REFRESH_EXPIRE_SECONDS` | `auth.refresh_expire_seconds` | `i64`，必须大于 0 |
| `MARKSHAREX_AUTH_ENCRYPT_KEY` | `auth.encrypt_key` | 必填、非空；必须长期固定，否则既有密文无法解密 |

旧环境变量 `MARKSHAREX_ENCRYPT_KEY` 仅作为兼容别名保留；新部署统一使用 `MARKSHAREX_AUTH_ENCRYPT_KEY`。如果两个变量同时存在，以新名称为准。

### 文件存储

| 环境变量 | TOML 字段 | 类型/说明 |
|---|---|---|
| `MARKSHAREX_STORAGE_UPLOAD_DIR` | `storage.upload_dir` | 字符串 |
| `MARKSHAREX_STORAGE_MAX_FILE_SIZE` | `storage.max_file_size` | `u64`，必须大于 0，单位字节 |
| `MARKSHAREX_STORAGE_ALLOWED_TYPES` | `storage.allowed_types` | 逗号分隔的 MIME 类型 |

### AI 与搜索

| 环境变量 | TOML 字段 | 类型/说明 |
|---|---|---|
| `MARKSHAREX_AI_MAX_TOOL_ROUNDS` | `ai.max_tool_rounds` | `u32`，必须大于 0 |
| `MARKSHAREX_AI_ALLOWED_PROVIDER_NETWORKS` | `ai.allowed_provider_networks` | 逗号分隔的 IP/CIDR |
| `MARKSHAREX_AI_SEARCH_PROVIDER` | `ai.search.provider` | 主搜索 Provider |
| `MARKSHAREX_AI_SEARCH_API_KEY` | `ai.search.api_key` | 主 Provider API Key；Tavily/Firecrawl 使用内置官方端点 |
| `MARKSHAREX_AI_SEARCH_FALLBACK_PROVIDER` | `ai.search.fallback_provider` | 降级 Provider |
| `MARKSHAREX_AI_SEARCH_FALLBACK_API_KEY` | `ai.search.fallback_api_key` | 降级 Provider API Key |
| `MARKSHAREX_AI_SEARCH_SEARXNG_URL` | `ai.search.searxng_url` | 自托管 SearXNG URL |
| `MARKSHAREX_AI_SEARCH_DUCKDUCKGO_URL` | `ai.search.duckduckgo_url` | DuckDuckGo Lite URL |
| `MARKSHAREX_AI_SEARCH_ALLOWED_SEARCH_NETWORKS` | `ai.search.allowed_search_networks` | 仅适用于配置型搜索服务的 IP/CIDR |

## 启动前校验

配置反序列化后还会执行以下语义校验：

- `server.port > 0`
- `database.max_connections > 0`
- `database.min_connections` 不得大于 `database.max_connections`。
- `auth.jwt_secret` 和 `auth.encrypt_key` 不得为空。
- `auth.jwt_expire_seconds` 和 `auth.refresh_expire_seconds` 必须大于 `0`。
- `storage.max_file_size` 必须大于 `0`。
- 配置了 `[ai]` 时，`ai.max_tool_rounds` 必须大于 `0`。
- `trusted_proxies` 必须全部是精确 IP；Provider/search network allowlist 必须全部是有效 IP/CIDR。
- CORS 必须是精确 HTTP(S) origin；SearXNG/DDG 地址必须是无凭据、查询和片段的 HTTP(S) URL。
- 搜索 Provider 仅接受 `tavily`、`firecrawl`、`searxng`、`duckduckgo`；选择 SearXNG 时必须配置其 URL。
- TOML 中的未知、拼错或废弃字段会直接报错；已删除的 `server.base_url` 和无运行时消费者的 `ai.search.api_url` 不再接受。

校验失败会阻止程序绑定端口、连接数据库或创建运行时目录。

## 安全说明

- 不要提交真实的 JWT secret、encryption key 或 AI API Key。
- encryption key 必须在部署后保持不变；更换后需要重新录入所有已加密的 API Key。
- `trusted_proxies` 只配置真实反向代理的精确 IP。
- CORS 默认留空；只有确需跨域的浏览器 origin 才加入 allowlist。
- Provider 和搜索网络 allowlist 只添加明确需要的 IP/CIDR，不要将 `0.0.0.0` 当作通配符。
