# MarkShareX AI API 友好化方案

> 需求编号：REQ-20260524-0001

## 目标

让 AI 工具（Claude、GPT、Hermes、Cursor 等）能够安全、便捷地访问 MarkShareX API，实现自动化文章管理、文件管理、设置修改等操作。

---

## 已实现：方案 A+B（API Key + 端点发现）

### 端点发现 — `GET /api/v1/`

```json
{
  "data": [
    {"method": "GET",  "path": "/api/v1/posts",     "description": "获取文章列表",   "auth_required": false},
    {"method": "POST", "path": "/api/v1/posts",     "description": "创建文章",       "auth_required": true},
    {"method": "GET",  "path": "/api/v1/search",    "description": "全文搜索",       "auth_required": false},
    {"method": "GET",  "path": "/api/v1/files",     "description": "获取文件列表",   "auth_required": true},
    ...
  ]
}
```

### API Key 认证

- **绑定用户**：Key 存在 `users.api_key` 列，权限等同该用户（管理员/子管理员/作者）
- **认证头**：`X-API-Key: msx-xxxxxxxx`
- **不传 Key**：走正常 JWT 登录，零影响现有逻辑
- **管理入口**：后台/前台 → 个人下拉菜单 → API Key

```bash
# 使用示例
curl -H "X-API-Key: msx-xxxxxxxx" http://host:5023/api/v1/posts
```

### 端点

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/profile/api-key` | 获取当前用户的 Key |
| PUT | `/api/v1/profile/api-key` | 生成/重置 Key |

### 数据库变更

```sql
-- migrations/20260524_001_add_api_key_to_users.sql
ALTER TABLE users ADD COLUMN api_key VARCHAR;
```

---

## 待实现：方案 C — MCP Server

### 概述

独立 Python 项目 `marksharex-mcp`，通过 MCP（Model Context Protocol）协议暴露 MarkShareX API 为 AI 原生工具。AI 客户端可直接 `list_posts()`、`create_post()` 调用，无需理解 HTTP/REST。

```
┌─────────────────────────────────┐
│  AI Client (Hermes/Claude/Cursor) │
└──────────┬──────────────────────┘
           │ MCP Protocol (stdio)
           ▼
┌─────────────────────────────────┐
│     marksharex-mcp (Python)      │
│  ┌───────────────────────────┐  │
│  │  MCP Server (mcp SDK)     │  │
│  │  ├─ list_posts            │  │
│  │  ├─ create_post           │  │
│  │  ├─ upload_file           │  │
│  │  └─ ... 20 tools          │  │
│  └───────────┬───────────────┘  │
│              │ HTTP + X-API-Key  │
└──────────────┼───────────────────┘
               ▼
