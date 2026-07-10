use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub data_dir: String,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub storage: StorageConfig,
    pub ai: Option<AiConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiConfig {
    #[serde(default = "default_max_tool_rounds")]
    pub max_tool_rounds: u32,
    pub search: Option<AiSearchConfig>,
}

fn default_max_tool_rounds() -> u32 { 8 }

#[derive(Debug, Clone, Deserialize)]
pub struct AiSearchConfig {
    #[serde(default = "default_search_provider")]
    pub provider: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub api_url: String,
    /// 主提供商超额时自动降级的备选（默认 firecrawl）
    #[serde(default = "default_fallback_provider")]
    pub fallback_provider: String,
    /// 降级提供商的 API Key
    #[serde(default)]
    pub fallback_api_key: String,
}

fn default_search_provider() -> String { "tavily".to_string() }
fn default_fallback_provider() -> String { "firecrawl".to_string() }

impl AiSearchConfig {
    /// 返回降级链：[(provider, api_key), ...]，最后永远是 duckduckgo 兜底
    pub fn fallback_chain(&self) -> Vec<(&str, &str)> {
        let mut chain = vec![(self.provider.as_str(), self.api_key.as_str())];
        if !self.fallback_provider.is_empty() && self.fallback_provider != self.provider {
            chain.push((self.fallback_provider.as_str(), self.fallback_api_key.as_str()));
        }
        if self.provider != "duckduckgo" && self.fallback_provider != "duckduckgo" {
            chain.push(("duckduckgo", "")); // 终极兜底
        }
        chain
    }

}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expire_seconds: i64,
    pub refresh_expire_seconds: i64,
    #[serde(default)]
    pub encrypt_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub upload_dir: String,
    pub max_file_size: u64,
    pub allowed_types: Vec<String>,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let mut builder = config::Config::builder()
            .add_source(config::File::with_name("config"));
        
        // 手动覆盖环境变量
        if let Ok(data_dir) = std::env::var("MARKSHAREX_DATA_DIR") {
            builder = builder.set_override("data_dir", data_dir.clone())?;
            #[cfg(debug_assertions)]
            println!("  ✅ 环境变量覆盖: data_dir = {}", data_dir);
        }
        
        if let Ok(upload_dir) = std::env::var("MARKSHAREX_STORAGE_UPLOAD_DIR") {
            builder = builder.set_override("storage.upload_dir", upload_dir.clone())?;
            #[cfg(debug_assertions)]
            println!("  ✅ 环境变量覆盖: storage.upload_dir = {}", upload_dir);
        }
        
        if let Ok(host) = std::env::var("MARKSHAREX_SERVER_HOST") {
            builder = builder.set_override("server.host", host.clone())?;
            #[cfg(debug_assertions)]
            println!("  ✅ 环境变量覆盖: server.host = {}", host);
        }
        
        if let Ok(port) = std::env::var("MARKSHAREX_SERVER_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                builder = builder.set_override("server.port", port_num)?;
                #[cfg(debug_assertions)]
                println!("  ✅ 环境变量覆盖: server.port = {}", port_num);
            }
        }
        
        // 数据库相关环境变量
        if let Ok(db_url) = std::env::var("MARKSHAREX_DATABASE_URL") {
            builder = builder.set_override("database.url", db_url.clone())?;
            #[cfg(debug_assertions)]
            println!("  ✅ 环境变量覆盖: database.url = {}", db_url);
        }
        
        if let Ok(max_connections) = std::env::var("MARKSHAREX_DATABASE_MAX_CONNECTIONS") {
            if let Ok(max_conns) = max_connections.parse::<u32>() {
                builder = builder.set_override("database.max_connections", max_conns)?;
                #[cfg(debug_assertions)]
                println!("  ✅ 环境变量覆盖: database.max_connections = {}", max_conns);
            }
        }
        
        if let Ok(min_connections) = std::env::var("MARKSHAREX_DATABASE_MIN_CONNECTIONS") {
            if let Ok(min_conns) = min_connections.parse::<u32>() {
                builder = builder.set_override("database.min_connections", min_conns)?;
                #[cfg(debug_assertions)]
                println!("  ✅ 环境变量覆盖: database.min_connections = {}", min_conns);
            }
        }

        // AI 相关环境变量
        if let Ok(max_rounds) = std::env::var("MARKSHAREX_AI_MAX_TOOL_ROUNDS") {
            if let Ok(rounds) = max_rounds.parse::<u32>() {
                builder = builder.set_override("ai.max_tool_rounds", rounds)?;
                #[cfg(debug_assertions)]
                println!("  ✅ 环境变量覆盖: ai.max_tool_rounds = {}", rounds);
            }
        }
        
        let config = builder.build()?;
        
        // 打印最终配置（仅 debug 模式）
        #[cfg(debug_assertions)]
        {
            println!("📝 最终配置调试:");
            println!("  - data_dir: {:?}", config.get_string("data_dir").ok());
            println!("  - storage.upload_dir: {:?}", config.get_string("storage.upload_dir").ok());
        }
        
        Ok(config.try_deserialize()?)
    }
}
