//! AI 工具执行引擎 — Tool trait + 内置工具实现 + 注册调度
//!
//! 支持 Tavily 和 Firecrawl 作为搜索后端。
//! 工具通过 ai_tools 数据库表注册，运行时从 DB 读取并 dispatch。

use async_trait::async_trait;
use chrono::Local;
use sea_orm::*;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::config::AiSearchConfig;
use crate::utils::{AppState, AppError};
use crate::models::entity::news;
use crate::models::entity::posts;
use crate::models::entity::categories;

// ═══════════════════════════════════════════════════════════════════════
//  Tool trait
// ═══════════════════════════════════════════════════════════════════════

/// AI 工具的统一接口。
/// 所有可被 LLM function calling 调用的工具都实现此 trait。
#[async_trait]
pub trait AiTool: Send + Sync {
    /// 数据库中的 function_name（与 ai_tools 表对应）
    fn name(&self) -> &str;

    /// 工具描述（给 LLM 看的）
    fn description(&self) -> &str;

    /// OpenAI function calling 格式的 parameters JSON Schema
    fn parameters(&self) -> Value;

    /// 执行工具，返回结果文本
    async fn execute(&self, args: Value, state: &AppState) -> Result<String, AppError>;
}

// ═══════════════════════════════════════════════════════════════════════
//  Tool Registry — 运行时从 DB 加载工具
// ═══════════════════════════════════════════════════════════════════════

pub struct ToolRegistry {
    /// function_name → tool instance
    tools: HashMap<String, Arc<dyn AiTool>>,
}

impl ToolRegistry {
    /// 创建空注册表，后续通过 register 添加工具
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    /// 注册一个工具
    pub fn register(&mut self, tool: Arc<dyn AiTool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// 查找工具
    pub fn get(&self, name: &str) -> Option<&Arc<dyn AiTool>> {
        self.tools.get(name)
    }

    /// 生成 OpenAI function calling 格式的 tools 列表
    pub fn to_openai_tools(&self) -> Vec<Value> {
        self.tools
            .values()
            .map(|t| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": t.name(),
                        "description": t.description(),
                        "parameters": t.parameters(),
                    }
                })
            })
            .collect()
    }

    /// 列出所有已注册的工具名
    #[allow(dead_code)]
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// 检查是否已注册指定名称的工具
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  Helper: 获取搜索配置
// ═══════════════════════════════════════════════════════════════════════

fn get_search_config(state: &AppState) -> Result<&AiSearchConfig, AppError> {
    state.config.ai.as_ref()
        .and_then(|a| a.search.as_ref())
        .ok_or_else(|| AppError::BadRequest("AI 搜索未配置，请在 config.toml 中设置 [ai.search]".into()))
}

fn get_search_api_key(state: &AppState) -> Result<String, AppError> {
    let cfg = get_search_config(state)?;
    // 优先从环境变量读取
    if let Ok(key) = std::env::var("MARKSHAREX_AI_SEARCH_API_KEY") {
        if !key.is_empty() {
            return Ok(key);
        }
    }
    if cfg.api_key.is_empty() {
        return Err(AppError::BadRequest(
            "AI 搜索 API Key 未设置。请在 config.toml [ai.search] 中设置 api_key 或设置环境变量 MARKSHAREX_AI_SEARCH_API_KEY".into()
        ));
    }
    Ok(cfg.api_key.clone())
}

// ═══════════════════════════════════════════════════════════════════════
//  Built-in Tool: web_search
// ═══════════════════════════════════════════════════════════════════════

pub struct WebSearchTool;

#[async_trait]
impl AiTool for WebSearchTool {
    fn name(&self) -> &str { "web_search" }

