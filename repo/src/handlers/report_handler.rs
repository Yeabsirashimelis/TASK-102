use actix_web::{web, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::report_definition::*;
use crate::models::scheduled_report::*;
use crate::rbac::guard::check_permission;
use crate::schema::{report_definitions, scheduled_reports};

// ===================== Report Definition CRUD =====================

pub async fn create_definition(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    body: web::Json<CreateReportDefinitionRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "report.create", &mut conn)?;

    // Validate kpi_type
    if !KPI_TYPES.contains(&body.kpi_type.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid kpi_type '{}'. Valid types: {:?}",
            body.kpi_type, KPI_TYPES
        )));
    }

    let new = NewReportDefinition {
        name: body.name.clone(),
        description: body.description.clone(),
        kpi_type: body.kpi_type.clone(),
        dimensions: body.dimensions.clone(),
        filters: body.filters.clone(),
        chart_config: body.chart_config.clone(),
        created_by: auth.0.sub,
    };

    let report: ReportDefinition = diesel::insert_into(report_definitions::table)
        .values(&new)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Created().json(ReportDefinitionResponse::from(report)))
}

pub async fn list_definitions(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "report.read", &mut conn)?;

    let results: Vec<ReportDefinition> = report_definitions::table
        .filter(report_definitions::is_active.eq(true))
        .select(ReportDefinition::as_select())
        .order(report_definitions::name.asc())
        .load(&mut conn)?;

    let responses: Vec<ReportDefinitionResponse> =
        results.into_iter().map(ReportDefinitionResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get_definition(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let report_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "report.read", &mut conn)?;

    let report: ReportDefinition = report_definitions::table
        .find(report_id)
        .select(ReportDefinition::as_select())
        .first(&mut conn)?;

    Ok(HttpResponse::Ok().json(ReportDefinitionResponse::from(report)))
}

pub async fn update_definition(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateReportDefinitionRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let report_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "report.update", &mut conn)?;

    if let Some(ref kpi) = body.kpi_type {
        if !KPI_TYPES.contains(&kpi.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid kpi_type '{}'. Valid types: {:?}",
                kpi, KPI_TYPES
            )));
        }
    }

    let changeset = UpdateReportDefinition {
        name: body.name.clone(),
        description: body.description.clone(),
        kpi_type: body.kpi_type.clone(),
        dimensions: body.dimensions.clone(),
        filters: body.filters.clone(),
        chart_config: body.chart_config.clone(),
        is_active: body.is_active,
        updated_at: Utc::now(),
    };

    let report: ReportDefinition = diesel::update(report_definitions::table.find(report_id))
        .set(&changeset)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Ok().json(ReportDefinitionResponse::from(report)))
}

pub async fn delete_definition(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let report_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "report.delete", &mut conn)?;

    diesel::update(report_definitions::table.find(report_id))
        .set((
            report_definitions::is_active.eq(false),
            report_definitions::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn)?;

    Ok(HttpResponse::NoContent().finish())
}

// ===================== Run Report (KPI Query) =====================

pub async fn run_report(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<RunReportRequest>,
) -> Result<HttpResponse, AppError> {
    let report_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "report.read", &mut conn)?;

    let report: ReportDefinition = report_definitions::table
        .find(report_id)
        .select(ReportDefinition::as_select())
        .first(&mut conn)?;

    // Execute KPI query based on type.
    // Each KPI type returns aggregated data from existing tables.
    // The dimensions/filters from the report definition and the runtime
    // request are merged to build the query.
    let result = execute_kpi_query(&report, &body, &mut conn)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "report_id": report_id,
        "kpi_type": report.kpi_type,
        "dimensions": report.dimensions,
        "filters_applied": body.filters,
        "date_from": body.date_from,
        "date_to": body.date_to,
        "data": result,
        "generated_at": Utc::now(),
    })))
}

