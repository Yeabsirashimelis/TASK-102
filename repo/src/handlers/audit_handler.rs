use actix_web::{web, HttpResponse};
use diesel::prelude::*;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::audit_log::*;
use crate::rbac::guard::check_permission;
use crate::schema::audit_log;

pub async fn list(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    query: web::Query<AuditQueryParams>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "audit.read", &mut conn)?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    let mut q = audit_log::table.into_boxed();

    if let Some(uid) = query.user_id {
        q = q.filter(audit_log::user_id.eq(uid));
    }
    if let Some(ref action) = query.action {
        q = q.filter(audit_log::action.eq(action));
    }
    if let Some(ref rt) = query.resource_type {
        q = q.filter(audit_log::resource_type.eq(rt));
    }
    if let Some(rid) = query.resource_id {
        q = q.filter(audit_log::resource_id.eq(rid));
    }
    if let Some(from) = query.from {
        q = q.filter(audit_log::created_at.ge(from));
    }
    if let Some(to) = query.to {
        q = q.filter(audit_log::created_at.le(to));
    }

    let results: Vec<AuditEntry> = q
        .select(AuditEntry::as_select())
        .order(audit_log::created_at.desc())
        .offset(offset)
        .limit(per_page)
        .load(&mut conn)?;

    let responses: Vec<AuditEntryResponse> =
        results.into_iter().map(AuditEntryResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    let entry_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "audit.read", &mut conn)?;

    let entry: AuditEntry = audit_log::table
        .find(entry_id)
        .select(AuditEntry::as_select())
        .first(&mut conn)?;

    Ok(HttpResponse::Ok().json(AuditEntryResponse::from(entry)))
}
