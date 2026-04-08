use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::dataset_versions;

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = dataset_versions)]
pub struct DatasetVersion {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub version_number: i32,
    pub storage_path: String,
    pub file_size_bytes: Option<i64>,
    pub sha256_hash: Option<String>,
    pub row_count: Option<i64>,
    pub transformation_note: Option<String>,
    pub is_current: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = dataset_versions)]
pub struct NewDatasetVersion {
    pub dataset_id: Uuid,
    pub version_number: i32,
    pub storage_path: String,
    pub file_size_bytes: Option<i64>,
    pub sha256_hash: Option<String>,
    pub row_count: Option<i64>,
    pub transformation_note: Option<String>,
    pub is_current: bool,
    pub created_by: Uuid,
}

#[derive(Serialize)]
pub struct DatasetVersionResponse {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub version_number: i32,
    pub storage_path: String,
    pub file_size_bytes: Option<i64>,
    pub sha256_hash: Option<String>,
    pub row_count: Option<i64>,
    pub transformation_note: Option<String>,
    pub is_current: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_version_ids: Option<Vec<Uuid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_dictionary: Option<Vec<super::field_dictionary::FieldDictionaryResponse>>,
}

impl From<DatasetVersion> for DatasetVersionResponse {
    fn from(v: DatasetVersion) -> Self {
        Self {
            id: v.id,
            dataset_id: v.dataset_id,
            version_number: v.version_number,
            storage_path: v.storage_path,
            file_size_bytes: v.file_size_bytes,
            sha256_hash: v.sha256_hash,
            row_count: v.row_count,
            transformation_note: v.transformation_note,
            is_current: v.is_current,
            created_by: v.created_by,
            created_at: v.created_at,
            parent_version_ids: None,
            field_dictionary: None,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateVersionRequest {
    pub storage_path: String,
    pub file_size_bytes: Option<i64>,
    pub sha256_hash: Option<String>,
    pub row_count: Option<i64>,
    pub transformation_note: Option<String>,
    #[serde(default)]
    pub parent_version_ids: Vec<Uuid>,
    #[serde(default)]
    pub field_dictionary: Vec<super::field_dictionary::FieldDictionaryInput>,
}

#[derive(Deserialize)]
pub struct VersionQueryParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Deserialize)]
pub struct RollbackRequest {
    pub target_version_id: Uuid,
    pub note: Option<String>,
}
