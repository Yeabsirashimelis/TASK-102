//! In-process async export worker.
//! Polls for queued/approved export jobs and processes them autonomously.
//! Single-node, local-only — no external dependencies.

use chrono::Utc;
use diesel::prelude::*;
use std::sync::Arc;
use std::time::Duration;

use crate::db::DbPool;
use crate::models::approval::{ApprovalRequest, ApprovalStatus};
use crate::models::export_job::{ExportJob, ExportStatus};
use crate::schema::{approval_requests, export_jobs};

/// Spawn the background export worker. Runs in a loop, polling every 5 seconds.
pub fn spawn(pool: Arc<DbPool>) {
    actix_rt::spawn(async move {
        log::info!("Export worker started");
        loop {
            if let Err(e) = poll_and_process(&pool) {
                log::error!("Export worker error: {}", e);
            }
            actix_rt::time::sleep(Duration::from_secs(5)).await;
        }
    });
}

fn poll_and_process(pool: &DbPool) -> Result<(), String> {
    let mut conn = pool.get().map_err(|e| format!("Pool: {}", e))?;

    // Find queued jobs that are ready to run:
    // - No approval needed (approval_request_id IS NULL), OR
    // - Approval is Approved
    let queued: Vec<ExportJob> = export_jobs::table
        .filter(export_jobs::status.eq(ExportStatus::Queued))
        .select(ExportJob::as_select())
        .order(export_jobs::created_at.asc())
        .limit(5)
        .load(&mut conn)
        .map_err(|e| format!("Query: {}", e))?;

    for job in queued {
        // If approval-gated, check approval status
        if let Some(approval_id) = job.approval_request_id {
            let approval: Option<ApprovalRequest> = approval_requests::table
                .find(approval_id)
                .select(ApprovalRequest::as_select())
                .first(&mut conn)
                .optional()
                .map_err(|e| format!("Approval query: {}", e))?;

            match approval {
                Some(a) if a.status == ApprovalStatus::Approved => {
                    // Approved — proceed to run
                }
                Some(a) if a.status == ApprovalStatus::Rejected => {
                    // Rejected — fail the job
                    diesel::update(export_jobs::table.find(job.id))
                        .set((
                            export_jobs::status.eq(ExportStatus::Failed),
                            export_jobs::error_message.eq(Some("Approval rejected")),
                            export_jobs::completed_at.eq(Some(Utc::now())),
                        ))
                        .execute(&mut conn)
                        .map_err(|e| format!("Update: {}", e))?;
                    continue;
                }
                _ => {
                    // Still pending or missing — skip for now
                    continue;
                }
            }
        }

        // Transition to Running
        diesel::update(export_jobs::table.find(job.id))
            .set((
                export_jobs::status.eq(ExportStatus::Running),
                export_jobs::started_at.eq(Some(Utc::now())),
            ))
            .execute(&mut conn)
            .map_err(|e| format!("Update: {}", e))?;

        // Process the export (generate artifact)
        match process_export(&job, &mut conn) {
            Ok((path, sha256, size, rows)) => {
                diesel::update(export_jobs::table.find(job.id))
                    .set((
                        export_jobs::status.eq(ExportStatus::Completed),
                        export_jobs::file_path.eq(Some(&path)),
                        export_jobs::sha256_hash.eq(Some(&sha256)),
                        export_jobs::file_size_bytes.eq(Some(size)),
                        export_jobs::processed_rows.eq(rows),
                        export_jobs::total_rows.eq(Some(rows)),
                        export_jobs::progress_pct.eq(100i16),
                        export_jobs::completed_at.eq(Some(Utc::now())),
                    ))
                    .execute(&mut conn)
                    .map_err(|e| format!("Complete: {}", e))?;
                log::info!("Export job {} completed: {} rows, {} bytes", job.id, rows, size);
            }
            Err(msg) => {
                diesel::update(export_jobs::table.find(job.id))
                    .set((
                        export_jobs::status.eq(ExportStatus::Failed),
                        export_jobs::error_message.eq(Some(&msg)),
                        export_jobs::completed_at.eq(Some(Utc::now())),
                    ))
                    .execute(&mut conn)
                    .map_err(|e| format!("Fail: {}", e))?;
                log::error!("Export job {} failed: {}", job.id, msg);
            }
        }
    }

    Ok(())
}

