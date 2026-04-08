use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::menu_scope::*;
use crate::rbac::guard::check_permission;
use crate::schema::menu_scopes;

pub async fn list(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "menu_scope.list", &mut conn)?;

    let results: Vec<MenuScope> = menu_scopes::table
        .order(menu_scopes::menu_key.asc())
        .load(&mut conn)?;

    let responses: Vec<MenuScopeResponse> =
        results.into_iter().map(MenuScopeResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let scope_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "menu_scope.read", &mut conn)?;

    let scope: MenuScope = menu_scopes::table.find(scope_id).first(&mut conn)?;
    Ok(HttpResponse::Ok().json(MenuScopeResponse::from(scope)))
}

pub async fn create(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    body: web::Json<CreateMenuScopeRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "menu_scope.create", &mut conn)?;

    let new_scope = NewMenuScope {
        permission_point_id: body.permission_point_id,
        menu_key: body.menu_key.clone(),
        description: body.description.clone(),
    };

    let scope: MenuScope = diesel::insert_into(menu_scopes::table)
        .values(&new_scope)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Created().json(MenuScopeResponse::from(scope)))
}

pub async fn update(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateMenuScopeRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let scope_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "menu_scope.update", &mut conn)?;

    let changeset = UpdateMenuScope {
        permission_point_id: body.permission_point_id,
        menu_key: body.menu_key.clone(),
        description: body.description.clone(),
    };

    let scope: MenuScope = diesel::update(menu_scopes::table.find(scope_id))
        .set(&changeset)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Ok().json(MenuScopeResponse::from(scope)))
}

pub async fn delete(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let scope_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "menu_scope.delete", &mut conn)?;

    diesel::delete(menu_scopes::table.find(scope_id)).execute(&mut conn)?;
    Ok(HttpResponse::NoContent().finish())
}
