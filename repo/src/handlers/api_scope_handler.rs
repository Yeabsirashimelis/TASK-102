use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::api_capability::*;
use crate::rbac::guard::check_permission;
use crate::schema::api_capabilities;

pub async fn list(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "api_cap.list", &mut conn)?;

    let results: Vec<ApiCapability> = api_capabilities::table
        .order(api_capabilities::path_pattern.asc())
        .load(&mut conn)?;

    let responses: Vec<ApiCapabilityResponse> =
        results.into_iter().map(ApiCapabilityResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let cap_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "api_cap.read", &mut conn)?;

    let cap: ApiCapability = api_capabilities::table.find(cap_id).first(&mut conn)?;
    Ok(HttpResponse::Ok().json(ApiCapabilityResponse::from(cap)))
}

pub async fn create(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    body: web::Json<CreateApiCapabilityRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "api_cap.create", &mut conn)?;

    let new_cap = NewApiCapability {
        permission_point_id: body.permission_point_id,
        http_method: body.http_method.clone(),
        path_pattern: body.path_pattern.clone(),
        description: body.description.clone(),
    };

    let cap: ApiCapability = diesel::insert_into(api_capabilities::table)
        .values(&new_cap)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Created().json(ApiCapabilityResponse::from(cap)))
}

pub async fn update(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateApiCapabilityRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let cap_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "api_cap.update", &mut conn)?;

    let changeset = UpdateApiCapability {
        permission_point_id: body.permission_point_id,
        http_method: body.http_method.clone(),
        path_pattern: body.path_pattern.clone(),
        description: body.description.clone(),
    };

    let cap: ApiCapability = diesel::update(api_capabilities::table.find(cap_id))
        .set(&changeset)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Ok().json(ApiCapabilityResponse::from(cap)))
}

pub async fn delete(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let cap_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "api_cap.delete", &mut conn)?;

    diesel::delete(api_capabilities::table.find(cap_id)).execute(&mut conn)?;
    Ok(HttpResponse::NoContent().finish())
}
