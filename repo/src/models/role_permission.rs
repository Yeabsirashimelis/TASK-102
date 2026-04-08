use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::role_permissions;

#[derive(Debug, Queryable, Identifiable, Selectable)]
#[diesel(table_name = role_permissions)]
pub struct RolePermission {
    pub id: Uuid,
    pub role_id: Uuid,
    pub permission_point_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = role_permissions)]
pub struct NewRolePermission {
    pub role_id: Uuid,
    pub permission_point_id: Uuid,
}

#[derive(Serialize)]
pub struct RolePermissionResponse {
    pub id: Uuid,
    pub role_id: Uuid,
    pub permission_point_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<RolePermission> for RolePermissionResponse {
    fn from(rp: RolePermission) -> Self {
        Self {
            id: rp.id,
            role_id: rp.role_id,
            permission_point_id: rp.permission_point_id,
            created_at: rp.created_at,
        }
    }
}

#[derive(Deserialize)]
pub struct BindPermissionRequest {
    pub permission_point_id: Uuid,
}