┌─────────────────────────────────┐
│       MarkShareX REST API        │
│       http://host:5023           │
└─────────────────────────────────┘
```

### 工具清单（8 组，20 个工具）

#### 📝 文章（Posts）

| 工具名 | 描述 | 参数 |
|--------|------|------|
| `list_posts` | 获取文章列表 | `page?`, `page_size?`, `status?`, `category_id?`, `tag_id?` |
| `get_post` | 根据 ID 获取文章详情 | `post_id: int` |
| `get_post_by_slug` | 根据 slug 获取文章 | `slug: str` |
| `create_post` | 创建新文章 | `title`, `content`, `slug?`, `summary?`, `category_id?`, `tags?`, `status?` |
| `update_post` | 更新文章 | `post_id`, `title?`, `content?`, `summary?`, `status?`, ... |
| `delete_post` | 删除文章 | `post_id` |
| `search_posts` | 全文搜索 | `q: str` |

#### 🖼️ 文件（Files）

| 工具名 | 描述 | 参数 |
|--------|------|------|
| `list_files` | 资源库文件列表 | `page?`, `page_size?` |
| `upload_file` | 上传文件 | `file_path: str`（MCP 客户端的本地路径） |
| `delete_file` | 删除文件 | `file_id` |

#### 📂 分类与标签

| 工具名 | 描述 | 参数 |
|--------|------|------|
| `list_categories` | 分类列表 | — |
| `create_category` | 创建分类 | `name`, `slug?`, `description?` |
| `list_tags` | 标签列表 | — |
| `create_tag` | 创建标签 | `name` |

#### ⚙️ 设置与统计

| 工具名 | 描述 | 参数 |
|--------|------|------|
| `get_settings` | 获取站点设置 | — |
| `update_settings` | 更新站点设置 | `settings: dict` |
| `get_analytics` | 浏览/点赞/评论统计 | — |
| `get_trend` | 访问趋势 | `days?` (默认 7) |

#### 📦 导入导出

| 工具名 | 描述 | 参数 |
|--------|------|------|
| `export_posts` | 导出文章为 Markdown | `post_ids?: list` |
| `import_posts` | 导入 Markdown 文章 | `file_path: str` |

#### 👤 用户管理（管理员权限）

| 工具名 | 描述 | 参数 |
|--------|------|------|
| `list_users` | 用户列表 | `page?`, `page_size?` |

### 项目结构

```
marksharex-mcp/
├── server.py              # MCP Server 入口（~30行）
├── client.py              # MarkShareX API 客户端（~80行）
├── tools/
│   ├── __init__.py
│   ├── posts.py           # 文章工具（7 个）
│   ├── files.py           # 文件工具（3 个）
│   ├── categories.py      # 分类/标签工具（4 个）
│   ├── settings.py        # 设置/统计工具（4 个）
│   └── users.py           # 用户管理/导入导出（3 个）
├── pyproject.toml
└── README.md
```

### 依赖

```toml
[project]
name = "marksharex-mcp"
version = "0.2.0"
requires-python = ">=3.10"
dependencies = ["mcp>=1.0", "httpx>=0.28"]

[project.scripts]
marksharex-mcp = "server:main"
```

### 使用方式

**Hermes Agent 配置（`~/.hermes/config.yaml`）：**

```yaml
mcp_servers:
  marksharex:
    command: "uv"
    args: ["run", "--directory", "/opt/data/projects/marksharex-mcp", "server.py"]
    env:
      MARKSHAREX_BASE_URL: "http://192.168.110.211:5023"
      MARKSHAREX_API_KEY: "msx-xxxxxxxx"   # 从后台个人菜单生成
    timeout: 60
```

**Claude Desktop 配置：**

```json
{
  "mcpServers": {
    "marksharex": {
      "command": "uv",
      "args": ["run", "--directory", "/path/to/marksharex-mcp", "server.py"],
      "env": {
        "MARKSHAREX_BASE_URL": "https://your-blog.com",
        "MARKSHAREX_API_KEY": "msx-xxxxxxxx"
      }
    }
  }
}
```

### 关键代码骨架

**server.py：**

```python
import os
from mcp.server import Server
from mcp.server.stdio import stdio_server

from client import MarkShareXClient
from tools.posts import register_post_tools
from tools.files import register_file_tools
from tools.categories import register_category_tools
from tools.settings import register_settings_tools
from tools.users import register_user_tools

def main():
    base_url = os.environ["MARKSHAREX_BASE_URL"]
    api_key = os.environ["MARKSHAREX_API_KEY"]
    client = MarkShareXClient(base_url, api_key)

    server = Server("marksharex-mcp")

    register_post_tools(server, client)
    register_file_tools(server, client)
    register_category_tools(server, client)
    register_settings_tools(server, client)
    register_user_tools(server, client)

    async def run():
        async with stdio_server() as (read, write):
            await server.run(read, write, server.create_initialization_options())

    import asyncio
    asyncio.run(run())
```

**client.py：**

```python
import httpx

class MarkShareXClient:
    def __init__(self, base_url: str, api_key: str):
        self.base_url = base_url.rstrip("/")
        self.headers = {"X-API-Key": api_key}

    async def get(self, path: str, **params) -> dict:
        async with httpx.AsyncClient() as c:
            r = await c.get(f"{self.base_url}/api/v1{path}",
                            headers=self.headers, params=params, timeout=30)
            r.raise_for_status()
            return r.json()

    async def post(self, path: str, data: dict) -> dict:
        async with httpx.AsyncClient() as c:
            r = await c.post(f"{self.base_url}/api/v1{path}",
                             headers=self.headers, json=data, timeout=30)
            r.raise_for_status()
            return r.json()

    async def put(self, path: str, data: dict) -> dict:
        async with httpx.AsyncClient() as c:
            r = await c.put(f"{self.base_url}/api/v1{path}",
                            headers=self.headers, json=data, timeout=30)
            r.raise_for_status()
            return r.json()

    async def delete(self, path: str) -> dict:
        async with httpx.AsyncClient() as c:
            r = await c.delete(f"{self.base_url}/api/v1{path}",
                               headers=self.headers, timeout=30)
            r.raise_for_status()
            return r.json()
