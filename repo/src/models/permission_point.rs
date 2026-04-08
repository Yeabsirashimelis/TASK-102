use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::permission_points;

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = permission_points)]
pub struct PermissionPoint {
    pub id: Uuid,
    pub code: String,
    pub description: Option<String>,
    pub requires_approval: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = permission_points)]
pub struct NewPermissionPoint {
    pub code: String,
    pub description: Option<String>,
    pub requires_approval: bool,
}

#[derive(AsChangeset)]
#[diesel(table_name = permission_points)]
pub struct UpdatePermissionPoint {
    pub code: Option<String>,
    pub description: Option<Option<String>>,
    pub requires_approval: Option<bool>,
}

#[derive(Serialize)]
pub struct PermissionPointResponse {
    pub id: Uuid,
    pub code: String,
    pub description: Option<String>,
    pub requires_approval: bool,
    pub created_at: DateTime<Utc>,
}

impl From<PermissionPoint> for PermissionPointResponse {
    fn from(p: PermissionPoint) -> Self {
        Self {
            id: p.id,
            code: p.code,
            description: p.description,
            requires_approval: p.requires_approval,
            created_at: p.created_at,
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct CreatePermissionPointRequest {
    #[validate(length(min = 1, max = 128))]
    pub code: String,
    pub description: Option<String>,
    #[serde(default)]
    pub requires_approval: bool,
}

#[derive(Deserialize, Validate)]
pub struct UpdatePermissionPointRequest {
    #[validate(length(min = 1, max = 128))]
    pub code: Option<String>,
    pub description: Option<Option<String>>,
    pub requires_approval: Option<bool>,
}
