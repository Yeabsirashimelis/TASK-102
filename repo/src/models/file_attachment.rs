use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Serialize;
use uuid::Uuid;

use crate::schema::file_attachments;

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = file_attachments)]
pub struct FileAttachment {
    pub id: Uuid,
    pub participant_id: Uuid,
    pub file_name: String,
    pub file_path: String,
    pub content_type: String,
    pub file_size_bytes: i64,
    pub sha256_hash: String,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = file_attachments)]
pub struct NewFileAttachment {
    pub participant_id: Uuid,
    pub file_name: String,
    pub file_path: String,
    pub content_type: String,
    pub file_size_bytes: i64,
    pub sha256_hash: String,
    pub uploaded_by: Uuid,
}

#[derive(Serialize)]
pub struct FileAttachmentResponse {
    pub id: Uuid,
    pub participant_id: Uuid,
    pub file_name: String,
    pub content_type: String,
    pub file_size_bytes: i64,
    pub sha256_hash: String,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<FileAttachment> for FileAttachmentResponse {
    fn from(f: FileAttachment) -> Self {
        Self {
            id: f.id,
            participant_id: f.participant_id,
            file_name: f.file_name,
            content_type: f.content_type,
            file_size_bytes: f.file_size_bytes,
            sha256_hash: f.sha256_hash,
            uploaded_by: f.uploaded_by,
            created_at: f.created_at,
        }
    }
}

/// Allowed MIME types for file uploads.
pub const ALLOWED_CONTENT_TYPES: &[&str] = &[
    "application/pdf",
    "image/jpeg",
    "image/png",
    "text/csv",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
];

/// Max file size: 10 MB
pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
