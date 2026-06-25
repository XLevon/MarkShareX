# 网络资源库设计与实现方案

> 需求编号：REQ-20260524-0002

## 目标

统一管理系统引用的所有外部网络资源（图片、视频等），以 ID 替代裸 URL 存储，实现：
1. 一处修改 URL，所有引用处自动生效
2. 避免死链——可从管理界面发现失效资源
3. 导出/导入时，ID ↔ URL 透明转换

---

## 一、数据模型

### 新表 `network_resources`

```sql
CREATE TABLE network_resources (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    url         TEXT NOT NULL,          -- 实际网络 URL
    label       TEXT,                   -- 标签/描述（可选）
    source_type VARCHAR DEFAULT 'image', -- image | video | other
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX idx_network_resources_url ON network_resources(url);
```

### 受影响的表

| 表 | 新增字段 | 类型 | 说明 |
|----|---------|------|------|
| `categories` | `network_resource_id` | INTEGER FK → network_resources.id | 分类封面引用 |
| `posts` | `cover_network_id` | INTEGER FK → network_resources.id | 文章封面引用 |

### 内容中的网络图片

Markdown 内容中的网络图片引用方案：在内容文本中使用 `/nr/:id` 短链，渲染时实时解析。

```
存储: ![alt](/nr/5)
渲染: ![alt](https://actual-cdn.com/image.png)   ← 由 Markdown 渲染器实时替换
```

---

## 二、URL 解析优先级

读取时三级 fallback：

```
network_resource_id → image_url → image_filename
      │                    │            │
      ▼                    ▼            ▼
  SELECT url FROM      直接使用      拼接 base_url
  network_resources
```

---

## 三、API 设计

### 基础 CRUD

```
GET    /api/v1/network-resources             列表（分页，支持搜索）
POST   /api/v1/network-resources             添加 {url, label?, source_type?}
PUT    /api/v1/network-resources/:id          修改 URL
DELETE /api/v1/network-resources/:id          删除（需检查是否有引用）

GET    /api/v1/network-resources/:id/resolve  解析 → 302 重定向到实际 URL
```

### 自动入库（内部分类/文章调用）

```
POST /api/v1/network-resources/ensure
Body: {"url": "https://example.com/img.jpg"}
→ URL 已存在 → {"data": {"id": 5, "url": "..."}}
→ URL 不存在 → 创建 → {"data": {"id": 6, "url": "..."}}
```

### 批量查询

```
POST /api/v1/network-resources/batch-resolve
Body: {"ids": [5, 6, 7]}
→ {"data": {"5": "https://...", "6": "https://...", "7": "https://..."}}
```

---

## 四、导出/导入适配（关键！）

### 导出逻辑

```
          存储状态                    导出行为
    ┌─────────────────┐       ┌──────────────────────┐
    │ cover_network_id │  →   │ resolve → 实际 URL     │ → 写入 frontmatter cover_url
    │ = 5              │       │ https://cdn.../a.jpg  │
    └─────────────────┘       └──────────────────────┘

    ┌─────────────────┐       ┌──────────────────────┐
    │ 内容: ![a](/nr/5)│  →   │ resolve → 实际 URL     │ → ![a](https://cdn.../a.jpg)
    │                 │       │ 写入导出文件内容中       │
    └─────────────────┘       └──────────────────────┘
```

### 导入逻辑

```
    ┌──────────────────────────────┐
    │ frontmatter cover_url:       │
    │ https://cdn.example.com/a.jpg│
    └──────────┬───────────────────┘
               │ 解析 URL
               ▼
    ┌──────────────────────────────┐
    │ network_resources 查重       │
    │ WHERE url = 'https://...'     │
    └──────┬───────────┬───────────┘
           │           │
      已存在         不存在
           │           │
           ▼           ▼
      用已有 ID    创建新记录 → 新 ID
           │           │
           └─────┬─────┘
                 ▼
    ┌──────────────────────────────┐
    │ 存储 cover_network_id = 5    │
    └──────────────────────────────┘
```

**内容中网络图片的导入：**

