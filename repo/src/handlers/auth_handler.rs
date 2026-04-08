use actix_web::{web, HttpRequest, HttpResponse};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::auth::{jwt, lockout, password};
use crate::config::AppConfig;
use crate::crypto::FieldEncryptor;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::login_attempt::NewLoginAttempt;
use crate::models::role::Role;
use crate::models::user::{NewUser, User};
use crate::rbac::delegation::get_active_delegations;
use crate::schema::{login_attempts, roles, users};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

fn client_ip(req: &HttpRequest) -> Option<String> {
    req.peer_addr().map(|a| a.ip().to_string())
}

pub async fn login(
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
    encryptor: web::Data<FieldEncryptor>,
    req: HttpRequest,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ip = client_ip(&req);

    // Look up user
    let user: User = match users::table
        .filter(users::username.eq(&body.username))
        .filter(users::is_active.eq(true))
        .first(&mut conn)
    {
        Ok(u) => u,
        Err(_) => {
            // Log attempt for unknown user
            diesel::insert_into(login_attempts::table)
                .values(&NewLoginAttempt {
                    username: body.username.clone(),
                    success: false,
                    ip_address: ip,
                })
                .execute(&mut conn)?;
            return Err(AppError::Unauthorized("Invalid credentials".into()));
        }
    };

    // Check lockout
    lockout::check_lockout(&user)?;

    // Decrypt stored password hash
    let hash_bytes = encryptor.decrypt(&user.password_hash_enc)?;
    let hash_str = String::from_utf8(hash_bytes)
        .map_err(|e| AppError::Internal(format!("Invalid hash encoding: {}", e)))?;

    // Verify password
    let valid = password::verify_password(&body.password, &hash_str)?;

    if !valid {
        lockout::record_failed_attempt(&mut conn, user.id, user.failed_attempts, &config)?;
        diesel::insert_into(login_attempts::table)
            .values(&NewLoginAttempt {
                username: body.username.clone(),
                success: false,
                ip_address: ip,
            })
            .execute(&mut conn)?;
        return Err(AppError::Unauthorized("Invalid credentials".into()));
    }

    // Reset failed attempts on success
    lockout::reset_failed_attempts(&mut conn, user.id)?;

    // Log successful attempt
    diesel::insert_into(login_attempts::table)
        .values(&NewLoginAttempt {
            username: body.username.clone(),
            success: true,
            ip_address: ip,
        })
        .execute(&mut conn)?;

    // Fetch role
    let role: Role = roles::table.find(user.role_id).first(&mut conn)?;

    // Fetch active delegations
    let delegations = get_active_delegations(&mut conn, user.id)?;

    // Issue tokens
    let access_token = jwt::issue_access_token(&user, &role, &delegations, &config)?;
    let refresh_token = jwt::issue_refresh_token(user.id, &config)?;

    Ok(HttpResponse::Ok().json(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".into(),
        expires_in: config.jwt_access_ttl_secs,
    }))
}

pub async fn refresh(
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
    body: web::Json<RefreshRequest>,
) -> Result<HttpResponse, AppError> {
    let claims = jwt::decode_token(&body.refresh_token, &config)?;
    if claims.token_type != "refresh" {
        return Err(AppError::Unauthorized("Invalid token type".into()));
    }

    let mut conn = pool.get().map_err(crate::errors::pool_err)?;

    let user: User = users::table
        .find(claims.sub)
        .filter(users::is_active.eq(true))
        .first(&mut conn)
        .map_err(|_| AppError::Unauthorized("User not found or inactive".into()))?;

    lockout::check_lockout(&user)?;

    let role: Role = roles::table.find(user.role_id).first(&mut conn)?;
    let delegations = get_active_delegations(&mut conn, user.id)?;

    let access_token = jwt::issue_access_token(&user, &role, &delegations, &config)?;

    Ok(HttpResponse::Ok().json(RefreshResponse {
        access_token,
        token_type: "Bearer".into(),
        expires_in: config.jwt_access_ttl_secs,
    }))
}

#[derive(Deserialize)]
pub struct BootstrapRequest {
    pub username: String,
    pub password: String,
}

/// POST /api/v1/auth/bootstrap
/// Creates the initial admin user. Only works when zero users exist.
pub async fn bootstrap(
    pool: web::Data<DbPool>,
    config: web::Data<AppConfig>,
    encryptor: web::Data<FieldEncryptor>,
    body: web::Json<BootstrapRequest>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;

    // Only allow if no users exist
    let count: i64 = users::table.count().get_result(&mut conn)?;
    if count > 0 {
        return Err(AppError::Forbidden(
            "Bootstrap is only available when no users exist".into(),
        ));
    }

    // Validate password
    password::validate_password(&body.password)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Hash and encrypt password
    let hash = password::hash_password(&body.password)?;
    let hash_enc = encryptor.encrypt(hash.as_bytes())?;

    let admin_role_id: uuid::Uuid = "a0000000-0000-0000-0000-000000000001"
        .parse()
        .unwrap();

    let new_user = NewUser {
        username: body.username.clone(),
        password_hash_enc: hash_enc,
        gov_id_enc: None,
        gov_id_last4: None,
        role_id: admin_role_id,
        department: None,
        location: None,
    };

    let user: User = diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(&mut conn)?;

    // Issue tokens immediately
    let role: Role = roles::table.find(user.role_id).first(&mut conn)?;
    let access_token = jwt::issue_access_token(&user, &role, &[], &config)?;
    let refresh_token = jwt::issue_refresh_token(user.id, &config)?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Admin user created",
        "user_id": user.id,
        "username": user.username,
        "access_token": access_token,
        "refresh_token": refresh_token,
        "token_type": "Bearer",
        "expires_in": config.jwt_access_ttl_secs,
    })))
}