    fn description(&self) -> &str {
        "搜索网络资讯，返回标题、URL 和摘要。适合查找最新新闻、技术动态等。"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "搜索关键词"
                },
                "limit": {
                    "type": "integer",
                    "description": "返回结果数量，默认 5，最大 10",
                    "default": 5
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: Value, state: &AppState) -> Result<String, AppError> {
        let query = args["query"].as_str().unwrap_or("").to_string();
        let limit = args["limit"].as_u64().unwrap_or(5).min(10) as usize;

        if query.is_empty() {
            return Err(AppError::BadRequest("搜索关键词不能为空".into()));
        }

        let cfg = get_search_config(state)?;
        let api_key = get_search_api_key(state)?;

        let client = reqwest::Client::new();
        let payload = match cfg.provider.as_str() {
            "tavily" => serde_json::json!({
                "api_key": api_key,
                "query": query,
                "max_results": limit,
                "include_raw_content": false,
                "include_images": false,
            }),
            "firecrawl" => serde_json::json!({
                "query": query,
                "limit": limit,
            }),
            _ => return Err(AppError::BadRequest(format!("不支持的搜索提供商: {}", cfg.provider))),
        };

        let endpoint = match cfg.provider.as_str() {
            "tavily" => format!("{}/search", cfg.api_url()),
            "firecrawl" => format!("{}/v1/search", cfg.api_url()),
            _ => unreachable!(),
        };

        let mut req = client.post(&endpoint).json(&payload);
        if cfg.provider == "firecrawl" {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let resp = req.send().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("搜索请求失败: {}", e)))?;

        let body: Value = resp.json().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("搜索响应解析失败: {}", e)))?;

        // 标准化输出
        let results: Vec<Value> = match cfg.provider.as_str() {
            "tavily" => {
                body.get("results").cloned().unwrap_or_default()
                    .as_array().cloned().unwrap_or_default()
                    .into_iter()
                    .map(|r| serde_json::json!({
                        "title": r["title"],
                        "url": r["url"],
                        "description": r["content"],
                    }))
                    .collect()
            }
            "firecrawl" => {
                body.get("data").and_then(|d| d.as_array()).cloned().unwrap_or_default()
                    .into_iter()
                    .map(|r| serde_json::json!({
                        "title": r["title"],
                        "url": r["url"],
                        "description": r["description"],
                    }))
                    .collect()
            }
            _ => vec![],
        };

        Ok(serde_json::to_string_pretty(&serde_json::json!({
            "success": true,
            "count": results.len(),
            "results": results,
        })).unwrap_or_default())
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  Built-in Tool: web_extract
// ═══════════════════════════════════════════════════════════════════════

pub struct WebExtractTool;

#[async_trait]
impl AiTool for WebExtractTool {
    fn name(&self) -> &str { "web_extract" }

