use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::permission_point::*;
use crate::models::role_permission::*;
use crate::rbac::guard::check_permission;
use crate::schema::{permission_points, role_permissions};

pub async fn list(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "permission.list", &mut conn)?;

    let results: Vec<PermissionPoint> = permission_points::table
        .order(permission_points::code.asc())
        .load(&mut conn)?;

    let responses: Vec<PermissionPointResponse> =
        results.into_iter().map(PermissionPointResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let perm_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "permission.read", &mut conn)?;

    let perm: PermissionPoint = permission_points::table.find(perm_id).first(&mut conn)?;
    Ok(HttpResponse::Ok().json(PermissionPointResponse::from(perm)))
}

pub async fn create(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    body: web::Json<CreatePermissionPointRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "permission.create", &mut conn)?;

    let new_perm = NewPermissionPoint {
        code: body.code.clone(),
        description: body.description.clone(),
        requires_approval: body.requires_approval,
    };

    let perm: PermissionPoint = diesel::insert_into(permission_points::table)
        .values(&new_perm)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Created().json(PermissionPointResponse::from(perm)))
}

pub async fn update(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdatePermissionPointRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let perm_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "permission.update", &mut conn)?;

    let changeset = UpdatePermissionPoint {
        code: body.code.clone(),
        description: body.description.clone(),
        requires_approval: body.requires_approval,
    };

    let perm: PermissionPoint = diesel::update(permission_points::table.find(perm_id))
        .set(&changeset)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Ok().json(PermissionPointResponse::from(perm)))
}

pub async fn delete(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let perm_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "permission.delete", &mut conn)?;

    diesel::delete(permission_points::table.find(perm_id)).execute(&mut conn)?;
    Ok(HttpResponse::NoContent().finish())
}

// --- Role-Permission bindings ---

pub async fn bind_to_role(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<BindPermissionRequest>,
) -> Result<HttpResponse, AppError> {
    let role_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "role_permission.bind", &mut conn)?;

    let new_binding = NewRolePermission {
        role_id,
        permission_point_id: body.permission_point_id,
    };

    let rp: RolePermission = diesel::insert_into(role_permissions::table)
        .values(&new_binding)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Created().json(RolePermissionResponse::from(rp)))
}

#[derive(serde::Deserialize)]
pub struct RolePermPath {
    pub role_id: Uuid,
    pub perm_id: Uuid,
}

pub async fn unbind_from_role(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<RolePermPath>,
) -> Result<HttpResponse, AppError> {
    let params = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "role_permission.unbind", &mut conn)?;

    diesel::delete(
        role_permissions::table
            .filter(role_permissions::role_id.eq(params.role_id))
            .filter(role_permissions::permission_point_id.eq(params.perm_id)),
    )
    .execute(&mut conn)?;

    Ok(HttpResponse::NoContent().finish())
}