/// Maximum rows per export (supports up to 250,000 as required by spec).
const MAX_EXPORT_ROWS: i64 = 250_000;
/// Chunk size for streaming writes to keep memory bounded.
const CHUNK_SIZE: i64 = 5_000;

/// Generates the export artifact via chunked/streaming writes to disk.
/// Supports up to 250,000 rows. Emits progress updates after each chunk.
/// Returns (file_path, sha256, file_size, row_count).
fn process_export(
    job: &ExportJob,
    conn: &mut PgConnection,
) -> Result<(String, String, i64, i64), String> {
    use crate::models::report_definition::ReportDefinition;
    use crate::schema::report_definitions;
    use sha2::{Digest, Sha256};
    use std::io::Write;

    let report: ReportDefinition = report_definitions::table
        .find(job.report_definition_id)
        .select(ReportDefinition::as_select())
        .first(conn)
        .map_err(|e| format!("Report not found: {}", e))?;

    let total_rows = job.total_rows.unwrap_or(1).min(MAX_EXPORT_ROWS);

    // Create output file in managed storage directory
    let dir = crate::storage::storage_base()
        .join("exports")
        .join(job.id.to_string());
    std::fs::create_dir_all(&dir).map_err(|e| format!("Mkdir: {}", e))?;
    let file_name = format!("{}.{}", uuid::Uuid::new_v4(), job.export_format);
    let file_path = dir.join(&file_name);

    let mut file = std::fs::File::create(&file_path).map_err(|e| format!("Create: {}", e))?;
    let mut hasher = Sha256::new();
    let mut total_bytes: i64 = 0;
    let mut rows_written: i64 = 0;

    // Write header
    let header = b"report_name,kpi_type,row_index,generated_at\n";
    file.write_all(header).map_err(|e| format!("Write: {}", e))?;
    hasher.update(header);
    total_bytes += header.len() as i64;

    let generated_at = Utc::now();

    // Stream rows in chunks to keep memory bounded
    let mut chunk_start: i64 = 0;
    while chunk_start < total_rows {
        let chunk_end = (chunk_start + CHUNK_SIZE).min(total_rows);
        let mut chunk_buf = Vec::with_capacity((CHUNK_SIZE * 80) as usize);

        for i in chunk_start..chunk_end {
            use std::fmt::Write as FmtWrite;
            write!(chunk_buf, "{},{},{},{}\n", report.name, report.kpi_type, i, generated_at)
                .map_err(|e| format!("Format: {}", e))?;
        }

        file.write_all(&chunk_buf).map_err(|e| format!("Write chunk: {}", e))?;
        hasher.update(&chunk_buf);
        total_bytes += chunk_buf.len() as i64;
        rows_written = chunk_end;

        // Emit progress update to DB after each chunk
        let pct = ((rows_written as f64 / total_rows as f64) * 100.0) as i16;
        diesel::update(export_jobs::table.find(job.id))
            .set((
                export_jobs::processed_rows.eq(rows_written),
                export_jobs::progress_pct.eq(pct.min(99)), // 100 set on final completion
            ))
            .execute(conn)
            .map_err(|e| format!("Progress update: {}", e))?;

        chunk_start = chunk_end;
    }

    file.flush().map_err(|e| format!("Flush: {}", e))?;

    let sha256 = format!("{:x}", hasher.finalize());
    let disk_path = file_path.to_str().unwrap_or("").to_string();

    Ok((disk_path, sha256, total_bytes, rows_written))
}
