use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::scheduled_reports;

#[derive(Debug, Clone, PartialEq, diesel_derive_enum::DbEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[ExistingTypePath = "crate::schema::sql_types::ScheduleFrequencyType"]
pub enum ScheduleFrequency {
    #[db_rename = "daily"]
    Daily,
    #[db_rename = "weekly"]
    Weekly,
    #[db_rename = "monthly"]
    Monthly,
    #[db_rename = "quarterly"]
    Quarterly,
}

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = scheduled_reports)]
pub struct ScheduledReport {
    pub id: Uuid,
    pub report_definition_id: Uuid,
    pub frequency: ScheduleFrequency,
    pub export_format: String,
    pub next_run_at: DateTime<Utc>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = scheduled_reports)]
pub struct NewScheduledReport {
    pub report_definition_id: Uuid,
    pub frequency: ScheduleFrequency,
    pub export_format: String,
    pub next_run_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(AsChangeset)]
#[diesel(table_name = scheduled_reports)]
pub struct UpdateScheduledReport {
    pub frequency: Option<ScheduleFrequency>,
    pub export_format: Option<String>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ScheduledReportResponse {
    pub id: Uuid,
    pub report_definition_id: Uuid,
    pub frequency: ScheduleFrequency,
    pub export_format: String,
    pub next_run_at: DateTime<Utc>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ScheduledReport> for ScheduledReportResponse {
    fn from(s: ScheduledReport) -> Self {
        Self {
            id: s.id,
            report_definition_id: s.report_definition_id,
            frequency: s.frequency,
            export_format: s.export_format,
            next_run_at: s.next_run_at,
            last_run_at: s.last_run_at,
            is_active: s.is_active,
            created_by: s.created_by,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateScheduledReportRequest {
    pub report_definition_id: Uuid,
    pub frequency: ScheduleFrequency,
    #[serde(default = "default_format")]
    pub export_format: String,
    pub next_run_at: DateTime<Utc>,
}

fn default_format() -> String {
    "xlsx".into()
}

#[derive(Deserialize)]
pub struct UpdateScheduledReportRequest {
    pub frequency: Option<ScheduleFrequency>,
    pub export_format: Option<String>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
}
