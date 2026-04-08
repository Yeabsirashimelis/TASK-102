use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::login_attempts;

#[derive(Debug, Queryable, Identifiable, Selectable)]
#[diesel(table_name = login_attempts)]
pub struct LoginAttempt {
    pub id: Uuid,
    pub username: String,
    pub success: bool,
    pub ip_address: Option<String>,
    pub attempted_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = login_attempts)]
pub struct NewLoginAttempt {
    pub username: String,
    pub success: bool,
    pub ip_address: Option<String>,
}