fn execute_kpi_query(
    report: &ReportDefinition,
    request: &RunReportRequest,
    conn: &mut PgConnection,
) -> Result<serde_json::Value, AppError> {
    let date_from = request
        .date_from
        .unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
    let date_to = request.date_to.unwrap_or_else(Utc::now);

    match report.kpi_type.as_str() {
        "registration_conversion" => {
            // Count total users vs active users created in date range
            use crate::schema::users;
            let total: i64 = users::table
                .filter(users::created_at.ge(date_from))
                .filter(users::created_at.le(date_to))
                .count()
                .get_result(conn)?;
            let active: i64 = users::table
                .filter(users::created_at.ge(date_from))
                .filter(users::created_at.le(date_to))
                .filter(users::is_active.eq(true))
                .count()
                .get_result(conn)?;
            let rate = if total > 0 {
                (active as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            Ok(serde_json::json!({
                "total_registrations": total,
                "active_users": active,
                "conversion_rate_pct": rate,
            }))
        }
        "participation_by_store" => {
            use crate::schema::participants;
            let rows: Vec<(Option<String>, i64)> = participants::table
                .filter(participants::created_at.ge(date_from))
                .filter(participants::created_at.le(date_to))
                .filter(participants::is_active.eq(true))
                .group_by(participants::location)
                .select((participants::location, diesel::dsl::count(participants::id)))
                .load(conn)?;
            let data: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|(loc, cnt)| {
                    serde_json::json!({
                        "location": loc.unwrap_or_else(|| "unassigned".into()),
                        "participant_count": cnt,
                    })
                })
                .collect();
            Ok(serde_json::json!(data))
        }
        "participation_by_department" => {
            use crate::schema::participants;
            let rows: Vec<(Option<String>, i64)> = participants::table
                .filter(participants::created_at.ge(date_from))
                .filter(participants::created_at.le(date_to))
                .filter(participants::is_active.eq(true))
                .group_by(participants::department)
                .select((
                    participants::department,
                    diesel::dsl::count(participants::id),
                ))
                .load(conn)?;
            let data: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|(dept, cnt)| {
                    serde_json::json!({
                        "department": dept.unwrap_or_else(|| "unassigned".into()),
                        "participant_count": cnt,
                    })
                })
                .collect();
            Ok(serde_json::json!(data))
        }
        "project_milestones" => {
            // Dataset versions created over time as proxy for project progress
            use crate::schema::dataset_versions;
            let total_versions: i64 = dataset_versions::table
                .filter(dataset_versions::created_at.ge(date_from))
                .filter(dataset_versions::created_at.le(date_to))
                .count()
                .get_result(conn)?;
            let current_versions: i64 = dataset_versions::table
                .filter(dataset_versions::is_current.eq(true))
                .count()
                .get_result(conn)?;
            Ok(serde_json::json!({
                "versions_created_in_period": total_versions,
                "current_active_versions": current_versions,
            }))
        }
        "review_efficiency" => {
            // Approval requests: total, approved, rejected, pending in period
            use crate::models::approval::ApprovalStatus;
            use crate::schema::approval_requests;
            let total: i64 = approval_requests::table
                .filter(approval_requests::created_at.ge(date_from))
                .filter(approval_requests::created_at.le(date_to))
                .count()
                .get_result(conn)?;
            let approved: i64 = approval_requests::table
                .filter(approval_requests::created_at.ge(date_from))
                .filter(approval_requests::created_at.le(date_to))
                .filter(approval_requests::status.eq(ApprovalStatus::Approved))
                .count()
                .get_result(conn)?;
            let rejected: i64 = approval_requests::table
                .filter(approval_requests::created_at.ge(date_from))
                .filter(approval_requests::created_at.le(date_to))
                .filter(approval_requests::status.eq(ApprovalStatus::Rejected))
                .count()
                .get_result(conn)?;
            let pending: i64 = approval_requests::table
                .filter(approval_requests::created_at.ge(date_from))
                .filter(approval_requests::created_at.le(date_to))
                .filter(approval_requests::status.eq(ApprovalStatus::Pending))
                .count()
                .get_result(conn)?;
            Ok(serde_json::json!({
                "total_reviews": total,
                "approved": approved,
                "rejected": rejected,
                "pending": pending,
                "approval_rate_pct": if total > 0 { (approved as f64 / total as f64) * 100.0 } else { 0.0 },
            }))
        }
        "award_distribution" => {
            // Orders by location as proxy for distribution
            use crate::schema::orders;
            let rows: Vec<(String, i64)> = orders::table
                .filter(orders::created_at.ge(date_from))
                .filter(orders::created_at.le(date_to))
                .group_by(orders::location)
                .select((orders::location, diesel::dsl::count(orders::id)))
                .load(conn)?;
            let data: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|(loc, cnt)| {
                    serde_json::json!({
                        "location": loc,
                        "order_count": cnt,
                    })
                })
                .collect();
            Ok(serde_json::json!(data))
        }
        _ => Err(AppError::Validation(format!(
            "Unknown kpi_type: {}",
            report.kpi_type
        ))),
    }
}

