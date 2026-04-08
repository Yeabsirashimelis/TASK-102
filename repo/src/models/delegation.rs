use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::delegations;

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = delegations)]
pub struct Delegation {
    pub id: Uuid,
    pub delegator_user_id: Uuid,
    pub delegate_user_id: Uuid,
    pub permission_point_id: Uuid,
    pub source_department: Option<String>,
    pub target_department: Option<String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = delegations)]
pub struct NewDelegation {
    pub delegator_user_id: Uuid,
    pub delegate_user_id: Uuid,
    pub permission_point_id: Uuid,
    pub source_department: Option<String>,
    pub target_department: Option<String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct DelegationResponse {
    pub id: Uuid,
    pub delegator_user_id: Uuid,
    pub delegate_user_id: Uuid,
    pub permission_point_id: Uuid,
    pub source_department: Option<String>,
    pub target_department: Option<String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<Delegation> for DelegationResponse {
    fn from(d: Delegation) -> Self {
        Self {
            id: d.id,
            delegator_user_id: d.delegator_user_id,
            delegate_user_id: d.delegate_user_id,
            permission_point_id: d.permission_point_id,
            source_department: d.source_department,
            target_department: d.target_department,
            starts_at: d.starts_at,
            ends_at: d.ends_at,
            is_active: d.is_active,
            created_at: d.created_at,
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct CreateDelegationRequest {
    pub delegate_user_id: Uuid,
    pub permission_point_id: Uuid,
    pub source_department: Option<String>,
    pub target_department: Option<String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}
