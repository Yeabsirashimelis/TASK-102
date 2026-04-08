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

/// Generates the export artifact content based on the report definition.
/// Returns (file_path, sha256, file_size, row_count).
fn process_export(
    job: &ExportJob,
    conn: &mut PgConnection,
) -> Result<(String, String, i64, i64), String> {
    use crate::models::report_definition::ReportDefinition;
    use crate::schema::report_definitions;

    // Load the report definition
    let report: ReportDefinition = report_definitions::table
        .find(job.report_definition_id)
        .select(ReportDefinition::as_select())
        .first(conn)
        .map_err(|e| format!("Report not found: {}", e))?;

    // Generate CSV content based on report metadata
    let header = format!("report_name,kpi_type,generated_at\n");
    let row = format!("{},{},{}\n", report.name, report.kpi_type, Utc::now());
    let estimated = job.total_rows.unwrap_or(1);
    let mut content = header;
    for i in 0..estimated.min(1000) {
        content.push_str(&format!("{},{},row_{}\n", report.name, report.kpi_type, i));
    }
    let data = content.as_bytes();
    let row_count = estimated.min(1000) + 1; // header + data rows

    // Store via managed storage
    let (path, sha256) = crate::storage::save_artifact(
        "exports",
        job.id,
        &job.export_format,
        data,
    )
    .map_err(|e| format!("Storage: {}", e))?;

    Ok((path, sha256, data.len() as i64, row_count))
}
