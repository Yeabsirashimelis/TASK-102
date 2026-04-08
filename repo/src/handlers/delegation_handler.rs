use actix_web::{web, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::delegation::*;
use crate::rbac::guard::check_permission;
use crate::schema::{delegations, role_permissions};

pub async fn list(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_permission(&auth.0, "delegation.list", &mut conn)?;

    let mut query = delegations::table.into_boxed();

    // Non-admin users see only delegations they created or received
    if ctx.data_scope != "" {
        query = query.filter(
            delegations::delegator_user_id
                .eq(ctx.user_id)
                .or(delegations::delegate_user_id.eq(ctx.user_id)),
        );
    }

    let results: Vec<Delegation> = query
        .order(delegations::created_at.desc())
        .load(&mut conn)?;

    let responses: Vec<DelegationResponse> =
        results.into_iter().map(DelegationResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn create(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    body: web::Json<CreateDelegationRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "delegation.create", &mut conn)?;

    // Validate time bounds
    if body.ends_at <= body.starts_at {
        return Err(AppError::Validation(
            "ends_at must be after starts_at".into(),
        ));
    }
    if body.ends_at <= Utc::now() {
        return Err(AppError::Validation(
            "ends_at must be in the future".into(),
        ));
    }

    // Verify the delegator holds the permission being delegated
    let delegator_has_perm: bool = role_permissions::table
        .filter(role_permissions::role_id.eq(auth.0.role_id))
        .filter(role_permissions::permission_point_id.eq(body.permission_point_id))
        .count()
        .get_result::<i64>(&mut conn)
        .map(|c| c > 0)
        .unwrap_or(false);

    if !delegator_has_perm {
        return Err(AppError::Forbidden(
            "Cannot delegate a permission you do not hold".into(),
        ));
    }

    let new_delegation = NewDelegation {
        delegator_user_id: auth.0.sub,
        delegate_user_id: body.delegate_user_id,
        permission_point_id: body.permission_point_id,
        source_department: body.source_department.clone(),
        target_department: body.target_department.clone(),
        starts_at: body.starts_at,
        ends_at: body.ends_at,
    };

    let delegation: Delegation = diesel::insert_into(delegations::table)
        .values(&new_delegation)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Created().json(DelegationResponse::from(delegation)))
}

pub async fn revoke(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let deleg_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "delegation.revoke", &mut conn)?;

    // Only the delegator or an admin can revoke
    let delegation: Delegation = delegations::table.find(deleg_id).first(&mut conn)?;
    if delegation.delegator_user_id != auth.0.sub {
        // Check if user is admin (empty data_scope = unrestricted)
        if !auth.0.data_scope.is_empty() {
            return Err(AppError::Forbidden(
                "Only the delegator can revoke this delegation".into(),
            ));
        }
    }

    diesel::update(delegations::table.find(deleg_id))
        .set(delegations::is_active.eq(false))
        .execute(&mut conn)?;

    Ok(HttpResponse::NoContent().finish())
}
