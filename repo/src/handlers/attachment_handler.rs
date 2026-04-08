use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use futures_util::StreamExt;
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::file_attachment::*;
use crate::rbac::guard::check_permission;
use crate::schema::file_attachments;
use crate::storage;

pub async fn upload(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    mut payload: Multipart,
) -> Result<HttpResponse, AppError> {
    let participant_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "participant.attach", &mut conn)?;

    // Verify participant exists
    use crate::schema::participants;
    let _: crate::models::participant::Participant = participants::table
        .find(participant_id)
        .select(crate::models::participant::Participant::as_select())
        .first(&mut conn)?;

    let mut uploaded = Vec::new();

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| AppError::Validation(format!("Multipart error: {}", e)))?;

        let content_disposition = field
            .content_disposition()
            .ok_or_else(|| AppError::Validation("Missing content disposition".into()))?
            .clone();
        let original_filename = content_disposition
            .get_filename()
            .ok_or_else(|| AppError::Validation("Missing filename in upload".into()))?
            .to_string();

        // Infer and validate content type from extension
        let content_type = storage::content_type_from_filename(&original_filename)?;
        storage::validate_content_type(&content_type)?;

        // Read all bytes
        let mut data = Vec::new();
        while let Some(chunk) = field.next().await {
            let chunk =
                chunk.map_err(|e| AppError::Internal(format!("Read chunk error: {}", e)))?;
            data.extend_from_slice(&chunk);

            // Early size check
            if data.len() as u64 > MAX_FILE_SIZE {
                return Err(AppError::Validation(format!(
                    "File '{}' exceeds 10 MB limit",
                    original_filename
                )));
            }
        }

        storage::validate_file_size(data.len() as u64)?;

        // Save to disk
        let (disk_path, sha256) = storage::save_file(participant_id, &original_filename, &data)?;

        // Check for duplicate by hash
        let existing: Option<FileAttachment> = file_attachments::table
            .filter(file_attachments::participant_id.eq(participant_id))
            .filter(file_attachments::sha256_hash.eq(&sha256))
            .select(FileAttachment::as_select())
            .first(&mut conn)
            .optional()?;

        if let Some(dup) = existing {
            // Remove the just-written file since it's a duplicate
            storage::delete_file(&disk_path)?;
            return Err(AppError::Conflict(format!(
                "Duplicate file detected (SHA-256 match with attachment {})",
                dup.id
            )));
        }

        let new_attachment = NewFileAttachment {
            participant_id,
            file_name: original_filename,
            file_path: disk_path,
            content_type,
            file_size_bytes: data.len() as i64,
            sha256_hash: sha256,
            uploaded_by: auth.0.sub,
        };

        let attachment: FileAttachment = diesel::insert_into(file_attachments::table)
            .values(&new_attachment)
            .get_result(&mut conn)?;

        uploaded.push(FileAttachmentResponse::from(attachment));
    }

    if uploaded.is_empty() {
        return Err(AppError::Validation("No files uploaded".into()));
    }

    Ok(HttpResponse::Created().json(uploaded))
}

pub async fn list(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let participant_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_permission(&auth.0, "participant.read", &mut conn)?;
    let p: crate::models::participant::Participant = crate::schema::participants::table
        .find(participant_id).select(crate::models::participant::Participant::as_select()).first(&mut conn)?;
    ctx.enforce_scope(p.created_by, p.department.as_deref(), p.location.as_deref())?;

    let attachments: Vec<FileAttachment> = file_attachments::table
        .filter(file_attachments::participant_id.eq(participant_id))
        .select(FileAttachment::as_select())
        .order(file_attachments::created_at.desc())
        .load(&mut conn)?;

    let responses: Vec<FileAttachmentResponse> =
        attachments.into_iter().map(FileAttachmentResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn download(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, AppError> {
    let (participant_id, attachment_id) = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_permission(&auth.0, "participant.read", &mut conn)?;
    let p: crate::models::participant::Participant = crate::schema::participants::table
        .find(participant_id).select(crate::models::participant::Participant::as_select()).first(&mut conn)?;
    ctx.enforce_scope(p.created_by, p.department.as_deref(), p.location.as_deref())?;

    let attachment: FileAttachment = file_attachments::table
        .filter(file_attachments::id.eq(attachment_id))
        .filter(file_attachments::participant_id.eq(participant_id))
        .select(FileAttachment::as_select())
        .first(&mut conn)?;

    let data = storage::read_file(&attachment.file_path)?;

    Ok(HttpResponse::Ok()
        .content_type(attachment.content_type)
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", attachment.file_name),
        ))
        .body(data))
}

pub async fn delete(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, AppError> {
    let (participant_id, attachment_id) = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_permission(&auth.0, "participant.attach", &mut conn)?;
    let p: crate::models::participant::Participant = crate::schema::participants::table
        .find(participant_id).select(crate::models::participant::Participant::as_select()).first(&mut conn)?;
    ctx.enforce_scope(p.created_by, p.department.as_deref(), p.location.as_deref())?;

    let attachment: FileAttachment = file_attachments::table
        .filter(file_attachments::id.eq(attachment_id))
        .filter(file_attachments::participant_id.eq(participant_id))
        .select(FileAttachment::as_select())
        .first(&mut conn)?;

    // Delete from disk
    storage::delete_file(&attachment.file_path)?;

    // Delete from DB
    diesel::delete(file_attachments::table.find(attachment_id)).execute(&mut conn)?;

    Ok(HttpResponse::NoContent().finish())
}
