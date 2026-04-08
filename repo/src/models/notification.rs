use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::notifications;

#[derive(Debug, Clone, PartialEq, diesel_derive_enum::DbEnum, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::NotificationStatusType"]
pub enum NotificationStatus {
    #[db_rename = "pending"]
    Pending,
    #[db_rename = "delivered"]
    Delivered,
    #[db_rename = "read"]
    Read,
    #[db_rename = "failed"]
    Failed,
}

#[derive(Debug, Clone, PartialEq, diesel_derive_enum::DbEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[ExistingTypePath = "crate::schema::sql_types::NotificationCategoryType"]
pub enum NotificationCategory {
    #[db_rename = "moderation_outcome"]
    ModerationOutcome,
    #[db_rename = "comment_reply"]
    CommentReply,
    #[db_rename = "system_announcement"]
    SystemAnnouncement,
    #[db_rename = "general"]
    General,
}

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = notifications)]
pub struct Notification {
    pub id: Uuid,
    pub recipient_user_id: Uuid,
    pub template_id: Option<Uuid>,
    pub category: NotificationCategory,
    pub subject: String,
    pub body: String,
    pub status: NotificationStatus,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = notifications)]
pub struct NewNotification {
    pub recipient_user_id: Uuid,
    pub template_id: Option<Uuid>,
    pub category: NotificationCategory,
    pub subject: String,
    pub body: String,
    pub status: NotificationStatus,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub recipient_user_id: Uuid,
    pub template_id: Option<Uuid>,
    pub category: NotificationCategory,
    pub subject: String,
    pub body: String,
    pub status: NotificationStatus,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<Notification> for NotificationResponse {
    fn from(n: Notification) -> Self {
        Self {
            id: n.id,
            recipient_user_id: n.recipient_user_id,
            template_id: n.template_id,
            category: n.category,
            subject: n.subject,
            body: n.body,
            status: n.status,
            reference_type: n.reference_type,
            reference_id: n.reference_id,
            read_at: n.read_at,
            created_at: n.created_at,
        }
    }
}

/// Send notification using a template with variable substitution.
#[derive(Deserialize)]
pub struct SendNotificationRequest {
    pub recipient_user_id: Uuid,
    pub template_code: String,
    pub variables: std::collections::HashMap<String, String>,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
}

/// Send a direct notification without a template.
#[derive(Deserialize)]
pub struct SendDirectNotificationRequest {
    pub recipient_user_id: Uuid,
    pub category: NotificationCategory,
    pub subject: String,
    pub body: String,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
}

/// Broadcast system announcement to all active users.
#[derive(Deserialize)]
pub struct BroadcastRequest {
    pub subject: String,
    pub body: String,
}

#[derive(Deserialize)]
pub struct NotificationQueryParams {
    pub status: Option<String>,
    pub category: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
