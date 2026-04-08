use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Serialize;
use uuid::Uuid;

use crate::schema::receipts;

#[derive(Debug, Queryable, Identifiable, Selectable)]
#[diesel(table_name = receipts)]
pub struct Receipt {
    pub id: Uuid,
    pub order_id: Uuid,
    pub receipt_number: String,
    pub receipt_data: serde_json::Value,
    pub printed_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub file_path: Option<String>,
    pub content_type: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub sha256_hash: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = receipts)]
pub struct NewReceipt {
    pub order_id: Uuid,
    pub receipt_number: String,
    pub receipt_data: serde_json::Value,
    pub created_by: Uuid,
    pub file_path: Option<String>,
    pub content_type: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub sha256_hash: Option<String>,
}

#[derive(Serialize)]
pub struct ReceiptResponse {
    pub id: Uuid,
    pub order_id: Uuid,
    pub receipt_number: String,
    pub receipt_data: serde_json::Value,
    pub printed_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub file_path: Option<String>,
    pub content_type: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub sha256_hash: Option<String>,
}

impl From<Receipt> for ReceiptResponse {
    fn from(r: Receipt) -> Self {
        Self {
            id: r.id,
            order_id: r.order_id,
            receipt_number: r.receipt_number,
            receipt_data: r.receipt_data,
            printed_at: r.printed_at,
            created_by: r.created_by,
            file_path: r.file_path,
            content_type: r.content_type,
            file_size_bytes: r.file_size_bytes,
            sha256_hash: r.sha256_hash,
        }
    }
}