    fn description(&self) -> &str {
        "抓取指定 URL 的网页内容，返回 Markdown 格式正文。适合获取文章全文。"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "urls": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "要抓取的 URL 列表，最多 5 个",
                    "maxItems": 5
                }
            },
            "required": ["urls"]
        })
    }

    async fn execute(&self, args: Value, state: &AppState) -> Result<String, AppError> {
        let urls: Vec<String> = args["urls"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        if urls.is_empty() {
            return Err(AppError::BadRequest("请提供至少一个 URL".into()));
        }

        let cfg = get_search_config(state)?;
        let api_key = get_search_api_key(state)?;
        let client = reqwest::Client::new();

        let mut results = Vec::new();
        for url in urls.iter().take(5) {
            let payload = match cfg.provider.as_str() {
                "tavily" => serde_json::json!({
                    "api_key": api_key,
                    "urls": [url],
                    "include_images": false,
                }),
                "firecrawl" => serde_json::json!({
                    "url": url,
                    "formats": ["markdown"],
                }),
                _ => continue,
            };

            let endpoint = match cfg.provider.as_str() {
                "tavily" => format!("{}/extract", cfg.api_url()),
                "firecrawl" => format!("{}/v1/scrape", cfg.api_url()),
                _ => continue,
            };

            let mut req = client.post(&endpoint).json(&payload);
            if cfg.provider == "firecrawl" {
                req = req.header("Authorization", format!("Bearer {}", api_key));
            }

            match req.send().await {
                Ok(resp) => {
                    if let Ok(body) = resp.json::<Value>().await {
                        let content = match cfg.provider.as_str() {
                            "tavily" => body["results"][0]["raw_content"]
                                .as_str().unwrap_or("").to_string(),
                            "firecrawl" => body["data"]["markdown"]
                                .as_str().or(body["data"]["content"].as_str())
                                .unwrap_or("").to_string(),
                            _ => String::new(),
                        };
                        // 截断过长内容（按字符而非字节，避免 UTF-8 边界 panic）
                        let truncated = if content.len() > 15000 {
                            let char_count = content.chars().count();
                            let max_chars = 15000usize.min(char_count);
                            let safe: String = content.chars().take(max_chars).collect();
                            format!("{}...\n\n(内容过长，已截断至前 {} 字符)", safe, max_chars)
                        } else {
                            content
                        };
                        results.push(serde_json::json!({
                            "url": url,
                            "content": truncated,
                        }));
                    } else {
                        // 回退：直接 HTTP GET
                        match fetch_url_directly(url).await {
                            Ok(text) => results.push(serde_json::json!({"url": url, "content": text, "fallback": true})),
                            Err(e) => results.push(serde_json::json!({"url": url, "error": e.to_string()})),
                        }
                    }
                }
                Err(_e) => {
                    // 回退：直接 HTTP GET
                    match fetch_url_directly(url).await {
                        Ok(text) => results.push(serde_json::json!({"url": url, "content": text, "fallback": true})),
                        Err(e2) => results.push(serde_json::json!({"url": url, "error": e2.to_string()})),
                    }
                }
            }
        }

        Ok(serde_json::to_string_pretty(&serde_json::json!({
            "success": true,
            "count": results.len(),
            "results": results,
        })).unwrap_or_default())
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  Built-in Tool: create_news
// ═══════════════════════════════════════════════════════════════════════

pub struct CreateNewsTool;

#[async_trait]
impl AiTool for CreateNewsTool {
    fn name(&self) -> &str { "create_news" }

    fn description(&self) -> &str {
        "创建一条资讯。需要提供 title（标题）、summary（摘要）、content（Markdown 正文）。可选 source_url（来源链接）、status（draft 草稿 或 published 已发布，默认 draft）。"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "资讯标题"
                },
                "summary": {
                    "type": "string",
                    "description": "简短摘要，200字以内"
                },
                "content": {
                    "type": "string",
                    "description": "Markdown 格式正文"
                },
                "source_url": {
                    "type": "string",
                    "description": "原文链接（可选）"
                },
                "status": {
                    "type": "string",
                    "description": "发布状态：draft（草稿，默认）或 published（已发布）",
                    "enum": ["draft", "published"],
                    "default": "draft"
                }
            },
            "required": ["title", "summary", "content"]
        })
    }

    async fn execute(&self, args: Value, state: &AppState) -> Result<String, AppError> {
        let title = args["title"].as_str().unwrap_or("").to_string();
        let summary = args["summary"].as_str().unwrap_or("").to_string();
        let content = args["content"].as_str().unwrap_or("").to_string();
        let _source_url = args["source_url"].as_str().unwrap_or("").to_string();
        let status = args["status"].as_str().unwrap_or("draft").to_string();

        if title.is_empty() {
            return Err(AppError::BadRequest("标题不能为空".into()));
        }

        if status != "draft" && status != "published" {
            return Err(AppError::BadRequest("status 只能是 draft 或 published".into()));
        }

        // 生成 content_html
        let content_html = crate::services::posts::render_markdown(&state.db, &content).await;

        // 保存副本用于响应
        let content_preview = content.chars().take(800).collect::<String>();
        let summary_clone = summary.clone();

        let now = crate::utils::now_local();
        let is_published = status == "published";
        let published_at = if is_published { Some(now) } else { None };

        let model = news::ActiveModel {
            title: Set(title.clone()),
            summary: Set(summary),
            content: Set(content),
            content_html: Set(content_html),
            status: Set(status.clone()),
            sort_order: Set(0),
            published_at: Set(published_at),
            user_id: Set(None), // AI 创建，无用户关联
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let inserted = model.insert(&state.db).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("创建资讯失败: {}", e)))?;

        let status_label = if is_published { "已发布" } else { "草稿" };

        Ok(serde_json::to_string_pretty(&serde_json::json!({
            "success": true,
            "message": format!("资讯「{}」已创建（{}），ID: {}", title, status_label, inserted.id),
            "news_id": inserted.id,
            "title": title,
            "summary": summary_clone,
            "status": status,
            "content_preview": content_preview,
            "draft_url": format!("http://{}:{}/admin/news/{}", state.config.server.host, state.config.server.port, inserted.id),
        })).unwrap_or_default())
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  Built-in Tool: create_post — AI 自动发布知识库文章
// ═══════════════════════════════════════════════════════════════════════

