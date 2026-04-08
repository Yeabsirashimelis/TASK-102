use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::crypto::mask_sensitive;
use crate::schema::users;

#[derive(Debug, Queryable, Identifiable, Selectable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash_enc: Vec<u8>,
    pub gov_id_enc: Option<Vec<u8>>,
    pub gov_id_last4: Option<String>,
    pub role_id: Uuid,
    pub department: Option<String>,
    pub location: Option<String>,
    pub is_active: bool,
    pub failed_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub password_hash_enc: Vec<u8>,
    pub gov_id_enc: Option<Vec<u8>>,
    pub gov_id_last4: Option<String>,
    pub role_id: Uuid,
    pub department: Option<String>,
    pub location: Option<String>,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub gov_id_display: Option<String>,
    pub role_id: Uuid,
    pub department: Option<String>,
    pub location: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            gov_id_display: u.gov_id_last4.map(|v| mask_sensitive(&v)),
            role_id: u.role_id,
            department: u.department,
            location: u.location,
            is_active: u.is_active,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 128))]
    pub username: String,
    #[validate(custom(function = "crate::auth::password::validate_password"))]
    pub password: String,
    pub gov_id: Option<String>,
    pub role_id: Uuid,
    pub department: Option<String>,
    pub location: Option<String>,
}
