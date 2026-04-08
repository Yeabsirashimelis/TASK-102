use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::menu_scopes;

#[derive(Debug, Queryable, Identifiable, Selectable)]
#[diesel(table_name = menu_scopes)]
pub struct MenuScope {
    pub id: Uuid,
    pub permission_point_id: Uuid,
    pub menu_key: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = menu_scopes)]
pub struct NewMenuScope {
    pub permission_point_id: Uuid,
    pub menu_key: String,
    pub description: Option<String>,
}

#[derive(AsChangeset)]
#[diesel(table_name = menu_scopes)]
pub struct UpdateMenuScope {
    pub permission_point_id: Option<Uuid>,
    pub menu_key: Option<String>,
    pub description: Option<Option<String>>,
}

#[derive(Serialize)]
pub struct MenuScopeResponse {
    pub id: Uuid,
    pub permission_point_id: Uuid,
    pub menu_key: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<MenuScope> for MenuScopeResponse {
    fn from(m: MenuScope) -> Self {
        Self {
            id: m.id,
            permission_point_id: m.permission_point_id,
            menu_key: m.menu_key,
            description: m.description,
            created_at: m.created_at,
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct CreateMenuScopeRequest {
    pub permission_point_id: Uuid,
    #[validate(length(min = 1, max = 256))]
    pub menu_key: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct UpdateMenuScopeRequest {
    pub permission_point_id: Option<Uuid>,
    pub menu_key: Option<String>,
    pub description: Option<Option<String>>,
}
