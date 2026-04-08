use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::errors::AppError;
use crate::models::delegation::Delegation;
use crate::models::role::Role;
use crate::models::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub role_id: Uuid,
    pub role_name: String,
    pub data_scope: String,
    pub scope_value: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub delegated_permissions: Vec<Uuid>,
    pub exp: i64,
    pub iat: i64,
    pub token_type: String,
}

pub fn issue_access_token(
    user: &User,
    role: &Role,
    delegations: &[Delegation],
    config: &AppConfig,
) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let delegated: Vec<Uuid> = delegations
        .iter()
        .map(|d| d.permission_point_id)
        .collect();

    let claims = Claims {
        sub: user.id,
        role_id: role.id,
        role_name: role.name.clone(),
        data_scope: format!("{:?}", role.data_scope).to_lowercase(),
        scope_value: role.scope_value.clone(),
        department: user.department.clone(),
        location: user.location.clone(),
        delegated_permissions: delegated,
        exp: now + config.jwt_access_ttl_secs,
        iat: now,
        token_type: "access".into(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Token encoding failed: {}", e)))
}

pub fn issue_refresh_token(user_id: Uuid, config: &AppConfig) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id,
        role_id: Uuid::nil(),
        role_name: String::new(),
        data_scope: String::new(),
        scope_value: None,
        department: None,
        location: None,
        delegated_permissions: vec![],
        exp: now + config.jwt_refresh_ttl_secs,
        iat: now,
        token_type: "refresh".into(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Token encoding failed: {}", e)))
}

pub fn decode_token(token: &str, config: &AppConfig) -> Result<Claims, AppError> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

    Ok(data.claims)
}
