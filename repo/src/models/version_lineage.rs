use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Serialize;
use uuid::Uuid;

use crate::schema::version_lineage;

#[derive(Debug, Queryable, Identifiable, Selectable)]
#[diesel(table_name = version_lineage)]
pub struct VersionLineage {
    pub id: Uuid,
    pub child_version_id: Uuid,
    pub parent_version_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = version_lineage)]
pub struct NewVersionLineage {
    pub child_version_id: Uuid,
    pub parent_version_id: Uuid,
}

#[derive(Serialize)]
pub struct LineageResponse {
    pub id: Uuid,
    pub child_version_id: Uuid,
    pub parent_version_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<VersionLineage> for LineageResponse {
    fn from(l: VersionLineage) -> Self {
        Self {
            id: l.id,
            child_version_id: l.child_version_id,
            parent_version_id: l.parent_version_id,
            created_at: l.created_at,
        }
    }
}