pub struct CreatePostTool;

#[async_trait]
impl AiTool for CreatePostTool {
    fn name(&self) -> &str { "create_post" }

    fn description(&self) -> &str {
        "创建一篇知识库文章。需要提供 title（标题）、content（Markdown 正文，可用 nr:ID 引用资源库图片）、category_id（分类 ID）。可选 summary、cover_image（nr:ID 或 URL）、status（draft 或 published，默认 draft）。"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "文章标题"
                },
                "content": {
                    "type": "string",
                    "description": "Markdown 格式正文。图片有三种写法：1) 本地资源: ![alt](/uploads/文件名.jpg)；2) 网络资源: ![alt](nr:资源ID) 引用资源库图片；3) 外部图片: ![alt](https://...)"
                },
                "category_id": {
                    "type": "integer",
                    "description": "分类 ID（必填，需先查询 categories 获取）"
                },
                "summary": {
                    "type": "string",
                    "description": "文章摘要（可选）"
                },
                "cover_image": {
                    "type": "string",
                    "description": "封面图：nr:资源ID 或完整 URL（可选）"
                },
                "status": {
                    "type": "string",
                    "description": "发布状态：draft（草稿，默认）或 published（已发布）",
                    "enum": ["draft", "published"],
                    "default": "draft"
                }
            },
            "required": ["title", "content", "category_id"]
        })
    }

    async fn execute(&self, args: Value, state: &AppState) -> Result<String, AppError> {
        let title = args["title"].as_str().unwrap_or("").to_string();
        let content = args["content"].as_str().unwrap_or("").to_string();
        let summary = args["summary"].as_str().unwrap_or("").to_string();
        let cover_image = args["cover_image"].as_str().unwrap_or("").to_string();
        let status = args["status"].as_str().unwrap_or("draft").to_string();
        let category_id: i32 = args["category_id"].as_i64().unwrap_or(0) as i32;

        if title.is_empty() { return Err(AppError::BadRequest("标题不能为空".into())); }
        if content.is_empty() { return Err(AppError::BadRequest("内容不能为空".into())); }
        if category_id <= 0 { return Err(AppError::BadRequest("category_id 无效".into())); }
        if status != "draft" && status != "published" {
            return Err(AppError::BadRequest("status 只能是 draft 或 published".into()));
        }

        // 验证分类存在
        let cat = categories::Entity::find_by_id(category_id)
            .filter(categories::Column::DeletedAt.is_null())
            .one(&state.db).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("查询分类失败: {}", e)))?
            .ok_or_else(|| AppError::BadRequest(format!("分类 #{} 不存在", category_id)))?;

        // 生成 slug
        let slug = crate::services::posts::generate_slug(&title);

        // 生成 content_html（支持 nr:{id} 图片引用解析）
        let content_html = crate::services::posts::render_markdown(&state.db, &content).await;

        let now = crate::utils::now_local();
        let is_published = status == "published";
        let published_at = if is_published { Some(now) } else { None };

        // 处理封面图
        let (cover_url, cover_filename, cover_nr_id) = if !cover_image.is_empty() {
            if cover_image.starts_with("nr:") {
                // 资源库图片引用
                let nr_id: i32 = cover_image.trim_start_matches("nr:").parse().unwrap_or(0);
                (Some(cover_image.clone()), None, Some(nr_id))
            } else if cover_image.starts_with("http://") || cover_image.starts_with("https://") {
                (Some(cover_image.clone()), None, None)
            } else {
                (None, Some(cover_image.clone()), None)
            }
        } else {
            (None, None, None)
        };

        let summary_opt = if summary.is_empty() { None } else { Some(summary) };

        let model = posts::ActiveModel {
            title: Set(title.clone()),
            slug: Set(slug),
            content: Set(Some(content)),
            content_html: Set(Some(content_html)),
            summary: Set(summary_opt),
            category_id: Set(Some(category_id)),
            cover_image_url: Set(cover_url),
            cover_image_filename: Set(cover_filename),
            cover_network_id: Set(cover_nr_id),
            status: Set(status.clone()),
            post_type: Set("article".to_string()),
            article_type: Set("original".to_string()),
            article_status: Set("completed".to_string()),
            is_pinned: Set(false),
            allow_comment: Set(true),
            sort_order: Set(0),
            view_count: Set(0),
            like_count: Set(0),
            comment_count: Set(0),
            published_at: Set(published_at),
            user_id: Set(1), // AI 创建，默认 admin
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let inserted = model.insert(&state.db).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("创建文章失败: {}", e)))?;

        let status_label = if is_published { "已发布" } else { "草稿" };
        let cat_name = cat.name;

        Ok(serde_json::to_string_pretty(&serde_json::json!({
            "success": true,
            "message": format!("文章「{}」已创建（{}）", title, status_label),
            "post_id": inserted.id,
            "title": title,
            "slug": inserted.slug,
            "category": cat_name,
            "status": status,
            "url": format!("http://{}:{}/posts/{}", state.config.server.host, state.config.server.port, inserted.slug),
        })).unwrap_or_default())
    }
}
// ═══════════════════════════════════════════════════════════════════════

