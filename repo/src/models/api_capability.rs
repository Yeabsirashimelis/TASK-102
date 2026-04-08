use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::api_capabilities;

#[derive(Debug, Queryable, Identifiable, Selectable)]
#[diesel(table_name = api_capabilities)]
pub struct ApiCapability {
    pub id: Uuid,
    pub permission_point_id: Uuid,
    pub http_method: String,
    pub path_pattern: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = api_capabilities)]
pub struct NewApiCapability {
    pub permission_point_id: Uuid,
    pub http_method: String,
    pub path_pattern: String,
    pub description: Option<String>,
}

#[derive(AsChangeset)]
#[diesel(table_name = api_capabilities)]
pub struct UpdateApiCapability {
    pub permission_point_id: Option<Uuid>,
    pub http_method: Option<String>,
    pub path_pattern: Option<String>,
    pub description: Option<Option<String>>,
}

#[derive(Serialize)]
pub struct ApiCapabilityResponse {
    pub id: Uuid,
    pub permission_point_id: Uuid,
    pub http_method: String,
    pub path_pattern: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<ApiCapability> for ApiCapabilityResponse {
    fn from(a: ApiCapability) -> Self {
        Self {
            id: a.id,
            permission_point_id: a.permission_point_id,
            http_method: a.http_method,
            path_pattern: a.path_pattern,
            description: a.description,
            created_at: a.created_at,
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct CreateApiCapabilityRequest {
    pub permission_point_id: Uuid,
    #[validate(length(min = 1, max = 10))]
    pub http_method: String,
    #[validate(length(min = 1, max = 512))]
    pub path_pattern: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct UpdateApiCapabilityRequest {
    pub permission_point_id: Option<Uuid>,
    pub http_method: Option<String>,
    pub path_pattern: Option<String>,
    pub description: Option<Option<String>>,
}
