#![allow(dead_code)] // Each integration-test crate uses a different subset of this shared fixture.

use std::net::SocketAddr;

use axum_test::TestServer;
use marksharex::{
    build_router,
    config::{AiConfig, AppConfig, AuthConfig, DatabaseConfig, ServerConfig, StorageConfig},
    migrations, models,
    models::entity::{
        ai_chat_session, categories, comments, files, posts, read_logs, settings, users,
    },
    services,
    utils::AppState,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, NotSet, PaginatorTrait, QueryFilter, Set, Statement,
};
use serde_json::{json, Value};
use tempfile::TempDir;

pub struct TestUser {
    pub id: i32,
    pub token: String,
}

pub struct TestPost {
    pub id: i32,
    pub title: String,
    pub slug: String,
}

pub struct PostChildCounts {
    pub likes: u64,
    pub comments: u64,
    pub read_logs: u64,
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
                allowed_types: vec!["image/png".to_string(), "image/jpeg".to_string()],
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

    pub async fn new_with_ai(allowed_provider_networks: Vec<String>) -> anyhow::Result<Self> {
        let temp = tempfile::tempdir()?;
        let data_dir = temp.path().join("data");
        let upload_dir = temp.path().join("uploads");
        std::fs::create_dir_all(data_dir.join("templates/default"))?;
        std::fs::create_dir_all(&upload_dir)?;

        let database_path = temp.path().join("marksharex-test.db");
        let config = AppConfig {
            data_dir: data_dir.to_string_lossy().into_owned(),
            server: ServerConfig { host: "127.0.0.1".to_string(), port: 0 },
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
                allowed_types: vec!["image/png".to_string(), "image/jpeg".to_string()],
            },
            ai: Some(AiConfig {
                max_tool_rounds: 8,
                search: None,
                allowed_provider_networks,
            }),
        };

        let db = models::init_db(&config.database).await?;
        models::run_migrations(&db).await?;
        migrations::run(&db).await?;

        let search_engine = services::search::init_index(&config.data_dir)?;
        let log_buffer = services::logs::LogBuffer::new(100);
        let state = AppState::new(db.clone(), config.clone(), search_engine, log_buffer);
        let app = build_router(state);
        let server = TestServer::new(app.into_make_service_with_connect_info::<SocketAddr>())?;

        Ok(Self { server, db, config, _temp: temp })
    }

    pub fn upload_dir(&self) -> &std::path::Path {
        std::path::Path::new(&self.config.storage.upload_dir)
    }

    pub fn temp_root(&self) -> &std::path::Path {
        self._temp.path()
    }

    pub async fn file_count(&self) -> anyhow::Result<u64> {
        Ok(files::Entity::find().count(&self.db).await?)
    }

    pub async fn category_count(&self) -> anyhow::Result<u64> {
        Ok(categories::Entity::find().count(&self.db).await?)
    }

    pub async fn insert_file_record(
        &self,
        user_id: i32,
        filename: &str,
        mime_type: &str,
    ) -> anyhow::Result<i32> {
        let now = marksharex::utils::now_local();
        let file = files::ActiveModel {
            user_id: Set(user_id),
            filename: Set(filename.to_string()),
            original_name: Set(filename.to_string()),
            mime_type: Set(mime_type.to_string()),
            size: Set(1),
            storage_path: Set(None),
            url: Set(None),
            md5_hash: Set(None),
            deleted_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&self.db)
        .await?;
        Ok(file.id)
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

    pub async fn create_post(
        &self,
        user: &TestUser,
        title: &str,
        status: &str,
    ) -> anyhow::Result<TestPost> {
        let response = self
            .server
            .post("/api/v1/posts")
            .authorization_bearer(&user.token)
            .json(&json!({"title": title, "status": status}))
            .await;
        response.assert_status_ok();
        let body = response.json::<Value>();
        Ok(TestPost {
            id: body["data"]["id"]
                .as_i64()
                .expect("post response should contain an integer id") as i32,
            title: body["data"]["title"]
                .as_str()
                .expect("post response should contain a title")
                .to_string(),
            slug: body["data"]["slug"]
                .as_str()
                .expect("post response should contain a slug")
                .to_string(),
        })
    }

    pub async fn db_counts_for_post(&self, post_id: i32) -> anyhow::Result<PostChildCounts> {
        let likes = self
            .db
            .query_one(Statement::from_sql_and_values(
                self.db.get_database_backend(),
                "SELECT COUNT(*) FROM likes WHERE post_id = ?",
                [post_id.into()],
            ))
            .await?
            .expect("COUNT query should return one row")
            .try_get_by_index::<i64>(0)? as u64;
        let comments = comments::Entity::find()
            .filter(comments::Column::PostId.eq(post_id))
            .count(&self.db)
            .await?;
        let read_logs = read_logs::Entity::find()
            .filter(read_logs::Column::PostId.eq(post_id))
            .count(&self.db)
            .await?;
        Ok(PostChildCounts {
            likes,
            comments,
            read_logs,
        })
    }

    pub async fn post_count(&self) -> anyhow::Result<u64> {
        Ok(posts::Entity::find().count(&self.db).await?)
    }

    pub async fn get_post_row(&self, post_id: i32) -> anyhow::Result<posts::Model> {
        posts::Entity::find_by_id(post_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("post {post_id} should exist in the test database"))
    }

    pub async fn set_user_role(&self, user_id: i32, role: &str) -> anyhow::Result<()> {
        let user = users::Entity::find_by_id(user_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("user {user_id} should exist"))?;
        let mut active = user.into_active_model();
        active.role = Set(role.to_string());
        active.updated_at = Set(marksharex::utils::now_local());
        active.update(&self.db).await?;
        Ok(())
    }

    pub async fn ai_session_exists(&self, session_id: i32) -> anyhow::Result<bool> {
        Ok(ai_chat_session::Entity::find_by_id(session_id)
            .one(&self.db)
            .await?
            .is_some())
    }

    pub async fn set_user_api_key(
        &self,
        user_id: i32,
        api_key: Option<&str>,
    ) -> anyhow::Result<()> {
        let user = users::Entity::find_by_id(user_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("user {user_id} should exist"))?;
        let mut active = user.into_active_model();
        active.api_key = Set(api_key.map(str::to_string));
        active.updated_at = Set(marksharex::utils::now_local());
        active.update(&self.db).await?;
        Ok(())
    }

    pub async fn set_user_status(&self, user_id: i32, status: &str) -> anyhow::Result<()> {
        let user = users::Entity::find_by_id(user_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("user {user_id} should exist"))?;
        let mut active = user.into_active_model();
        active.status = Set(status.to_string());
        active.updated_at = Set(marksharex::utils::now_local());
        active.update(&self.db).await?;
        Ok(())
    }

    pub async fn set_user_active(&self, user_id: i32, is_active: bool) -> anyhow::Result<()> {
        let user = users::Entity::find_by_id(user_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("user {user_id} should exist"))?;
        let mut active = user.into_active_model();
        active.is_active = Set(is_active);
        active.updated_at = Set(marksharex::utils::now_local());
        active.update(&self.db).await?;
        Ok(())
    }

    pub async fn set_user_deleted(&self, user_id: i32, deleted: bool) -> anyhow::Result<()> {
        let user = users::Entity::find_by_id(user_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("user {user_id} should exist"))?;
        let mut active = user.into_active_model();
        active.deleted_at = Set(deleted.then(marksharex::utils::now_local));
        active.updated_at = Set(marksharex::utils::now_local());
        active.update(&self.db).await?;
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