pub struct GetCurrentDatetimeTool;

#[async_trait]
impl AiTool for GetCurrentDatetimeTool {
    fn name(&self) -> &str { "get_current_datetime" }

    fn description(&self) -> &str {
        "获取当前日期和时间（含星期），返回服务器本地时间（CST/UTC+8）。无需参数。"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _args: Value, _state: &AppState) -> Result<String, AppError> {
        let now = Local::now();
        Ok(now.format("%Y年%m月%d日 %H:%M:%S %A (UTC+8)").to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  Built-in Tool: api_request
// ═══════════════════════════════════════════════════════════════════════

pub struct ApiRequestTool {
    /// 当前登录用户的 token，用于以用户身份调用 API。cron 调度时为 None。
    user_token: Option<String>,
}

#[async_trait]
impl AiTool for ApiRequestTool {
    fn name(&self) -> &str { "api_request" }

    fn description(&self) -> &str {
        "搜索站内资源或调用外部 API 并返回超链接。用于调用站内搜索（GET /api/v1/search?q=关键词），或查询分类、标签、用户等。返回 JSON 结果，你自行解析并整理为可读格式，以 Markdown 超链接呈现。也可调用外部接口。"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "请求 URL"
                },
                "method": {
                    "type": "string",
                    "description": "HTTP 方法：GET 或 POST",
                    "enum": ["GET", "POST"],
                    "default": "GET"
                },
                "body": {
                    "type": "string",
                    "description": "POST 请求体（JSON 字符串），仅 method=POST 时使用"
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self, args: Value, state: &AppState) -> Result<String, AppError> {
        let url = args["url"].as_str().unwrap_or("").to_string();
        let method = args["method"].as_str().unwrap_or("GET").to_uppercase();

        if url.is_empty() {
            return Err(AppError::BadRequest("URL 不能为空".into()));
        }

        // 相对路径自动补全为完整 URL（如 /api/v1/search → http://127.0.0.1:5023/api/v1/search）
        let url = if url.starts_with("http://") || url.starts_with("https://") {
            url
        } else {
            let path = url.trim_start_matches('/');
            format!("http://127.0.0.1:{}/{}", state.config.server.port, path)
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .danger_accept_invalid_certs(false)
            .build()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("创建 HTTP 客户端失败: {}", e)))?;

        let mut req = match method.as_str() {
            "POST" => {
                let body = args["body"].as_str().unwrap_or("{}").to_string();
                client.post(&url).header("Content-Type", "application/json").body(body)
            }
            _ => client.get(&url),
        };

        // 仅本站 API 注入当前用户 token；外部请求由调用者从上下文推断认证方式
        if let Some(ref token) = self.user_token {
            if is_local_service_url(&url, &state.config.server.host, state.config.server.port) {
                req = req.header("Authorization", format!("Bearer {}", token));
            }
        }

        let resp = req.send().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("API 请求失败: {}", e)))?;

        let status = resp.status();
        let body = resp.text().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("读取响应失败: {}", e)))?;

        // 截断过长响应（按字符而非字节截断，防止中文截断 panic）
        let truncated = if body.chars().count() > 8000 {
            let preview: String = body.chars().take(8000).collect();
            format!("{}...\n\n(响应过长，已截断至前 8000 字符)", preview)
        } else {
            body
        };

        Ok(format!("HTTP {} — 响应:\n{}", status.as_u16(), truncated))
    }
}


