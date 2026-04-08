use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::delivery_logs;

#[derive(Debug, Clone, PartialEq, diesel_derive_enum::DbEnum, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::DeliveryResultType"]
pub enum DeliveryResult {
    #[db_rename = "success"]
    Success,
    #[db_rename = "failure"]
    Failure,
}

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = delivery_logs)]
pub struct DeliveryLog {
    pub id: Uuid,
    pub notification_id: Uuid,
    pub attempt_number: i32,
    pub result: DeliveryResult,
    pub error_message: Option<String>,
    pub attempted_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = delivery_logs)]
pub struct NewDeliveryLog {
    pub notification_id: Uuid,
    pub attempt_number: i32,
    pub result: DeliveryResult,
    pub error_message: Option<String>,
}

#[derive(Serialize)]
pub struct DeliveryLogResponse {
    pub id: Uuid,
    pub notification_id: Uuid,
    pub attempt_number: i32,
    pub result: DeliveryResult,
    pub error_message: Option<String>,
    pub attempted_at: DateTime<Utc>,
}

impl From<DeliveryLog> for DeliveryLogResponse {
    fn from(d: DeliveryLog) -> Self {
        Self {
            id: d.id,
            notification_id: d.notification_id,
            attempt_number: d.attempt_number,
            result: d.result,
            error_message: d.error_message,
            attempted_at: d.attempted_at,
        }
    }
}
