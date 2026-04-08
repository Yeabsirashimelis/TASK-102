use actix_web::{web, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::role::*;
use crate::rbac::guard::check_permission;
use crate::schema::roles;

pub async fn list(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "role.list", &mut conn)?;

    let results: Vec<Role> = roles::table
        .filter(roles::is_active.eq(true))
        .order(roles::name.asc())
        .load(&mut conn)?;

    let responses: Vec<RoleResponse> = results.into_iter().map(RoleResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let role_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "role.read", &mut conn)?;

    let role: Role = roles::table.find(role_id).first(&mut conn)?;
    Ok(HttpResponse::Ok().json(RoleResponse::from(role)))
}

pub async fn create(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    body: web::Json<CreateRoleRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "role.create", &mut conn)?;

    let new_role = NewRole {
        name: body.name.clone(),
        description: body.description.clone(),
        data_scope: body.data_scope.clone(),
        scope_value: body.scope_value.clone(),
    };

    let role: Role = diesel::insert_into(roles::table)
        .values(&new_role)
        .get_result(&mut conn)?;

    let after = serde_json::json!({"id": role.id, "name": &role.name});
    let _ = crate::audit::service::audit_write(&mut conn, auth.0.sub, "create", "roles", Some(role.id), None, Some(&after));

    Ok(HttpResponse::Created().json(RoleResponse::from(role)))
}

pub async fn update(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateRoleRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let role_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "role.update", &mut conn)?;

    let before_role: Role = roles::table.find(role_id).first(&mut conn)?;
    let before = serde_json::json!({"id": before_role.id, "name": &before_role.name, "is_active": before_role.is_active});

    let changeset = UpdateRole {
        name: body.name.clone(),
        description: body.description.clone(),
        data_scope: body.data_scope.clone(),
        scope_value: body.scope_value.clone(),
        is_active: body.is_active,
        updated_at: Utc::now(),
    };

    let role: Role = diesel::update(roles::table.find(role_id))
        .set(&changeset)
        .get_result(&mut conn)?;

    let after = serde_json::json!({"id": role.id, "name": &role.name, "is_active": role.is_active});
    let _ = crate::audit::service::audit_write(&mut conn, auth.0.sub, "update", "roles", Some(role_id), Some(&before), Some(&after));

    Ok(HttpResponse::Ok().json(RoleResponse::from(role)))
}

pub async fn delete(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let role_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "role.delete", &mut conn)?;

    let before = serde_json::json!({"id": role_id, "is_active": true});
    diesel::update(roles::table.find(role_id))
        .set((roles::is_active.eq(false), roles::updated_at.eq(Utc::now())))
        .execute(&mut conn)?;
    let after = serde_json::json!({"id": role_id, "is_active": false});
    let _ = crate::audit::service::audit_write(&mut conn, auth.0.sub, "delete", "roles", Some(role_id), Some(&before), Some(&after));

    Ok(HttpResponse::NoContent().finish())
}
