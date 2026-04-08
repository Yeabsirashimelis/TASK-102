use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::roles;

#[derive(Debug, Clone, diesel_derive_enum::DbEnum, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[ExistingTypePath = "crate::schema::sql_types::DataScopeEnum"]
pub enum DataScope {
    #[db_rename = "department"]
    Department,
    #[db_rename = "location"]
    Location,
    #[db_rename = "individual"]
    Individual,
}

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = roles)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub data_scope: DataScope,
    pub scope_value: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = roles)]
pub struct NewRole {
    pub name: String,
    pub description: Option<String>,
    pub data_scope: DataScope,
    pub scope_value: Option<String>,
}

#[derive(AsChangeset)]
#[diesel(table_name = roles)]
pub struct UpdateRole {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub data_scope: Option<DataScope>,
    pub scope_value: Option<Option<String>>,
    pub is_active: Option<bool>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct RoleResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub data_scope: DataScope,
    pub scope_value: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Role> for RoleResponse {
    fn from(r: Role) -> Self {
        Self {
            id: r.id,
            name: r.name,
            description: r.description,
            data_scope: r.data_scope,
            scope_value: r.scope_value,
            is_active: r.is_active,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct CreateRoleRequest {
    #[validate(length(min = 1, max = 128))]
    pub name: String,
    pub description: Option<String>,
    pub data_scope: DataScope,
    pub scope_value: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct UpdateRoleRequest {
    #[validate(length(min = 1, max = 128))]
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub data_scope: Option<DataScope>,
    pub scope_value: Option<Option<String>>,
    pub is_active: Option<bool>,
}
