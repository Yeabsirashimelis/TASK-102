use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::audit_log;

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = audit_log)]
pub struct AuditEntry {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub http_method: String,
    pub http_path: String,
    pub before_hash: Option<String>,
    pub after_hash: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = audit_log)]
pub struct NewAuditEntry {
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub http_method: String,
    pub http_path: String,
    pub before_hash: Option<String>,
    pub after_hash: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub ip_address: Option<String>,
}

#[derive(Serialize)]
pub struct AuditEntryResponse {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub http_method: String,
    pub http_path: String,
    pub before_hash: Option<String>,
    pub after_hash: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<AuditEntry> for AuditEntryResponse {
    fn from(a: AuditEntry) -> Self {
        Self {
            id: a.id,
            user_id: a.user_id,
            action: a.action,
            resource_type: a.resource_type,
            resource_id: a.resource_id,
            http_method: a.http_method,
            http_path: a.http_path,
            before_hash: a.before_hash,
            after_hash: a.after_hash,
            metadata: a.metadata,
            ip_address: a.ip_address,
            created_at: a.created_at,
        }
    }
}

#[derive(Deserialize)]
pub struct AuditQueryParams {
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
