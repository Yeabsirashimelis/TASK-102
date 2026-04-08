use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::approval::*;
use crate::rbac::guard::{check_permission, check_permission_for_request, check_permission_no_approval};
use crate::schema::{approval_policies, approval_requests};

pub async fn list(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission_for_request(&auth.0, "approval.list", req.method().as_str(), req.path(), &mut conn)?;

    let results: Vec<ApprovalRequest> = approval_requests::table
        .select(ApprovalRequest::as_select())
        .order(approval_requests::created_at.desc())
        .load(&mut conn)?;

    let responses: Vec<ApprovalRequestResponse> =
        results.into_iter().map(ApprovalRequestResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let req_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission_for_request(&auth.0, "approval.read", req.method().as_str(), req.path(), &mut conn)?;

    let approval: ApprovalRequest = approval_requests::table
        .find(req_id)
        .select(ApprovalRequest::as_select())
        .first(&mut conn)?;
    Ok(HttpResponse::Ok().json(ApprovalRequestResponse::from(approval)))
}

pub async fn approve(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let req_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission_for_request(&auth.0, "approval.decide", req.method().as_str(), req.path(), &mut conn)?;

    let approval: ApprovalRequest = approval_requests::table
        .find(req_id)
        .select(ApprovalRequest::as_select())
        .first(&mut conn)?;

    if approval.status != ApprovalStatus::Pending {
        return Err(AppError::Validation("Approval request is not pending".into()));
    }

    // Independent approver rule: requester cannot approve their own request
    if approval.requester_user_id == auth.0.sub {
        return Err(AppError::Forbidden(
            "Requester cannot approve their own request — independent approver required".into(),
        ));
    }

    let policy: ApprovalPolicy = approval_policies::table
        .filter(approval_policies::permission_point_id.eq(approval.permission_point_id))
        .filter(approval_policies::approver_role_id.eq(auth.0.role_id))
        .first(&mut conn)
        .map_err(|_| {
            AppError::Forbidden("Your role is not an authorized approver for this action".into())
        })?;

    let approver_id = auth.0.sub;
    if approval.approved_by.contains(&approver_id) {
        return Err(AppError::Validation("You have already approved this request".into()));
    }

    let mut new_approved = approval.approved_by.clone();
    new_approved.push(approver_id);

    let new_count = new_approved.len() as i32;

    if new_count >= policy.min_approvers {
        diesel::update(approval_requests::table.find(req_id))
            .set((
                approval_requests::approved_by.eq(&new_approved),
                approval_requests::status.eq(ApprovalStatus::Approved),
                approval_requests::resolved_at.eq(Some(Utc::now())),
            ))
            .execute(&mut conn)?;
    } else {
        diesel::update(approval_requests::table.find(req_id))
            .set(approval_requests::approved_by.eq(&new_approved))
            .execute(&mut conn)?;
    }

    let updated: ApprovalRequest = approval_requests::table
        .find(req_id)
        .select(ApprovalRequest::as_select())
        .first(&mut conn)?;
    Ok(HttpResponse::Ok().json(ApprovalRequestResponse::from(updated)))
}

pub async fn reject(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let req_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission_for_request(&auth.0, "approval.decide", req.method().as_str(), req.path(), &mut conn)?;

    let approval: ApprovalRequest = approval_requests::table
        .find(req_id)
        .select(ApprovalRequest::as_select())
        .first(&mut conn)?;

    if approval.status != ApprovalStatus::Pending {
        return Err(AppError::Validation("Approval request is not pending".into()));
    }

    // Independent approver rule
    if approval.requester_user_id == auth.0.sub {
        return Err(AppError::Forbidden(
            "Requester cannot reject their own request — independent reviewer required".into(),
        ));
    }

    let _policy: ApprovalPolicy = approval_policies::table
        .filter(approval_policies::permission_point_id.eq(approval.permission_point_id))
        .filter(approval_policies::approver_role_id.eq(auth.0.role_id))
        .first(&mut conn)
        .map_err(|_| {
            AppError::Forbidden("Your role is not an authorized approver for this action".into())
        })?;

    diesel::update(approval_requests::table.find(req_id))
        .set((
            approval_requests::status.eq(ApprovalStatus::Rejected),
            approval_requests::rejected_by.eq(Some(auth.0.sub)),
            approval_requests::resolved_at.eq(Some(Utc::now())),
        ))
        .execute(&mut conn)?;

    let updated: ApprovalRequest = approval_requests::table
        .find(req_id)
        .select(ApprovalRequest::as_select())
        .first(&mut conn)?;
    Ok(HttpResponse::Ok().json(ApprovalRequestResponse::from(updated)))
}

pub async fn create_approval_request(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    body: web::Json<CreateApprovalRequestInput>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "approval.request.create", &mut conn)?;

    let new_req = NewApprovalRequest {
        permission_point_id: body.permission_point_id,
        requester_user_id: auth.0.sub,
        payload: body.payload.clone(),
    };

    let req: ApprovalRequest = diesel::insert_into(approval_requests::table)
        .values(&new_req)
        .returning(ApprovalRequest::as_returning())
        .get_result(&mut conn)?;

    Ok(HttpResponse::Accepted().json(ApprovalRequestResponse::from(req)))
}

#[derive(serde::Deserialize)]
pub struct CreateApprovalRequestInput {
    pub permission_point_id: Uuid,
    pub payload: serde_json::Value,
}
