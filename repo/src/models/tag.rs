use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::{participant_tags, tags};

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = tags)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = tags)]
pub struct NewTag {
    pub name: String,
}

#[derive(Serialize)]
pub struct TagResponse {
    pub id: Uuid,
    pub name: String,
}

impl From<Tag> for TagResponse {
    fn from(t: Tag) -> Self {
        Self {
            id: t.id,
            name: t.name,
        }
    }
}

#[derive(Debug, Queryable, Identifiable, Selectable)]
#[diesel(table_name = participant_tags)]
pub struct ParticipantTag {
    pub id: Uuid,
    pub participant_id: Uuid,
    pub tag_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = participant_tags)]
pub struct NewParticipantTag {
    pub participant_id: Uuid,
    pub tag_id: Uuid,
}

#[derive(Deserialize)]
pub struct SetTagsRequest {
    pub tags: Vec<String>,
}
