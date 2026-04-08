use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::participants;

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = participants)]
pub struct Participant {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub employee_id: Option<String>,
    pub notes: Option<String>,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = participants)]
pub struct NewParticipant {
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub employee_id: Option<String>,
    pub notes: Option<String>,
    pub created_by: Uuid,
}

#[derive(AsChangeset)]
#[diesel(table_name = participants)]
pub struct UpdateParticipant {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<Option<String>>,
    pub phone: Option<Option<String>>,
    pub department: Option<Option<String>>,
    pub location: Option<Option<String>>,
    pub employee_id: Option<Option<String>>,
    pub notes: Option<Option<String>>,
    pub is_active: Option<bool>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ParticipantResponse {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub employee_id: Option<String>,
    pub notes: Option<String>,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl From<Participant> for ParticipantResponse {
    fn from(p: Participant) -> Self {
        Self {
            id: p.id,
            first_name: p.first_name,
            last_name: p.last_name,
            email: p.email,
            phone: p.phone,
            department: p.department,
            location: p.location,
            employee_id: p.employee_id,
            notes: p.notes,
            is_active: p.is_active,
            created_by: p.created_by,
            created_at: p.created_at,
            updated_at: p.updated_at,
            tags: None,
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct CreateParticipantRequest {
    #[validate(length(min = 1, max = 128))]
    pub first_name: String,
    #[validate(length(min = 1, max = 128))]
    pub last_name: String,
    #[validate(email)]
    pub email: Option<String>,
    pub phone: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub employee_id: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Deserialize, Validate)]
pub struct UpdateParticipantRequest {
    #[validate(length(min = 1, max = 128))]
    pub first_name: Option<String>,
    #[validate(length(min = 1, max = 128))]
    pub last_name: Option<String>,
    pub email: Option<Option<String>>,
    pub phone: Option<Option<String>>,
    pub department: Option<Option<String>>,
    pub location: Option<Option<String>>,
    pub employee_id: Option<Option<String>>,
    pub notes: Option<Option<String>>,
    pub is_active: Option<bool>,
}

#[derive(Deserialize)]
pub struct ParticipantSearchParams {
    pub q: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub tag: Option<String>,
    pub is_active: Option<bool>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Deserialize)]
pub struct BulkTagRequest {
    pub participant_ids: Vec<Uuid>,
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
pub struct BulkDeactivateRequest {
    pub participant_ids: Vec<Uuid>,
}

#[derive(Serialize)]
pub struct BulkResultResponse {
    pub affected: usize,
}
