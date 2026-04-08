use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::notification_templates;

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = notification_templates)]
pub struct NotificationTemplate {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub subject_template: String,
    pub body_template: String,
    pub category: String,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = notification_templates)]
pub struct NewNotificationTemplate {
    pub code: String,
    pub name: String,
    pub subject_template: String,
    pub body_template: String,
    pub category: String,
    pub created_by: Uuid,
}

#[derive(AsChangeset)]
#[diesel(table_name = notification_templates)]
pub struct UpdateNotificationTemplate {
    pub name: Option<String>,
    pub subject_template: Option<String>,
    pub body_template: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct NotificationTemplateResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub subject_template: String,
    pub body_template: String,
    pub category: String,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<NotificationTemplate> for NotificationTemplateResponse {
    fn from(t: NotificationTemplate) -> Self {
        Self {
            id: t.id,
            code: t.code,
            name: t.name,
            subject_template: t.subject_template,
            body_template: t.body_template,
            category: t.category,
            is_active: t.is_active,
            created_by: t.created_by,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct CreateTemplateRequest {
    #[validate(length(min = 1, max = 128))]
    pub code: String,
    #[validate(length(min = 1, max = 256))]
    pub name: String,
    #[validate(length(min = 1, max = 512))]
    pub subject_template: String,
    #[validate(length(min = 1))]
    pub body_template: String,
    #[serde(default = "default_category")]
    pub category: String,
}

fn default_category() -> String {
    "general".into()
}

#[derive(Deserialize, Validate)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub subject_template: Option<String>,
    pub body_template: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
}
