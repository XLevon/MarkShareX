use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub data_dir: String,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub storage: StorageConfig,
    pub ai: Option<AiConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiConfig {
    #[serde(default = "default_max_tool_rounds")]
    pub max_tool_rounds: u32,
    pub search: Option<AiSearchConfig>,
    /// Explicit IP/CIDR allowlist that providers may connect to despite
    /// being private addresses. Useful for local Ollama instances.
    /// Example: ["192.168.1.100", "10.0.0.0/24"]. `0.0.0.0` is not a wildcard.
    #[serde(default)]
    pub allowed_provider_networks: Vec<String>,
}

fn default_max_tool_rounds() -> u32 {
    8
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiSearchConfig {
    #[serde(default = "default_search_provider")]
    pub provider: String,
    #[serde(default)]
    pub api_key: String,
    /// 主提供商超额时自动降级的备选（默认 firecrawl）
    #[serde(default = "default_fallback_provider")]
    pub fallback_provider: String,
    /// 降级提供商的 API Key
    #[serde(default)]
    pub fallback_api_key: String,
    /// SearXNG 自托管搜索地址（留空则不加入降级链）
    #[serde(default)]
    pub searxng_url: String,
    /// DuckDuckGo 搜索地址（默认 lite.duckduckgo.com，可改为代理地址）
    #[serde(default = "default_duckduckgo_url")]
    pub duckduckgo_url: String,
    /// 仅供配置的 SearXNG/DuckDuckGo 搜索服务使用的明确内网 IP/CIDR。
    /// 不应用于用户提供的 web_extract URL，也不复用 provider allowlist。
    #[serde(default)]
    pub allowed_search_networks: Vec<String>,
}

fn default_search_provider() -> String {
    "tavily".to_string()
}
fn default_fallback_provider() -> String {
    "firecrawl".to_string()
}
fn default_duckduckgo_url() -> String {
    "https://lite.duckduckgo.com/lite/".to_string()
}

impl AiSearchConfig {
    /// 返回降级链：[(provider, api_key), ...]，最后永远是 duckduckgo 兜底
    pub fn fallback_chain(&self) -> Vec<(&str, &str)> {
        let mut chain = vec![(self.provider.as_str(), self.api_key.as_str())];
        if !self.fallback_provider.is_empty() && self.fallback_provider != self.provider {
            chain.push((
                self.fallback_provider.as_str(),
                self.fallback_api_key.as_str(),
            ));
        }
        // SearXNG 自托管搜索（配置了才加入降级链）
        if !self.searxng_url.is_empty() {
            chain.push(("searxng", ""));
        }
        if self.provider != "duckduckgo" && self.fallback_provider != "duckduckgo" {
            chain.push(("duckduckgo", "")); // 终极兜底
        }
        chain
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub trusted_proxies: Vec<String>,
    /// Exact cross-origin browser origins allowed to call the API.
    /// Empty means no cross-origin access; same-origin requests do not need CORS headers.
    #[serde(default)]
    pub cors_allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expire_seconds: i64,
    pub refresh_expire_seconds: i64,
    #[serde(default)]
    pub encrypt_key: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StorageConfig {
    pub upload_dir: String,
    pub max_file_size: u64,
    pub allowed_types: Vec<String>,
}

#[derive(Clone, Copy)]
enum EnvironmentValueKind {
    String,
    U16,
    U32,
    U64,
    I64,
    List,
}

#[derive(Clone, Copy)]
struct EnvironmentBinding {
    name: &'static str,
    path: &'static str,
    kind: EnvironmentValueKind,
}

const ENVIRONMENT_BINDINGS: &[EnvironmentBinding] = &[
    EnvironmentBinding {
        name: "MARKSHAREX_DATA_DIR",
        path: "data_dir",
        kind: EnvironmentValueKind::String,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_SERVER_HOST",
        path: "server.host",
        kind: EnvironmentValueKind::String,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_SERVER_PORT",
        path: "server.port",
        kind: EnvironmentValueKind::U16,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_SERVER_TRUSTED_PROXIES",
        path: "server.trusted_proxies",
        kind: EnvironmentValueKind::List,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_SERVER_CORS_ALLOWED_ORIGINS",
        path: "server.cors_allowed_origins",
        kind: EnvironmentValueKind::List,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_DATABASE_URL",
        path: "database.url",
        kind: EnvironmentValueKind::String,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_DATABASE_MAX_CONNECTIONS",
        path: "database.max_connections",
        kind: EnvironmentValueKind::U32,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_DATABASE_MIN_CONNECTIONS",
        path: "database.min_connections",
        kind: EnvironmentValueKind::U32,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_AUTH_JWT_SECRET",
        path: "auth.jwt_secret",
        kind: EnvironmentValueKind::String,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_AUTH_JWT_EXPIRE_SECONDS",
        path: "auth.jwt_expire_seconds",
        kind: EnvironmentValueKind::I64,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_AUTH_REFRESH_EXPIRE_SECONDS",
        path: "auth.refresh_expire_seconds",
        kind: EnvironmentValueKind::I64,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_AUTH_ENCRYPT_KEY",
        path: "auth.encrypt_key",
        kind: EnvironmentValueKind::String,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_STORAGE_UPLOAD_DIR",
        path: "storage.upload_dir",
        kind: EnvironmentValueKind::String,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_STORAGE_MAX_FILE_SIZE",
        path: "storage.max_file_size",
        kind: EnvironmentValueKind::U64,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_STORAGE_ALLOWED_TYPES",
        path: "storage.allowed_types",
        kind: EnvironmentValueKind::List,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_AI_MAX_TOOL_ROUNDS",
        path: "ai.max_tool_rounds",
        kind: EnvironmentValueKind::U32,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_AI_ALLOWED_PROVIDER_NETWORKS",
        path: "ai.allowed_provider_networks",
        kind: EnvironmentValueKind::List,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_AI_SEARCH_PROVIDER",
        path: "ai.search.provider",
        kind: EnvironmentValueKind::String,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_AI_SEARCH_API_KEY",
        path: "ai.search.api_key",
        kind: EnvironmentValueKind::String,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_AI_SEARCH_FALLBACK_PROVIDER",
        path: "ai.search.fallback_provider",
        kind: EnvironmentValueKind::String,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_AI_SEARCH_FALLBACK_API_KEY",
        path: "ai.search.fallback_api_key",
        kind: EnvironmentValueKind::String,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_AI_SEARCH_SEARXNG_URL",
        path: "ai.search.searxng_url",
        kind: EnvironmentValueKind::String,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_AI_SEARCH_DUCKDUCKGO_URL",
        path: "ai.search.duckduckgo_url",
        kind: EnvironmentValueKind::String,
    },
    EnvironmentBinding {
        name: "MARKSHAREX_AI_SEARCH_ALLOWED_SEARCH_NETWORKS",
        path: "ai.search.allowed_search_networks",
        kind: EnvironmentValueKind::List,
    },
];

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let base = config::Config::builder()
            .add_source(config::File::with_name("config"))
            .build()?;
        Self::load_from_config_with_env(base, |name| std::env::var(name).ok())
    }

    #[cfg(test)]
    fn load_from_file_with_env<F>(path: &std::path::Path, get_env: F) -> anyhow::Result<Self>
    where
        F: Fn(&str) -> Option<String>,
    {
        let base = config::Config::builder()
            .add_source(config::File::from(path.to_path_buf()))
            .build()?;
        Self::load_from_config_with_env(base, get_env)
    }

    fn load_from_config_with_env<F>(base: config::Config, get_env: F) -> anyhow::Result<Self>
    where
        F: Fn(&str) -> Option<String>,
    {
        fn parse<T>(name: &str, raw: &str) -> anyhow::Result<T>
        where
            T: std::str::FromStr,
            T::Err: std::fmt::Display,
        {
            raw.parse::<T>()
                .map_err(|error| anyhow::anyhow!("invalid value for {name}: {raw:?}: {error}"))
        }

        fn list(raw: &str) -> Vec<String> {
            raw.split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect()
        }

        let mut builder = config::Config::builder().add_source(base);
        let mut canonical_encrypt_key_was_set = false;

        for binding in ENVIRONMENT_BINDINGS {
            let Some(raw) = get_env(binding.name) else {
                continue;
            };
            if binding.name == "MARKSHAREX_AUTH_ENCRYPT_KEY" {
                canonical_encrypt_key_was_set = true;
            }
            builder = match binding.kind {
                EnvironmentValueKind::String => builder.set_override(binding.path, raw)?,
                EnvironmentValueKind::U16 => {
                    builder.set_override(binding.path, parse::<u16>(binding.name, &raw)?)?
                }
                EnvironmentValueKind::U32 => {
                    builder.set_override(binding.path, parse::<u32>(binding.name, &raw)?)?
                }
                EnvironmentValueKind::U64 => {
                    builder.set_override(binding.path, parse::<u64>(binding.name, &raw)?)?
                }
                EnvironmentValueKind::I64 => {
                    builder.set_override(binding.path, parse::<i64>(binding.name, &raw)?)?
                }
                EnvironmentValueKind::List => builder.set_override(binding.path, list(&raw))?,
            };
        }

        // Compatibility only: the canonical nested name always wins, including when it is empty.
        if !canonical_encrypt_key_was_set {
            if let Some(raw) = get_env("MARKSHAREX_ENCRYPT_KEY") {
                builder = builder.set_override("auth.encrypt_key", raw)?;
            }
        }

        let config = builder.build()?;
        let config: Self = config.try_deserialize()?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> anyhow::Result<()> {
        fn valid_http_url(path: &str, value: &str) -> anyhow::Result<()> {
            if value.is_empty() {
                return Ok(());
            }
            let parsed = url::Url::parse(value)
                .map_err(|error| anyhow::anyhow!("{path} must be a valid HTTP(S) URL: {error}"))?;
            anyhow::ensure!(
                matches!(parsed.scheme(), "http" | "https")
                    && parsed.host_str().is_some()
                    && parsed.username().is_empty()
                    && parsed.password().is_none()
                    && parsed.query().is_none()
                    && parsed.fragment().is_none(),
                "{path} must be an HTTP(S) URL without credentials, query, or fragment"
            );
            Ok(())
        }

        fn valid_origin(value: &str) -> bool {
            let Ok(parsed) = url::Url::parse(value) else {
                return false;
            };
            let has_only_authority = value
                .split_once("://")
                .is_some_and(|(_, authority)| !authority.is_empty() && !authority.contains('/'));
            matches!(parsed.scheme(), "http" | "https")
                && parsed.host_str().is_some()
                && parsed.username().is_empty()
                && parsed.password().is_none()
                && has_only_authority
                && parsed.path() == "/"
                && parsed.query().is_none()
                && parsed.fragment().is_none()
        }

        anyhow::ensure!(self.server.port > 0, "server.port must be greater than 0");
        for proxy in &self.server.trusted_proxies {
            anyhow::ensure!(
                proxy.parse::<std::net::IpAddr>().is_ok(),
                "server.trusted_proxies contains invalid exact IP: {proxy:?}"
            );
        }
        for origin in &self.server.cors_allowed_origins {
            anyhow::ensure!(
                valid_origin(origin),
                "server.cors_allowed_origins contains invalid exact HTTP(S) origin: {origin:?}"
            );
        }
        anyhow::ensure!(
            self.database.max_connections > 0,
            "database.max_connections must be greater than 0"
        );
        anyhow::ensure!(
            self.database.min_connections <= self.database.max_connections,
            "database.min_connections must not exceed database.max_connections"
        );
        anyhow::ensure!(
            !self.auth.jwt_secret.trim().is_empty(),
            "auth.jwt_secret must not be empty or whitespace"
        );
        anyhow::ensure!(
            self.auth.jwt_expire_seconds > 0,
            "auth.jwt_expire_seconds must be greater than 0"
        );
        anyhow::ensure!(
            self.auth.refresh_expire_seconds > 0,
            "auth.refresh_expire_seconds must be greater than 0"
        );
        anyhow::ensure!(
            !self.auth.encrypt_key.trim().is_empty(),
            "auth.encrypt_key must not be empty or whitespace"
        );
        anyhow::ensure!(
            self.storage.max_file_size > 0,
            "storage.max_file_size must be greater than 0"
        );
        if let Some(ai) = &self.ai {
            anyhow::ensure!(
                ai.max_tool_rounds > 0,
                "ai.max_tool_rounds must be greater than 0"
            );
            for network in &ai.allowed_provider_networks {
                anyhow::ensure!(
                    crate::utils::ip_utils::is_valid_ip(network),
                    "ai.allowed_provider_networks contains invalid IP/CIDR: {network:?}"
                );
            }
            if let Some(search) = &ai.search {
                const PROVIDERS: &[&str] = &["tavily", "firecrawl", "searxng", "duckduckgo"];
                anyhow::ensure!(
                    PROVIDERS.contains(&search.provider.as_str()),
                    "ai.search.provider is unsupported: {:?}",
                    search.provider
                );
                anyhow::ensure!(
                    search.fallback_provider.is_empty()
                        || PROVIDERS.contains(&search.fallback_provider.as_str()),
                    "ai.search.fallback_provider is unsupported: {:?}",
                    search.fallback_provider
                );
                for network in &search.allowed_search_networks {
                    anyhow::ensure!(
                        crate::utils::ip_utils::is_valid_ip(network),
                        "ai.search.allowed_search_networks contains invalid IP/CIDR: {network:?}"
                    );
                }
                valid_http_url("ai.search.searxng_url", &search.searxng_url)?;
                valid_http_url("ai.search.duckduckgo_url", &search.duckduckgo_url)?;
                if search.provider == "searxng" || search.fallback_provider == "searxng" {
                    anyhow::ensure!(
                        !search.searxng_url.is_empty(),
                        "ai.search.searxng_url must be set when SearXNG is selected"
                    );
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn example_config_deserializes_without_fixed_secrets() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.example.toml");
        let config: AppConfig = config::Config::builder()
            .add_source(config::File::from(path))
            .build()
            .expect("example config should parse")
            .try_deserialize()
            .expect("example config should match AppConfig");

        assert!(
            config.auth.jwt_secret.is_empty(),
            "the public example must not ship a reusable JWT secret"
        );
        assert!(
            config.auth.encrypt_key.is_empty(),
            "the public example must not ship a reusable encryption secret"
        );
    }

    #[test]
    fn startup_script_does_not_reference_removed_server_base_url() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/start.sh");
        let script = std::fs::read_to_string(path).expect("start script should be readable");
        assert!(!script.contains("MARKSHAREX_SERVER_BASE_URL"));
        assert!(script.contains("MARKSHAREX_AUTH_JWT_SECRET"));
        assert!(script.contains("MARKSHAREX_AUTH_ENCRYPT_KEY"));
        assert!(script.contains("MARKSHAREX_INIT_ENV_ONLY"));
    }

    #[test]
    fn startup_script_generates_and_preserves_random_secrets_before_docker() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let temp = tempfile::tempdir().expect("temporary project directory should be created");
        std::fs::copy(root.join(".env.example"), temp.path().join(".env.example"))
            .expect("env example should be copied");

        let run_initializer = || {
            let output = std::process::Command::new("bash")
                .arg(root.join("scripts/start.sh"))
                .env("MARKSHAREX_PROJECT_DIR", temp.path())
                .env("MARKSHAREX_INIT_ENV_ONLY", "1")
                .output()
                .expect("start script should execute");
            assert!(
                output.status.success(),
                "initializer failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        };

        let read_secret = |name: &str| {
            let env = std::fs::read_to_string(temp.path().join(".env"))
                .expect("generated .env should be readable");
            env.lines()
                .find_map(|line| line.strip_prefix(&format!("{name}=")))
                .map(str::to_string)
                .expect("generated secret should exist")
        };

        run_initializer();
        let jwt = read_secret("MARKSHAREX_AUTH_JWT_SECRET");
        let encryption = read_secret("MARKSHAREX_AUTH_ENCRYPT_KEY");
        for secret in [&jwt, &encryption] {
            assert_eq!(secret.len(), 64);
            assert!(secret.bytes().all(|byte| byte.is_ascii_hexdigit()));
        }
        assert_ne!(jwt, encryption);

        run_initializer();
        assert_eq!(read_secret("MARKSHAREX_AUTH_JWT_SECRET"), jwt);
        assert_eq!(read_secret("MARKSHAREX_AUTH_ENCRYPT_KEY"), encryption);

        std::fs::write(
            temp.path().join(".env"),
            "MARKSHAREX_AUTH_JWT_SECRET=   \nMARKSHAREX_AUTH_ENCRYPT_KEY=\t\n",
        )
        .expect("whitespace-only fixture should be written");
        run_initializer();
        let whitespace_jwt = read_secret("MARKSHAREX_AUTH_JWT_SECRET");
        let whitespace_encryption = read_secret("MARKSHAREX_AUTH_ENCRYPT_KEY");
        for secret in [&whitespace_jwt, &whitespace_encryption] {
            assert_eq!(secret.len(), 64);
            assert!(secret.bytes().all(|byte| byte.is_ascii_hexdigit()));
        }
        assert_ne!(whitespace_jwt, whitespace_encryption);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(temp.path().join(".env"))
                .expect("generated .env should have metadata")
                .permissions()
                .mode();
            assert_eq!(mode & 0o077, 0);
        }
    }

    #[cfg(unix)]
    #[test]
    fn base_only_startup_does_not_require_or_create_runtime_configuration() {
        use std::os::unix::fs::PermissionsExt;

        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let temp = tempfile::tempdir().expect("temporary project directory should be created");
        let bin = temp.path().join("bin");
        std::fs::create_dir(&bin).expect("fake bin directory should be created");
        let docker = bin.join("docker");
        std::fs::write(&docker, "#!/bin/sh\nexit 0\n").expect("fake docker should be written");
        std::fs::set_permissions(&docker, std::fs::Permissions::from_mode(0o755))
            .expect("fake docker should be executable");
        let path = format!(
            "{}:{}",
            bin.display(),
            std::env::var("PATH").unwrap_or_default()
        );

        let output = std::process::Command::new("bash")
            .arg(root.join("scripts/start.sh"))
            .arg("--base-only")
            .env("MARKSHAREX_PROJECT_DIR", temp.path())
            .env("PATH", path)
            .output()
            .expect("base-only startup should execute");
        assert!(
            output.status.success(),
            "base-only failed: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!temp.path().join(".env").exists());
    }

    #[test]
    fn docker_image_never_copies_the_local_secret_bearing_config() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Dockerfile");
        let dockerfile = std::fs::read_to_string(path).expect("Dockerfile should be readable");
        assert!(!dockerfile
            .lines()
            .any(|line| line.trim() == "COPY config.toml ./"));
        assert!(dockerfile
            .lines()
            .any(|line| line.trim() == "COPY config.example.toml ./config.toml"));

        let ignore_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".dockerignore");
        let dockerignore = std::fs::read_to_string(ignore_path)
            .expect(".dockerignore should protect local secrets");
        assert!(dockerignore.lines().any(|line| line.trim() == ".env*"));
        assert!(dockerignore
            .lines()
            .any(|line| line.trim() == "!.env.example"));
        assert!(dockerignore
            .lines()
            .any(|line| line.trim() == "config.toml"));
    }

    #[test]
    fn removed_or_misspelled_config_fields_fail_closed() {
        use std::io::Write;

        let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.example.toml");
        let source =
            std::fs::read_to_string(source_path).expect("example config should be readable");
        let source = source.replace(
            "port = 5023",
            "port = 5023\nbase_url = \"https://removed.example\"",
        );
        let mut file = tempfile::Builder::new()
            .suffix(".toml")
            .tempfile()
            .expect("temporary config should be created");
        file.write_all(source.as_bytes())
            .expect("temporary config should be written");

        let error = AppConfig::load_from_file_with_env(file.path(), |name| {
            let value = match name {
                "MARKSHAREX_AUTH_JWT_SECRET" => "jwt-test-secret",
                "MARKSHAREX_AUTH_ENCRYPT_KEY" => "encrypt-test-secret",
                _ => return None,
            };
            Some(value.to_string())
        })
        .expect_err("removed config fields must not be silently ignored");

        assert!(format!("{error:#}").contains("base_url"));

        let source = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("config.example.toml"),
        )
        .expect("example config should be readable")
        .replace(
            "api_key = \"\"",
            "api_key = \"\"\napi_url = \"https://dead.example\"",
        );
        let mut file = tempfile::Builder::new()
            .suffix(".toml")
            .tempfile()
            .expect("temporary config should be created");
        file.write_all(source.as_bytes())
            .expect("temporary config should be written");
        let error = AppConfig::load_from_file_with_env(file.path(), |name| {
            let value = match name {
                "MARKSHAREX_AUTH_JWT_SECRET" => "jwt-test-secret",
                "MARKSHAREX_AUTH_ENCRYPT_KEY" => "encrypt-test-secret",
                _ => return None,
            };
            Some(value.to_string())
        })
        .expect_err("dead search api_url must not remain a misleading configuration field");
        assert!(format!("{error:#}").contains("api_url"));
    }

    #[test]
    fn invalid_numeric_environment_override_is_rejected() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.example.toml");
        let cases = [
            ("MARKSHAREX_SERVER_PORT", "not-a-port"),
            ("MARKSHAREX_SERVER_PORT", "70000"),
            ("MARKSHAREX_DATABASE_MAX_CONNECTIONS", "-1"),
            ("MARKSHAREX_STORAGE_MAX_FILE_SIZE", "-1"),
            ("MARKSHAREX_AUTH_JWT_EXPIRE_SECONDS", " 3600"),
        ];

        for (name_under_test, invalid_value) in cases {
            let error = AppConfig::load_from_file_with_env(&path, |name| {
                (name == name_under_test).then(|| invalid_value.to_string())
            })
            .expect_err("invalid numeric overrides must fail instead of being ignored");

            let message = format!("{error:#}");
            assert!(message.contains(name_under_test));
            assert!(message.contains(invalid_value));
        }
    }

    #[test]
    fn every_supported_environment_variable_overrides_its_config_field() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.example.toml");
        let values = std::collections::HashMap::from([
            ("MARKSHAREX_DATA_DIR", "/tmp/marksharex-data"),
            ("MARKSHAREX_SERVER_HOST", "127.0.0.1"),
            ("MARKSHAREX_SERVER_PORT", "6123"),
            ("MARKSHAREX_SERVER_TRUSTED_PROXIES", "127.0.0.1,10.0.0.2"),
            (
                "MARKSHAREX_SERVER_CORS_ALLOWED_ORIGINS",
                "https://one.example,https://two.example",
            ),
            ("MARKSHAREX_DATABASE_URL", "sqlite:///tmp/config-test.db"),
            ("MARKSHAREX_DATABASE_MAX_CONNECTIONS", "17"),
            ("MARKSHAREX_DATABASE_MIN_CONNECTIONS", "3"),
            ("MARKSHAREX_AUTH_JWT_SECRET", "jwt-test-secret"),
            ("MARKSHAREX_AUTH_JWT_EXPIRE_SECONDS", "7200"),
            ("MARKSHAREX_AUTH_REFRESH_EXPIRE_SECONDS", "1209600"),
            ("MARKSHAREX_AUTH_ENCRYPT_KEY", "encrypt-test-secret"),
            ("MARKSHAREX_STORAGE_UPLOAD_DIR", "/tmp/marksharex-uploads"),
            ("MARKSHAREX_STORAGE_MAX_FILE_SIZE", "4096"),
            (
                "MARKSHAREX_STORAGE_ALLOWED_TYPES",
                "text/plain,application/pdf",
            ),
            ("MARKSHAREX_AI_MAX_TOOL_ROUNDS", "13"),
            (
                "MARKSHAREX_AI_ALLOWED_PROVIDER_NETWORKS",
                "192.168.1.5,10.0.0.0/24",
            ),
            ("MARKSHAREX_AI_SEARCH_PROVIDER", "firecrawl"),
            ("MARKSHAREX_AI_SEARCH_API_KEY", "search-primary"),
            ("MARKSHAREX_AI_SEARCH_FALLBACK_PROVIDER", "tavily"),
            ("MARKSHAREX_AI_SEARCH_FALLBACK_API_KEY", "search-fallback"),
            ("MARKSHAREX_AI_SEARCH_SEARXNG_URL", "https://searx.example"),
            (
                "MARKSHAREX_AI_SEARCH_DUCKDUCKGO_URL",
                "https://duck.example/lite/",
            ),
            (
                "MARKSHAREX_AI_SEARCH_ALLOWED_SEARCH_NETWORKS",
                "172.16.0.0/16,192.168.2.4",
            ),
        ]);

        let config = AppConfig::load_from_file_with_env(&path, |name| {
            values.get(name).map(|value| (*value).to_string())
        })
        .expect("all documented overrides should load");

        assert_eq!(config.data_dir, "/tmp/marksharex-data");
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 6123);
        assert_eq!(config.server.trusted_proxies, ["127.0.0.1", "10.0.0.2"]);
        assert_eq!(
            config.server.cors_allowed_origins,
            ["https://one.example", "https://two.example"]
        );
        assert_eq!(config.database.url, "sqlite:///tmp/config-test.db");
        assert_eq!(config.database.max_connections, 17);
        assert_eq!(config.database.min_connections, 3);
        assert_eq!(config.auth.jwt_secret, "jwt-test-secret");
        assert_eq!(config.auth.jwt_expire_seconds, 7200);
        assert_eq!(config.auth.refresh_expire_seconds, 1_209_600);
        assert_eq!(config.auth.encrypt_key, "encrypt-test-secret");
        assert_eq!(config.storage.upload_dir, "/tmp/marksharex-uploads");
        assert_eq!(config.storage.max_file_size, 4096);
        assert_eq!(
            config.storage.allowed_types,
            ["text/plain", "application/pdf"]
        );
        let ai = config.ai.expect("AI config should exist");
        assert_eq!(ai.max_tool_rounds, 13);
        assert_eq!(ai.allowed_provider_networks, ["192.168.1.5", "10.0.0.0/24"]);
        let search = ai.search.expect("AI search config should exist");
        assert_eq!(search.provider, "firecrawl");
        assert_eq!(search.api_key, "search-primary");

        assert_eq!(search.fallback_provider, "tavily");
        assert_eq!(search.fallback_api_key, "search-fallback");
        assert_eq!(search.searxng_url, "https://searx.example");
        assert_eq!(search.duckduckgo_url, "https://duck.example/lite/");
        assert_eq!(
            search.allowed_search_networks,
            ["172.16.0.0/16", "192.168.2.4"]
        );
        assert_eq!(values.len(), ENVIRONMENT_BINDINGS.len());
    }

    #[test]
    fn ai_environment_overrides_construct_optional_sections_when_toml_omits_them() {
        use std::io::Write;

        let source = r#"
data_dir = "./data"

[server]
host = "127.0.0.1"
port = 5023

[database]
url = "sqlite://./data/test.db?mode=rwc"
max_connections = 2
min_connections = 1

[auth]
jwt_secret = ""
jwt_expire_seconds = 3600
refresh_expire_seconds = 604800
encrypt_key = ""

[storage]
upload_dir = "./data/uploads"
max_file_size = 1024
allowed_types = ["text/plain"]
"#;
        let mut file = tempfile::Builder::new()
            .suffix(".toml")
            .tempfile()
            .expect("temporary config should be created");
        file.write_all(source.as_bytes())
            .expect("temporary config should be written");

        let config = AppConfig::load_from_file_with_env(file.path(), |name| {
            let value = match name {
                "MARKSHAREX_AUTH_JWT_SECRET" => "jwt-test-secret",
                "MARKSHAREX_AUTH_ENCRYPT_KEY" => "encrypt-test-secret",
                "MARKSHAREX_AI_SEARCH_PROVIDER" => "duckduckgo",
                "MARKSHAREX_AI_MAX_TOOL_ROUNDS" => "11",
                _ => return None,
            };
            Some(value.to_string())
        })
        .expect("environment overrides should create optional AI sections");

        let ai = config.ai.expect("AI config should be created");
        assert_eq!(ai.max_tool_rounds, 11);
        assert_eq!(
            ai.search
                .expect("AI search config should be created")
                .provider,
            "duckduckgo"
        );
    }

    #[test]
    fn env_example_exactly_matches_the_supported_environment_contract() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env.example");
        let env_example = std::fs::read_to_string(path).expect("env example should be readable");
        let documented = env_example
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                (!line.is_empty() && !line.starts_with('#'))
                    .then(|| line.split_once('=').map(|(name, _)| name))
                    .flatten()
            })
            .collect::<std::collections::BTreeSet<_>>();
        let supported = ENVIRONMENT_BINDINGS
            .iter()
            .map(|binding| binding.name)
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(documented, supported);
    }

    #[test]
    fn config_documentation_covers_the_complete_environment_contract() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/CONFIG.md");
        let documentation =
            std::fs::read_to_string(path).expect("docs/CONFIG.md should document configuration");

        for name in ENVIRONMENT_BINDINGS.iter().map(|binding| binding.name) {
            assert!(
                documentation.contains(name),
                "docs/CONFIG.md is missing {name}"
            );
        }
        for binding in ENVIRONMENT_BINDINGS {
            assert!(
                documentation.contains(&format!("`{}` | `{}`", binding.name, binding.path)),
                "docs/CONFIG.md has no exact mapping for {} -> {}",
                binding.name,
                binding.path
            );
        }
        assert!(documentation.contains("环境变量 > `config.toml`"));
        assert!(documentation.contains("MARKSHAREX_ENCRYPT_KEY"));
    }

    #[test]
    fn environment_bindings_are_unique_and_follow_the_nested_naming_contract() {
        let mut names = std::collections::BTreeSet::new();
        let mut paths = std::collections::BTreeSet::new();
        for binding in ENVIRONMENT_BINDINGS {
            assert!(
                names.insert(binding.name),
                "duplicate env name: {}",
                binding.name
            );
            assert!(
                paths.insert(binding.path),
                "duplicate config path: {}",
                binding.path
            );
            assert_eq!(
                binding.name,
                format!(
                    "MARKSHAREX_{}",
                    binding.path.replace('.', "_").to_uppercase()
                )
            );
        }
    }

    #[test]
    fn legacy_encrypt_key_alias_remains_compatible() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.example.toml");
        let config = AppConfig::load_from_file_with_env(&path, |name| {
            let value = match name {
                "MARKSHAREX_AUTH_JWT_SECRET" => "jwt-test-secret",
                "MARKSHAREX_ENCRYPT_KEY" => "legacy-encrypt-key",
                _ => return None,
            };
            Some(value.to_string())
        })
        .expect("legacy encryption key alias should remain supported");

        assert_eq!(config.auth.encrypt_key, "legacy-encrypt-key");
    }

    #[test]
    fn canonical_encrypt_key_name_wins_over_the_legacy_alias() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.example.toml");
        let config = AppConfig::load_from_file_with_env(&path, |name| {
            let value = match name {
                "MARKSHAREX_AUTH_JWT_SECRET" => "jwt-test-secret",
                "MARKSHAREX_AUTH_ENCRYPT_KEY" => "canonical-encrypt-key",
                "MARKSHAREX_ENCRYPT_KEY" => "legacy-encrypt-key",
                _ => return None,
            };
            Some(value.to_string())
        })
        .expect("canonical encryption key should win over the compatibility alias");

        assert_eq!(config.auth.encrypt_key, "canonical-encrypt-key");
    }

    #[test]
    fn required_authentication_secrets_fail_closed() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.example.toml");
        let missing_jwt = AppConfig::load_from_file_with_env(&path, |_| None)
            .expect_err("an empty JWT secret must be rejected");
        assert!(format!("{missing_jwt:#}").contains("auth.jwt_secret"));

        let missing_encrypt = AppConfig::load_from_file_with_env(&path, |name| {
            (name == "MARKSHAREX_AUTH_JWT_SECRET").then(|| "jwt-test-secret".to_string())
        })
        .expect_err("an empty encryption key must be rejected");
        assert!(format!("{missing_encrypt:#}").contains("auth.encrypt_key"));

        let blank_jwt = AppConfig::load_from_file_with_env(&path, |name| {
            let value = match name {
                "MARKSHAREX_AUTH_JWT_SECRET" => "   ",
                "MARKSHAREX_AUTH_ENCRYPT_KEY" => "encrypt-test-secret",
                _ => return None,
            };
            Some(value.to_string())
        })
        .expect_err("a whitespace-only JWT secret must be rejected");
        assert!(format!("{blank_jwt:#}").contains("auth.jwt_secret"));

        let blank_encrypt = AppConfig::load_from_file_with_env(&path, |name| {
            let value = match name {
                "MARKSHAREX_AUTH_JWT_SECRET" => "jwt-test-secret",
                "MARKSHAREX_AUTH_ENCRYPT_KEY" => "\t",
                _ => return None,
            };
            Some(value.to_string())
        })
        .expect_err("a whitespace-only encryption key must be rejected");
        assert!(format!("{blank_encrypt:#}").contains("auth.encrypt_key"));
    }

    #[test]
    fn invalid_numeric_relationships_and_zero_limits_are_rejected() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.example.toml");
        let cases = [
            ("MARKSHAREX_SERVER_PORT", "0", "server.port"),
            (
                "MARKSHAREX_DATABASE_MAX_CONNECTIONS",
                "0",
                "database.max_connections",
            ),
            (
                "MARKSHAREX_AUTH_JWT_EXPIRE_SECONDS",
                "0",
                "auth.jwt_expire_seconds",
            ),
            (
                "MARKSHAREX_AUTH_REFRESH_EXPIRE_SECONDS",
                "0",
                "auth.refresh_expire_seconds",
            ),
            (
                "MARKSHAREX_STORAGE_MAX_FILE_SIZE",
                "0",
                "storage.max_file_size",
            ),
            ("MARKSHAREX_AI_MAX_TOOL_ROUNDS", "0", "ai.max_tool_rounds"),
        ];

        for (invalid_name, invalid_value, expected_path) in cases {
            let error = AppConfig::load_from_file_with_env(&path, |name| {
                let value = match name {
                    "MARKSHAREX_AUTH_JWT_SECRET" => "jwt-test-secret",
                    "MARKSHAREX_AUTH_ENCRYPT_KEY" => "encrypt-test-secret",
                    _ if name == invalid_name => invalid_value,
                    _ => return None,
                };
                Some(value.to_string())
            })
            .expect_err("zero-valued runtime limits must fail");
            assert!(
                format!("{error:#}").contains(expected_path),
                "unexpected error for {invalid_name}: {error:#}"
            );
        }

        let error = AppConfig::load_from_file_with_env(&path, |name| {
            let value = match name {
                "MARKSHAREX_AUTH_JWT_SECRET" => "jwt-test-secret",
                "MARKSHAREX_AUTH_ENCRYPT_KEY" => "encrypt-test-secret",
                "MARKSHAREX_DATABASE_MIN_CONNECTIONS" => "20",
                "MARKSHAREX_DATABASE_MAX_CONNECTIONS" => "10",
                _ => return None,
            };
            Some(value.to_string())
        })
        .expect_err("min connections above max connections must fail");
        assert!(format!("{error:#}").contains("database.min_connections"));
    }

    #[test]
    fn invalid_structured_configuration_values_fail_closed() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.example.toml");
        let cases = [
            (
                "MARKSHAREX_SERVER_TRUSTED_PROXIES",
                "127.0.0.1,not-an-ip",
                "server.trusted_proxies",
            ),
            (
                "MARKSHAREX_SERVER_CORS_ALLOWED_ORIGINS",
                "https://admin.example/path",
                "server.cors_allowed_origins",
            ),
            (
                "MARKSHAREX_AI_ALLOWED_PROVIDER_NETWORKS",
                "10.0.0.0/99",
                "ai.allowed_provider_networks",
            ),
            (
                "MARKSHAREX_AI_SEARCH_ALLOWED_SEARCH_NETWORKS",
                "private-network",
                "ai.search.allowed_search_networks",
            ),
            (
                "MARKSHAREX_AI_SEARCH_PROVIDER",
                "unknown-provider",
                "ai.search.provider",
            ),
            (
                "MARKSHAREX_AI_SEARCH_FALLBACK_PROVIDER",
                "unknown-provider",
                "ai.search.fallback_provider",
            ),
            (
                "MARKSHAREX_AI_SEARCH_DUCKDUCKGO_URL",
                "file:///etc/passwd",
                "ai.search.duckduckgo_url",
            ),
        ];

        for (invalid_name, invalid_value, expected_path) in cases {
            let error = AppConfig::load_from_file_with_env(&path, |name| {
                let value = match name {
                    "MARKSHAREX_AUTH_JWT_SECRET" => "jwt-test-secret",
                    "MARKSHAREX_AUTH_ENCRYPT_KEY" => "encrypt-test-secret",
                    _ if name == invalid_name => invalid_value,
                    _ => return None,
                };
                Some(value.to_string())
            })
            .expect_err("invalid structured values must fail during configuration loading");
            assert!(
                format!("{error:#}").contains(expected_path),
                "unexpected error for {invalid_name}: {error:#}"
            );
        }

        let searxng_without_url = AppConfig::load_from_file_with_env(&path, |name| {
            let value = match name {
                "MARKSHAREX_AUTH_JWT_SECRET" => "jwt-test-secret",
                "MARKSHAREX_AUTH_ENCRYPT_KEY" => "encrypt-test-secret",
                "MARKSHAREX_AI_SEARCH_PROVIDER" => "searxng",
                _ => return None,
            };
            Some(value.to_string())
        })
        .expect_err("SearXNG selection without its URL must fail during configuration loading");
        assert!(format!("{searxng_without_url:#}").contains("ai.search.searxng_url"));
    }
}
