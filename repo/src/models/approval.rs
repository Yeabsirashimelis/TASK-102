use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::{approval_policies, approval_requests};

// --- Approval Status enum ---

#[derive(Debug, Clone, PartialEq, diesel_derive_enum::DbEnum, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::ApprovalStatusType"]
pub enum ApprovalStatus {
    #[db_rename = "pending"]
    Pending,
    #[db_rename = "approved"]
    Approved,
    #[db_rename = "rejected"]
    Rejected,
    #[db_rename = "expired"]
    Expired,
}

// --- Approval Policy ---

#[derive(Debug, Queryable, Identifiable, Selectable)]
#[diesel(table_name = approval_policies)]
pub struct ApprovalPolicy {
    pub id: Uuid,
    pub permission_point_id: Uuid,
    pub min_approvers: i32,
    pub approver_role_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = approval_policies)]
pub struct NewApprovalPolicy {
    pub permission_point_id: Uuid,
    pub min_approvers: i32,
    pub approver_role_id: Uuid,
}

#[derive(Serialize)]
pub struct ApprovalPolicyResponse {
    pub id: Uuid,
    pub permission_point_id: Uuid,
    pub min_approvers: i32,
    pub approver_role_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<ApprovalPolicy> for ApprovalPolicyResponse {
    fn from(p: ApprovalPolicy) -> Self {
        Self {
            id: p.id,
            permission_point_id: p.permission_point_id,
            min_approvers: p.min_approvers,
            approver_role_id: p.approver_role_id,
            created_at: p.created_at,
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct CreateApprovalPolicyRequest {
    pub permission_point_id: Uuid,
    #[validate(range(min = 1))]
    pub min_approvers: i32,
    pub approver_role_id: Uuid,
}

// --- Approval Request ---

#[derive(Debug, Queryable, Identifiable, Selectable)]
#[diesel(table_name = approval_requests)]
pub struct ApprovalRequest {
    pub id: Uuid,
    pub permission_point_id: Uuid,
    pub requester_user_id: Uuid,
    pub payload: serde_json::Value,
    pub status: ApprovalStatus,
    pub approved_by: Vec<Uuid>,
    pub rejected_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = approval_requests)]
pub struct NewApprovalRequest {
    pub permission_point_id: Uuid,
    pub requester_user_id: Uuid,
    pub payload: serde_json::Value,
}

#[derive(Serialize)]
pub struct ApprovalRequestResponse {
    pub id: Uuid,
    pub permission_point_id: Uuid,
    pub requester_user_id: Uuid,
    pub payload: serde_json::Value,
    pub status: ApprovalStatus,
    pub approved_by: Vec<Uuid>,
    pub rejected_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<ApprovalRequest> for ApprovalRequestResponse {
    fn from(a: ApprovalRequest) -> Self {
        Self {
            id: a.id,
            permission_point_id: a.permission_point_id,
            requester_user_id: a.requester_user_id,
            payload: a.payload,
            status: a.status,
            approved_by: a.approved_by,
            rejected_by: a.rejected_by,
            resolved_at: a.resolved_at,
            created_at: a.created_at,
        }
    }
}
