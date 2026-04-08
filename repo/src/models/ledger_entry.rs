use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::ledger_entries;

#[derive(Debug, Clone, PartialEq, diesel_derive_enum::DbEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[ExistingTypePath = "crate::schema::sql_types::TenderTypeType"]
pub enum TenderType {
    #[db_rename = "cash"]
    Cash,
    #[db_rename = "card"]
    Card,
    #[db_rename = "gift_card"]
    GiftCard,
}

#[derive(Debug, Clone, PartialEq, diesel_derive_enum::DbEnum, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::LedgerEntryKindType"]
pub enum LedgerEntryKind {
    #[db_rename = "payment"]
    Payment,
    #[db_rename = "refund"]
    Refund,
    #[db_rename = "reversal"]
    Reversal,
}

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = ledger_entries)]
pub struct LedgerEntry {
    pub id: Uuid,
    pub order_id: Uuid,
    pub tender_type: TenderType,
    pub entry_kind: LedgerEntryKind,
    pub amount_cents: i64,
    pub reference_code: Option<String>,
    pub idempotency_key: Uuid,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = ledger_entries)]
pub struct NewLedgerEntry {
    pub order_id: Uuid,
    pub tender_type: TenderType,
    pub entry_kind: LedgerEntryKind,
    pub amount_cents: i64,
    pub reference_code: Option<String>,
    pub idempotency_key: Uuid,
    pub created_by: Uuid,
}

#[derive(Serialize)]
pub struct LedgerEntryResponse {
    pub id: Uuid,
    pub order_id: Uuid,
    pub tender_type: TenderType,
    pub entry_kind: LedgerEntryKind,
    pub amount_cents: i64,
    pub reference_code: Option<String>,
    pub idempotency_key: Uuid,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<LedgerEntry> for LedgerEntryResponse {
    fn from(le: LedgerEntry) -> Self {
        Self {
            id: le.id,
            order_id: le.order_id,
            tender_type: le.tender_type,
            entry_kind: le.entry_kind,
            amount_cents: le.amount_cents,
            reference_code: le.reference_code,
            idempotency_key: le.idempotency_key,
            created_by: le.created_by,
            created_at: le.created_at,
        }
    }
}

#[derive(Deserialize)]
pub struct AddPaymentRequest {
    pub tender_type: TenderType,
    pub amount_cents: i64,
    pub reference_code: Option<String>,
    pub idempotency_key: Uuid,
}