/// 辅助函数：直接抓取 URL 并提取文本
async fn fetch_url_directly(url: &str) -> Result<String, AppError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("创建 HTTP 客户端失败: {}", e)))?;

    let resp = client.get(url)
        .header("User-Agent", "Mozilla/5.0 (compatible; MarkShareX/1.0)")
        .send().await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("请求失败: {}", e)))?;

    let html = resp.text().await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("读取响应失败: {}", e)))?;

    // 简单去 HTML 标签
    let text = strip_html(&html);
    let truncated = truncate_str(&text, 10000);
    Ok(truncated)
}

/// 去除 HTML 标签，保留纯文本
fn strip_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        if c == '<' { in_tag = true; continue; }
        if c == '>' { in_tag = false; continue; }
        if !in_tag { result.push(c); }
    }
    // 压缩多余空白
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 按字符安全截断
fn truncate_str(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max_chars).collect();
    format!("{}...\n\n(内容过长，已截断至前 {} 字符)", truncated, max_chars)
}

/// 判断 URL 是否指向本服务（用于 api_request 安全注入 token）
fn is_local_service_url(url: &str, config_host: &str, config_port: u16) -> bool {
    // 相对路径始终算本站
    if url.starts_with('/') {
        return true;
    }
    // 匹配本机可能的访问方式
    let port_str = config_port.to_string();
    [
        format!("http://{}:{}", config_host, port_str),
        format!("https://{}:{}", config_host, port_str),
        format!("http://127.0.0.1:{}", port_str),
        format!("http://localhost:{}", port_str),
    ]
    .iter()
    .any(|prefix| url.starts_with(prefix))
}

/// 包裹一个内置工具，仅覆盖 description() 和 parameters() 为 DB 中的值。
/// execute() 仍按原工具执行。
struct DbOverrideTool {
    inner: Arc<dyn AiTool>,
    item: ai_tool::Model,
}

#[async_trait]
impl AiTool for DbOverrideTool {
    fn name(&self) -> &str { self.inner.name() }
    fn description(&self) -> &str { &self.item.description }
    fn parameters(&self) -> Value {
        serde_json::from_str(&self.item.parameters_schema)
            .unwrap_or_else(|_| self.inner.parameters())
    }

    async fn execute(&self, args: Value, state: &AppState) -> Result<String, AppError> {
        self.inner.execute(args, state).await
    }
}

struct DbTool {
    item: ai_tool::Model,
}

#[async_trait]
impl AiTool for DbTool {
    fn name(&self) -> &str { &self.item.function_name }
    fn description(&self) -> &str { &self.item.description }

    fn parameters(&self) -> Value {
        serde_json::from_str(&self.item.parameters_schema).unwrap_or(serde_json::json!({
            "type": "object", "properties": {}
        }))
    }

