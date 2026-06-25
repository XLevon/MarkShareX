use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use crate::config::AuthConfig;
use crate::utils::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user_id: i32,
    pub username: String,
    pub role: String,
    pub status: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn generate_token(user_id: i32, username: &str, role: &str, status: &str, config: &AuthConfig) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = Claims {
        user_id,
        username: username.to_string(),
        role: role.to_string(),
        status: status.to_string(),
        iat: now.timestamp(),
        exp: (now + Duration::seconds(config.jwt_expire_seconds)).timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("JWT encode error: {}", e)))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())
        .map(|data| data.claims)
        .map_err(|_| AppError::Unauthorized)
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Password hash error: {}", e)))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    bcrypt::verify(password, hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Password verify error: {}", e)))
}
