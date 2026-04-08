use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::datasets;

#[derive(Debug, Clone, PartialEq, diesel_derive_enum::DbEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[ExistingTypePath = "crate::schema::sql_types::DatasetTypeType"]
pub enum DatasetType {
    #[db_rename = "raw"]
    Raw,
    #[db_rename = "cleaned"]
    Cleaned,
    #[db_rename = "feature"]
    Feature,
    #[db_rename = "result"]
    Result,
}

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = datasets)]
pub struct Dataset {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub dataset_type: DatasetType,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = datasets)]
pub struct NewDataset {
    pub name: String,
    pub description: Option<String>,
    pub dataset_type: DatasetType,
    pub created_by: Uuid,
}

#[derive(AsChangeset)]
#[diesel(table_name = datasets)]
pub struct UpdateDataset {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub dataset_type: Option<DatasetType>,
    pub is_active: Option<bool>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct DatasetResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub dataset_type: DatasetType,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_version: Option<i32>,
}

impl From<Dataset> for DatasetResponse {
    fn from(d: Dataset) -> Self {
        Self {
            id: d.id,
            name: d.name,
            description: d.description,
            dataset_type: d.dataset_type,
            is_active: d.is_active,
            created_by: d.created_by,
            created_at: d.created_at,
            updated_at: d.updated_at,
            current_version: None,
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct CreateDatasetRequest {
    #[validate(length(min = 1, max = 256))]
    pub name: String,
    pub description: Option<String>,
    pub dataset_type: DatasetType,
}

#[derive(Deserialize, Validate)]
pub struct UpdateDatasetRequest {
    #[validate(length(min = 1, max = 256))]
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub dataset_type: Option<DatasetType>,
    pub is_active: Option<bool>,
}

#[derive(Deserialize)]
pub struct DatasetQueryParams {
    pub dataset_type: Option<String>,
    pub is_active: Option<bool>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
