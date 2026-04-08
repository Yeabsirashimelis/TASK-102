use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::idempotency_keys;

#[derive(Debug, Queryable, Identifiable, Selectable)]
#[diesel(table_name = idempotency_keys)]
#[diesel(primary_key(key))]
pub struct IdempotencyRecord {
    pub key: Uuid,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub response_status: i16,
    pub response_body: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = idempotency_keys)]
pub struct NewIdempotencyRecord {
    pub key: Uuid,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub response_status: i16,
    pub response_body: serde_json::Value,
}
