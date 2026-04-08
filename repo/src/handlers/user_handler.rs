use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use validator::Validate;

use crate::auth::middleware::AuthenticatedUser;
use crate::auth::password;
use crate::config::AppConfig;
use crate::crypto::FieldEncryptor;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::user::{CreateUserRequest, NewUser, User, UserResponse};
use crate::rbac::guard::check_permission;
use crate::schema::users;

pub async fn create_user(
    pool: web::Data<DbPool>,
    _config: web::Data<AppConfig>,
    encryptor: web::Data<FieldEncryptor>,
    auth: AuthenticatedUser,
    body: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "user.create", &mut conn)?;

    // Hash the password, then encrypt the hash
    let hash = password::hash_password(&body.password)?;
    let hash_enc = encryptor.encrypt(hash.as_bytes())?;

    // Handle government ID
    let (gov_id_enc, gov_id_last4) = if let Some(ref gov_id) = body.gov_id {
        let encrypted = encryptor.encrypt(gov_id.as_bytes())?;
        let last4 = if gov_id.len() >= 4 {
            Some(gov_id[gov_id.len() - 4..].to_string())
        } else {
            Some(gov_id.clone())
        };
        (Some(encrypted), last4)
    } else {
        (None, None)
    };

    let new_user = NewUser {
        username: body.username.clone(),
        password_hash_enc: hash_enc,
        gov_id_enc,
        gov_id_last4,
        role_id: body.role_id,
        department: body.department.clone(),
        location: body.location.clone(),
    };

    let user: User = diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Created().json(UserResponse::from(user)))
}

pub async fn list_users(
    pool: web::Data<DbPool>,
    _config: web::Data<AppConfig>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_permission(&auth.0, "user.list", &mut conn)?;

    let mut query = users::table
        .filter(users::is_active.eq(true))
        .into_boxed();

    // Apply data-scope filtering
    match ctx.data_scope.as_str() {
        "department" => {
            if let Some(ref dept) = ctx.department {
                query = query.filter(users::department.eq(dept));
            }
        }
        "location" => {
            if let Some(ref loc) = ctx.location {
                query = query.filter(users::location.eq(loc));
            }
        }
        "individual" => {
            query = query.filter(users::id.eq(ctx.user_id));
        }
        _ => {} // unrestricted
    }

    let results: Vec<User> = query.load(&mut conn)?;
    let responses: Vec<UserResponse> = results.into_iter().map(UserResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get_user(
    pool: web::Data<DbPool>,
    _config: web::Data<AppConfig>,
    auth: AuthenticatedUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_permission(&auth.0, "user.read", &mut conn)?;

    let user: User = users::table.find(user_id).first(&mut conn)?;

    if !ctx.owner_in_scope(user.id)
        || !ctx.department_in_scope(user.department.as_deref())
        || !ctx.location_in_scope(user.location.as_deref())
    {
        return Err(AppError::Forbidden("Out of data scope".into()));
    }

    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}
