use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::field_dictionaries;

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = field_dictionaries)]
pub struct FieldDictionary {
    pub id: Uuid,
    pub version_id: Uuid,
    pub field_name: String,
    pub field_type: String,
    pub meaning: Option<String>,
    pub source_system: Option<String>,
    pub last_updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = field_dictionaries)]
pub struct NewFieldDictionary {
    pub version_id: Uuid,
    pub field_name: String,
    pub field_type: String,
    pub meaning: Option<String>,
    pub source_system: Option<String>,
}

#[derive(AsChangeset)]
#[diesel(table_name = field_dictionaries)]
pub struct UpdateFieldDictionary {
    pub field_type: Option<String>,
    pub meaning: Option<Option<String>>,
    pub source_system: Option<Option<String>>,
    pub last_updated_at: DateTime<Utc>,
}

#[derive(Serialize, Clone)]
pub struct FieldDictionaryResponse {
    pub id: Uuid,
    pub version_id: Uuid,
    pub field_name: String,
    pub field_type: String,
    pub meaning: Option<String>,
    pub source_system: Option<String>,
    pub last_updated_at: DateTime<Utc>,
}

impl From<FieldDictionary> for FieldDictionaryResponse {
    fn from(f: FieldDictionary) -> Self {
        Self {
            id: f.id,
            version_id: f.version_id,
            field_name: f.field_name,
            field_type: f.field_type,
            meaning: f.meaning,
            source_system: f.source_system,
            last_updated_at: f.last_updated_at,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct FieldDictionaryInput {
    pub field_name: String,
    pub field_type: String,
    pub meaning: Option<String>,
    pub source_system: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct UpdateFieldDictionaryRequest {
    pub field_type: Option<String>,
    pub meaning: Option<Option<String>>,
    pub source_system: Option<Option<String>>,
}
