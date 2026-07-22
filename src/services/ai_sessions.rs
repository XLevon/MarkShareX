//! AI 会话查询与生命周期业务编排。
//!
//! HTTP Controller 只负责 extractor 和响应 DTO；数据库实时角色、owner scope、
//! 列表排序/计数、详情读取、授权删除以及斜杠命令编排统一保留在此处。

use crate::models::entity::{ai_chat_message, ai_chat_session, ai_model, ai_provider, users};
use crate::utils::{AppError, AppState};
use sea_orm::*;

pub(crate) struct SessionListItem {
    pub(crate) id: i32,
    pub(crate) title: String,
    pub(crate) user_id: i32,
    pub(crate) user_display_name: Option<String>,
    pub(crate) agent_config_id: Option<i32>,
    pub(crate) msg_count: usize,
    pub(crate) created_at: chrono::NaiveDateTime,
    pub(crate) updated_at: chrono::NaiveDateTime,
}

pub(crate) struct SessionDetailData {
    pub(crate) session: ai_chat_session::Model,
    pub(crate) messages: Vec<ai_chat_message::Model>,
}

pub(crate) struct SlashCommandResult {
    pub(crate) reply: String,
    pub(crate) session_id: i32,
}

async fn authorize_read_or_delete(
    state: &AppState,
    actor_user_id: i32,
    owner_id: i32,
) -> Result<(), AppError> {
    if actor_user_id == owner_id
        || crate::services::auth::current_active_role(state, actor_user_id).await? == "admin"
    {
        Ok(())
    } else {
        Err(AppError::NotFound("会话不存在".into()))
    }
}

pub(crate) async fn list_sessions(
    state: &AppState,
    actor_user_id: i32,
) -> Result<Vec<SessionListItem>, AppError> {
    let is_admin =
        crate::services::auth::current_active_role(state, actor_user_id).await? == "admin";

    let sessions = if is_admin {
        ai_chat_session::Entity::find()
            .order_by_desc(ai_chat_session::Column::UpdatedAt)
            .all(&state.db)
            .await?
    } else {
        ai_chat_session::Entity::find()
            .filter(ai_chat_session::Column::UserId.eq(actor_user_id))
            .order_by_desc(ai_chat_session::Column::UpdatedAt)
            .all(&state.db)
            .await?
    };

    let mut result = Vec::new();
    for session in sessions {
        let count = ai_chat_message::Entity::find()
            .filter(ai_chat_message::Column::SessionId.eq(session.id))
            .count(&state.db)
            .await?;
        let user_display_name = if is_admin {
            users::Entity::find_by_id(session.user_id)
                .one(&state.db)
                .await?
                .and_then(|user| user.display_name.or(Some(user.username)))
        } else {
            None
        };
        result.push(SessionListItem {
            id: session.id,
            title: session.title,
            user_id: session.user_id,
            user_display_name,
            agent_config_id: session.agent_config_id,
            msg_count: count as usize,
            created_at: session.created_at,
            updated_at: session.updated_at,
        });
    }

    Ok(result)
}

pub(crate) async fn get_session_detail(
    state: &AppState,
    actor_user_id: i32,
    session_id: i32,
) -> Result<SessionDetailData, AppError> {
    let session = ai_chat_session::Entity::find_by_id(session_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("会话不存在".into()))?;
    authorize_read_or_delete(state, actor_user_id, session.user_id).await?;

    let messages = ai_chat_message::Entity::find()
        .filter(ai_chat_message::Column::SessionId.eq(session_id))
        .order_by_asc(ai_chat_message::Column::CreatedAt)
        .all(&state.db)
        .await?;

    Ok(SessionDetailData { session, messages })
}

pub(crate) async fn delete_session(
    state: &AppState,
    actor_user_id: i32,
    session_id: i32,
) -> Result<(), AppError> {
    let session = ai_chat_session::Entity::find_by_id(session_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("会话不存在".into()))?;
    authorize_read_or_delete(state, actor_user_id, session.user_id).await?;
    ai_chat_session::Entity::delete_by_id(session_id)
        .exec(&state.db)
        .await?;
    Ok(())
}

pub(crate) async fn handle_slash_command(
    state: &AppState,
    actor_user_id: i32,
    cmd: &str,
    requested_session_id: Option<i32>,
    agent_config_id: Option<i32>,
) -> Result<SlashCommandResult, AppError> {
    let now = crate::utils::now_local();
    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
    let command = parts[0].to_lowercase();

    if let Some(session_id) = requested_session_id {
        let session = ai_chat_session::Entity::find_by_id(session_id)
            .one(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("会话不存在".into()))?;
        if actor_user_id != session.user_id {
            return Err(AppError::NotFound("会话不存在".into()));
        }
    }

    let mut created_session_id = None;
    let reply = match command.as_str() {
        "/new" => {
            let title = String::from(*parts.get(1).unwrap_or(&"新会话"));
            let model = ai_chat_session::ActiveModel {
                user_id: Set(actor_user_id),
                title: Set(title.clone()),
                agent_config_id: Set(agent_config_id),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            };
            let session = model.insert(&state.db).await?;
            created_session_id = Some(session.id);
            format!("✅ 已创建新会话「{}」(ID: {})", title, session.id)
        }
        "/model" => {
            let models = ai_model::Entity::find()
                .order_by_asc(ai_model::Column::ProviderId)
                .order_by_asc(ai_model::Column::Name)
                .all(&state.db)
                .await?;
            if models.is_empty() {
                "📭 暂无可用模型。请在管理后台添加模型。".to_string()
            } else {
                let mut out = "📋 可用模型：\n\n".to_string();
                for m in &models {
                    let provider = ai_provider::Entity::find_by_id(m.provider_id)
                        .one(&state.db)
                        .await?;
                    let pname = provider
                        .map(|p| p.name)
                        .unwrap_or_else(|| "未知".to_string());
                    out.push_str(&format!(
                        "  • {} ({})  {}\n",
                        m.name,
                        pname,
                        if m.is_default { "⭐默认" } else { "" }
                    ));
                }
                out
            }
        }
        "/help" => "可用命令：\n\
             • /new [标题] — 新建会话\n\
             • /model — 查看可用模型\n\
             • /help — 显示帮助"
            .to_string(),
        _ => {
            format!("❓ 未知命令「{}」。输入 /help 查看可用命令。", command)
        }
    };

    let session_id = if let Some(sid) = created_session_id {
        sid
    } else if let Some(sid) = requested_session_id {
        sid
    } else {
        let model = ai_chat_session::ActiveModel {
            user_id: Set(actor_user_id),
            title: Set("命令".to_string()),
            agent_config_id: Set(agent_config_id),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        model.insert(&state.db).await?.id
    };

    let msg = ai_chat_message::ActiveModel {
        session_id: Set(session_id),
        role: Set("system".to_string()),
        content: Set(format!("用户执行命令: {}", cmd)),
        created_at: Set(now),
        ..Default::default()
    };
    msg.insert(&state.db).await?;

    Ok(SlashCommandResult { reply, session_id })
}