```

**tools/posts.py（示例）：**

```python
from mcp.server import Server
from mcp.types import Tool, TextContent

def register_post_tools(server: Server, client):
    """注册文章相关工具"""

    @server.call_tool()
    async def list_posts(arguments: dict) -> list[TextContent]:
        """获取文章列表，支持分页和过滤"""
        result = await client.get("/posts", **arguments)
        posts = result.get("data", [])
        pagination = result.get("pagination", {})
        text = f"共 {pagination.get('total', len(posts))} 篇文章\n\n"
        for p in posts:
            text += f"- [{p['id']}] {p['title']} ({p.get('status', '?')})\n"
        return [TextContent(type="text", text=text)]

    @server.call_tool()
    async def get_post(arguments: dict) -> list[TextContent]:
        """根据 ID 获取文章详情"""
        post_id = arguments["post_id"]
        result = await client.get(f"/posts/{post_id}")
        post = result.get("data", {})
        text = f"## {post.get('title')}\n\n{post.get('content', '')}"
        return [TextContent(type="text", text=text)]

    @server.call_tool()
    async def create_post(arguments: dict) -> list[TextContent]:
        """创建新文章"""
        result = await client.post("/posts", arguments)
        post = result.get("data", {})
        return [TextContent(type="text",
                text=f"文章创建成功: [{post['id']}] {post['title']}")]

    @server.call_tool()
    async def update_post(arguments: dict) -> list[TextContent]:
        """更新文章"""
        post_id = arguments.pop("post_id")
        result = await client.put(f"/posts/{post_id}", arguments)
        post = result.get("data", {})
        return [TextContent(type="text",
                text=f"文章更新成功: [{post['id']}] {post.get('title')}")]

    @server.call_tool()
    async def delete_post(arguments: dict) -> list[TextContent]:
        """删除文章（软删除）"""
        post_id = arguments["post_id"]
        await client.delete(f"/posts/{post_id}")
        return [TextContent(type="text", text=f"文章 {post_id} 已删除")]

    @server.call_tool()
    async def get_post_by_slug(arguments: dict) -> list[TextContent]:
        """根据 slug 获取文章"""
        slug = arguments["slug"]
        result = await client.get(f"/posts/slug/{slug}")
        post = result.get("data", {})
        text = f"## {post.get('title')}\n\n{post.get('content', '')}"
        return [TextContent(type="text", text=text)]

    @server.call_tool()
    async def search_posts(arguments: dict) -> list[TextContent]:
        """全文搜索文章"""
        q = arguments["q"]
        result = await client.get("/search", q=q)
        posts = result.get("data", [])
        if not posts:
            return [TextContent(type="text", text=f"未找到与「{q}」相关的文章")]
        text = f"搜索「{q}」的结果：\n\n"
        for p in posts:
            text += f"- [{p['id']}] {p['title']}\n"
        return [TextContent(type="text", text=text)]
```

### 开发量估算

| 模块 | 文件 | 工时 |
|------|------|------|
| 项目骨架 | `server.py`, `client.py`, `pyproject.toml` | 1h |
| 文章工具 | `tools/posts.py`（7 个工具） | 2h |
| 文件工具 | `tools/files.py`（3 个工具） | 1h |
| 分类/标签 | `tools/categories.py`（4 个工具） | 1h |
| 设置/统计 | `tools/settings.py`（4 个工具） | 1h |
| 用户/导入导出 | `tools/users.py`（3 个工具） | 0.5h |
| 测试 + README | — | 1.5h |
| **合计** | | **~8h** |

---

## 安全考量

- API Key 权限等同于用户自身角色
- 建议仅在内网或 VPN 环境使用
- 未来可扩展：多 Key + 权限分级（只读/读写/管理员）+ 访问日志
