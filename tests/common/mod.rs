#![allow(dead_code)] // Each integration-test crate uses a different subset of this shared fixture.

use std::net::SocketAddr;

use axum_test::TestServer;
use marksharex::{
    build_router,
    config::{AppConfig, AuthConfig, DatabaseConfig, ServerConfig, StorageConfig},
    migrations, models,
    models::entity::{settings, users},
    services,
    utils::AppState,
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, NotSet, Set};
use tempfile::TempDir;

pub struct TestUser {
    pub id: i32,
    pub token: String,
}

pub struct TestApp {
    pub server: TestServer,
    pub db: DatabaseConnection,
    config: AppConfig,
    _temp: TempDir,
}

impl TestApp {
    pub async fn new() -> anyhow::Result<Self> {
        let temp = tempfile::tempdir()?;
        let data_dir = temp.path().join("data");
        let upload_dir = temp.path().join("uploads");
        std::fs::create_dir_all(data_dir.join("templates/default"))?;
        std::fs::create_dir_all(&upload_dir)?;

        let database_path = temp.path().join("marksharex-test.db");
        let config = AppConfig {
            data_dir: data_dir.to_string_lossy().into_owned(),
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 0,
            },
            database: DatabaseConfig {
                url: format!("sqlite://{}?mode=rwc", database_path.display()),
                max_connections: 1,
                min_connections: 1,
            },
            auth: AuthConfig {
                jwt_secret: "integration-test-only-secret".to_string(),
                jwt_expire_seconds: 300,
                refresh_expire_seconds: 600,
                encrypt_key: String::new(),
            },
            storage: StorageConfig {
                upload_dir: upload_dir.to_string_lossy().into_owned(),
                max_file_size: 1024 * 1024,
                allowed_types: Vec::new(),
            },
            ai: None,
        };

        let db = models::init_db(&config.database).await?;
        models::run_migrations(&db).await?;
        migrations::run(&db).await?;

        let search_engine = services::search::init_index(&config.data_dir)?;
        let log_buffer = services::logs::LogBuffer::new(100);
        let state = AppState::new(db.clone(), config.clone(), search_engine, log_buffer);
        let app = build_router(state);
        let server = TestServer::new(app.into_make_service_with_connect_info::<SocketAddr>())?;

        Ok(Self {
            server,
            db,
            config,
            _temp: temp,
        })
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> anyhow::Result<()> {
        let now = marksharex::utils::now_local();
        if let Some(setting) = settings::Entity::find_by_id(key).one(&self.db).await? {
            let mut active = setting.into_active_model();
            active.value = Set(value.to_string());
            active.updated_at = Set(now);
            active.update(&self.db).await?;
        } else {
            settings::ActiveModel {
                key: Set(key.to_string()),
                value: Set(value.to_string()),
                updated_at: Set(now),
            }
            .insert(&self.db)
            .await?;
        }
        Ok(())
    }

    pub async fn create_user(&self, username: &str, role: &str) -> anyhow::Result<TestUser> {
        let now = marksharex::utils::now_local();
        let user = users::ActiveModel {
            id: NotSet,
            username: Set(username.to_string()),
            email: Set(format!("{username}@example.test")),
            password_hash: Set("not-used-by-integration-tests".to_string()),
            display_name: Set(Some(username.to_string())),
            avatar_url: Set(None),
            role: Set(role.to_string()),
            bio: Set(None),
            title: Set(None),
            is_active: Set(true),
            status: Set("active".to_string()),
            api_key: Set(None),
            last_login_at: Set(None),
            deleted_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(&self.db)
        .await?;

        let token = services::auth::generate_token(
            user.id,
            &user.username,
            user.display_name.clone(),
            &user.role,
            &user.status,
            &self.config.auth,
        )?;

        Ok(TestUser { id: user.id, token })
    }
}