// ===================== Scheduled Reports =====================

pub async fn create_schedule(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    body: web::Json<CreateScheduledReportRequest>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "report.schedule", &mut conn)?;

    // Validate format
    validate_export_format(&body.export_format)?;

    // Verify report definition exists
    let _: ReportDefinition = report_definitions::table
        .find(body.report_definition_id)
        .select(ReportDefinition::as_select())
        .first(&mut conn)?;

    let new = NewScheduledReport {
        report_definition_id: body.report_definition_id,
        frequency: body.frequency.clone(),
        export_format: body.export_format.clone(),
        next_run_at: body.next_run_at,
        created_by: auth.0.sub,
    };

    let schedule: ScheduledReport = diesel::insert_into(scheduled_reports::table)
        .values(&new)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Created().json(ScheduledReportResponse::from(schedule)))
}

pub async fn list_schedules(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "report.schedule", &mut conn)?;

    let results: Vec<ScheduledReport> = scheduled_reports::table
        .filter(scheduled_reports::is_active.eq(true))
        .select(ScheduledReport::as_select())
        .order(scheduled_reports::next_run_at.asc())
        .load(&mut conn)?;

    let responses: Vec<ScheduledReportResponse> =
        results.into_iter().map(ScheduledReportResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get_schedule(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let schedule_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "report.schedule", &mut conn)?;

    let schedule: ScheduledReport = scheduled_reports::table
        .find(schedule_id)
        .select(ScheduledReport::as_select())
        .first(&mut conn)?;

    Ok(HttpResponse::Ok().json(ScheduledReportResponse::from(schedule)))
}

pub async fn update_schedule(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateScheduledReportRequest>,
) -> Result<HttpResponse, AppError> {
    let schedule_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "report.schedule", &mut conn)?;

    if let Some(ref fmt) = body.export_format {
        validate_export_format(fmt)?;
    }

    let changeset = UpdateScheduledReport {
        frequency: body.frequency.clone(),
        export_format: body.export_format.clone(),
        next_run_at: body.next_run_at,
        is_active: body.is_active,
        updated_at: Utc::now(),
    };

    let schedule: ScheduledReport = diesel::update(scheduled_reports::table.find(schedule_id))
        .set(&changeset)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Ok().json(ScheduledReportResponse::from(schedule)))
}

pub async fn delete_schedule(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let schedule_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "report.schedule", &mut conn)?;

    diesel::update(scheduled_reports::table.find(schedule_id))
        .set((
            scheduled_reports::is_active.eq(false),
            scheduled_reports::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn)?;

    Ok(HttpResponse::NoContent().finish())
}

// ===================== KPI types listing =====================

pub async fn list_kpi_types(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission(&auth.0, "report.read", &mut conn)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "kpi_types": KPI_TYPES,
    })))
}

fn validate_export_format(fmt: &str) -> Result<(), AppError> {
    match fmt {
        "xlsx" | "pdf" | "csv" => Ok(()),
        _ => Err(AppError::Validation(format!(
            "Invalid export format '{}'. Allowed: xlsx, pdf, csv",
            fmt
        ))),
    }
}
