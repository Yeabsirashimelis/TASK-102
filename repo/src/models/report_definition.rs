use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::report_definitions;

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = report_definitions)]
pub struct ReportDefinition {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub kpi_type: String,
    pub dimensions: serde_json::Value,
    pub filters: serde_json::Value,
    pub chart_config: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = report_definitions)]
pub struct NewReportDefinition {
    pub name: String,
    pub description: Option<String>,
    pub kpi_type: String,
    pub dimensions: serde_json::Value,
    pub filters: serde_json::Value,
    pub chart_config: Option<serde_json::Value>,
    pub created_by: Uuid,
}

#[derive(AsChangeset)]
#[diesel(table_name = report_definitions)]
pub struct UpdateReportDefinition {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub kpi_type: Option<String>,
    pub dimensions: Option<serde_json::Value>,
    pub filters: Option<serde_json::Value>,
    pub chart_config: Option<Option<serde_json::Value>>,
    pub is_active: Option<bool>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ReportDefinitionResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub kpi_type: String,
    pub dimensions: serde_json::Value,
    pub filters: serde_json::Value,
    pub chart_config: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ReportDefinition> for ReportDefinitionResponse {
    fn from(r: ReportDefinition) -> Self {
        Self {
            id: r.id,
            name: r.name,
            description: r.description,
            kpi_type: r.kpi_type,
            dimensions: r.dimensions,
            filters: r.filters,
            chart_config: r.chart_config,
            is_active: r.is_active,
            created_by: r.created_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct CreateReportDefinitionRequest {
    #[validate(length(min = 1, max = 256))]
    pub name: String,
    pub description: Option<String>,
    #[validate(length(min = 1, max = 128))]
    pub kpi_type: String,
    pub dimensions: serde_json::Value,
    #[serde(default = "default_filters")]
    pub filters: serde_json::Value,
    pub chart_config: Option<serde_json::Value>,
}

fn default_filters() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Deserialize, Validate)]
pub struct UpdateReportDefinitionRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub kpi_type: Option<String>,
    pub dimensions: Option<serde_json::Value>,
    pub filters: Option<serde_json::Value>,
    pub chart_config: Option<Option<serde_json::Value>>,
    pub is_active: Option<bool>,
}

/// Query parameters for executing a report KPI query.
#[derive(Deserialize)]
pub struct RunReportRequest {
    #[serde(default = "default_filters")]
    pub filters: serde_json::Value,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
}

/// Supported KPI types for analytics queries.
pub const KPI_TYPES: &[&str] = &[
    "registration_conversion",
    "participation_by_store",
    "participation_by_department",
    "project_milestones",
    "review_efficiency",
    "award_distribution",
];