    async fn execute(&self, _args: Value, _state: &AppState) -> Result<String, AppError> {
        Ok(format!("工具「{}」已收到调用，但此工具为声明式定义，暂无执行逻辑。", self.item.name))
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  Factory: 创建带有内置工具 + DB 工具的 ToolRegistry
// ═══════════════════════════════════════════════════════════════════════

use crate::models::entity::ai_tool;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;

/// 用户上下文：传递给需要登录身份的工具（如 api_request）
#[derive(Clone)]
pub struct UserContext {
    pub token: String,
}

/// 动态加载工具：内置工具 + 数据库中 enabled=true 的自定义工具
/// is_privileged: 管理员可调用全部工具，非管理员只能调用只读工具
/// user: 当前登录用户上下文（api_request 等工具需要）
///
/// 规则：
/// 1. DB 中 enabled=false 的内置工具不会被注册（即使有 Rust 实现）
/// 2. DB 中 enabled=true 的内置工具，用 DB 描述/参数覆盖 Rust 默认值
/// 3. DB 中有记录但无 Rust 实现 → 注册为声明式 DbTool
/// 4. 无 DB 记录的内置工具 → 用 Rust 默认值注册
pub async fn create_registry(
    db: &DatabaseConnection,
    is_privileged: bool,
    user: Option<&UserContext>,
) -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    // 加载所有 DB 工具（含 disabled），按 function_name 索引
    let all_db: Vec<_> = ai_tool::Entity::find()
        .all(db).await
        .unwrap_or_default();
    let db_map: HashMap<String, &ai_tool::Model> = all_db.iter()
        .map(|t| (t.function_name.clone(), t))
        .collect();

    // 辅助函数：注册内置工具（受 DB enabled + 权限控制）
    fn try_register(
        reg: &mut ToolRegistry,
        db_map: &HashMap<String, &ai_tool::Model>,
        name: &str,
        tool: impl AiTool + 'static,
        need_privilege: bool,
        is_privileged: bool,
    ) {
        if need_privilege && !is_privileged { return; }
        // DB 中有记录且 enabled=false → 跳过
        if let Some(item) = db_map.get(name) {
            if !item.enabled { return; }
        }
        // 无 DB 记录或用 DB 描述/参数覆盖
        let t: Arc<dyn AiTool> = if let Some(item) = db_map.get(name) {
            Arc::new(DbOverrideTool { inner: Arc::new(tool), item: (*item).clone() })
        } else {
            Arc::new(tool)
        };
        reg.register(t);
    }

    // 基础工具（所有用户可用）
    try_register(&mut registry, &db_map, "web_search", WebSearchTool, false, is_privileged);
    try_register(&mut registry, &db_map, "web_extract", WebExtractTool, false, is_privileged);
    try_register(&mut registry, &db_map, "get_current_datetime", GetCurrentDatetimeTool, false, is_privileged);

    // api_request — 需要携带用户 token 以用户身份调用
    if let Some(item) = db_map.get("api_request") {
        if !item.enabled { /* DB 禁用，跳过 */ }
        else {
            let tool = ApiRequestTool { user_token: user.map(|u| u.token.clone()) };
            let t: Arc<dyn AiTool> = Arc::new(DbOverrideTool { inner: Arc::new(tool), item: (*item).clone() });
            registry.register(t);
        }
    } else {
        let tool = ApiRequestTool { user_token: user.map(|u| u.token.clone()) };
        registry.register(Arc::new(tool));
    }

    // 特权工具（仅 admin/sub_admin）
    try_register(&mut registry, &db_map, "create_news", CreateNewsTool, true, is_privileged);
    try_register(&mut registry, &db_map, "create_post", CreatePostTool, true, is_privileged);

    // 加载 DB 中的纯自定义工具（不被内置工具覆盖的，且 enabled=true）
    for item in all_db {
        if item.enabled && !registry.contains(item.function_name.as_str()) {
            registry.register(Arc::new(DbTool { item }));
        }
    }

    registry
}