```
原始内容: ![alt](https://cdn.example.com/img.png)
  ↓ 提取 URL → ensure → 得到 ID=5
  ↓ 替换内容引用
新内容: ![alt](/nr/5)
  ↓ 渲染时实时解析为实际 URL
```

---

## 五、受影响的模块及改造

| # | 模块 | 改造内容 | 复杂度 |
|---|------|---------|--------|
| 1 | `migrations/` | 新建 `network_resources` 表 + `categories`/`posts` 加 FK 字段 | ⭐⭐ |
| 2 | `models/entity/` | 新增 `network_resources.rs` 实体，修改 `categories.rs`、`posts.rs` | ⭐ |
| 3 | `controllers/network_resources.rs` | 新控制器：CRUD + ensure + batch-resolve + resolve(302) | ⭐⭐⭐ |
| 4 | `controllers/mod.rs` | 注册新路由 | ⭐ |
| 5 | `controllers/categories.rs` | 保存时：URL → ensure → 存 ID；读取时：ID → resolve URL | ⭐⭐⭐ |
| 6 | `controllers/posts.rs` | 同上，封面字段适配 | ⭐⭐⭐ |
| 7 | `controllers/import_export.rs` | 导出：ID → URL；导入：URL → ensure → ID；内容 `/nr/:id` ↔ URL | ⭐⭐⭐⭐ |
| 8 | `services/posts.rs` | Markdown 渲染时解析 `/nr/:id` → 实际 URL | ⭐⭐ |
| 9 | `templates/default/post.html` | 封面渲染改用 resolve 后的 URL | ⭐ |
| 10 | `frontend/views/admin/NetworkResources.vue` | 新页面：网络资源管理（表格+添加弹窗） | ⭐⭐⭐ |
| 11 | `frontend/views/admin/Files.vue` | 增加「网络资源」Tab | ⭐ |
| 12 | 分类/文章编辑器 | 资源选择器增加「网络资源」选项 | ⭐⭐⭐ |

---

## 六、实施分阶段

### P1：基础设施（~2h）

```
✅ 迁移脚本：network_resources 表 + FK 字段
✅ 实体模型：network_resources.rs + 修改 categories/posts
✅ 后端 CRUD：controllers/network_resources.rs
✅ 注册路由
```

### P2：前端管理（~2h）

```
✅ NetworkResources.vue — 列表、添加、编辑、删除
✅ Files.vue — 增加「网络资源」Tab
```

### P3：分类/封面集成（~2.5h）

```
✅ categories 控制器：URL → ensure → ID 自动入库
✅ posts 控制器：cover_image URL → ensure → ID 自动入库
✅ 前端编辑器：分类封面、文章封面选择器增加网络资源选项
```

### P4：导出/导入适配（~2h）

```
✅ 导出：ID → resolve → 实际 URL（cover + 内容图片）
✅ 导入：URL → ensure → ID（cover + 内容图片）
✅ 内容 `/nr/:id` ↔ URL 互转
```

### P5：渲染适配（~1h）

```
✅ Markdown 渲染器：`/nr/:id` → 查询 network_resources → 替换为实际 URL
✅ 模板页面：封面 URL 使用 resolve
✅ 测试验证
```

## 七、关键复杂度

1. **URL 唯一索引** — 同一 URL 只存一条，多处引用共享
2. **级联更新** — 修改 network_resource.url 后所有引用处自动生效（运行时解析，无需更新引用表）
3. **删除保护** — 删除前检查 categories/posts 的引用，防止孤儿 ID
4. **内容图片 `/nr/:id`** — 需要同时修改 Markdown 渲染管道和导入导出两个方向
5. **向后兼容** — 已有直接填 URL 的数据保持原样，三级 fallback 保证不破坏
6. **自动入库透明化** — 用户填 URL → 系统自动 ensure → 存 ID，对前端透明

## 八、工时估算

| 阶段 | 工时 |
|------|------|
| P1 基础设施 | 2h |
| P2 前端管理 | 2h |
| P3 分类/封面集成 | 2.5h |
| P4 导出/导入适配 | 2h |
| P5 渲染适配 | 1h |
| **合计** | **~9.5h** |
