# 配置说明

MarkShareX 通过 `config.toml` 文件进行配置，所有配置项均可通过环境变量覆盖。

## 配置文件 (`config.toml`)

```toml
[server]
host = "0.0.0.0"
port = 5023
base_url = "http://localhost:5023"

[database]
url = "sqlite://./data/marksharex.db?mode=rwc"
max_connections = 10
min_connections = 1

[auth]
jwt_secret = "your-secret-key-change-in-production"
jwt_expire_seconds = 3600       # Access Token 1 小时
refresh_expire_seconds = 604800  # Refresh Token 7 天

[storage]
upload_dir = "./data/uploads"
max_file_size = 10485760         # 10 MB
allowed_types = ["image/jpeg", "image/png", "image/gif", "image/webp", "image/svg+xml", "application/pdf", "text/markdown", "text/plain"]

theme = "default"
data_dir = "./data"
```

## 环境变量

所有配置项均可通过环境变量覆盖，变量名格式为 `MARKSHAREX_<SECTION>_<KEY>`（大写，下划线分隔）。

| 环境变量 | 对应配置项 | 说明 |
|----------|-----------|------|
| `MARKSHAREX_SERVER_HOST` | `server.host` | 服务器监听地址 |
| `MARKSHAREX_SERVER_PORT` | `server.port` | 服务器监听端口 |
| `MARKSHAREX_SERVER_BASE_URL` | `server.base_url` | 公网访问 URL（用于生成文件链接） |
| `MARKSHAREX_DATA_DIR` | `data_dir` | 数据根目录（数据库、上传文件、主题等） |
| `MARKSHAREX_DATABASE_URL` | `database.url` | 数据库连接字符串 |
| `MARKSHAREX_DATABASE_MAX_CONNECTIONS` | `database.max_connections` | 数据库连接池最大连接数 |
| `MARKSHAREX_DATABASE_MIN_CONNECTIONS` | `database.min_connections` | 数据库连接池最小连接数 |
| `MARKSHAREX_AUTH_JWT_SECRET` | `auth.jwt_secret` | JWT 签名密钥（**生产环境必须修改**） |
| `MARKSHAREX_AUTH_JWT_EXPIRE_SECONDS` | `auth.jwt_expire_seconds` | Access Token 过期时间（秒） |
| `MARKSHAREX_AUTH_REFRESH_EXPIRE_SECONDS` | `auth.refresh_expire_seconds` | Refresh Token 过期时间（秒） |
| `MARKSHAREX_STORAGE_UPLOAD_DIR` | `storage.upload_dir` | 文件上传目录 |
| `MARKSHAREX_STORAGE_MAX_FILE_SIZE` | `storage.max_file_size` | 最大上传文件大小（字节） |
| `MARKSHAREX_THEME` | `theme` | 默认主题名称 |

### 环境变量示例 (`.env`)

```bash
MARKSHAREX_SERVER_HOST=0.0.0.0
MARKSHAREX_SERVER_PORT=8080
MARKSHAREX_SERVER_BASE_URL=https://blog.example.com
MARKSHAREX_DATA_DIR=./data
MARKSHAREX_DATABASE_URL=sqlite://./data/marksharex.db?mode=rwc
MARKSHAREX_DATABASE_MAX_CONNECTIONS=10
MARKSHAREX_DATABASE_MIN_CONNECTIONS=1
MARKSHAREX_AUTH_JWT_SECRET=your-secret-key-change-in-production
MARKSHAREX_AUTH_JWT_EXPIRE_SECONDS=3600
MARKSHAREX_AUTH_REFRESH_EXPIRE_SECONDS=604800
MARKSHAREX_STORAGE_UPLOAD_DIR=./data/uploads
MARKSHAREX_STORAGE_MAX_FILE_SIZE=10485760
MARKSHAREX_THEME=default
```

## 配置优先级

环境变量 > `config.toml` > 默认值

Docker 部署时建议使用 `.env` 文件或 `docker-compose.yml` 中的 `environment` 字段设置环境变量。
