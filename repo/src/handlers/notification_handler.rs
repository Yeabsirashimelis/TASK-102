use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::delivery_log::*;
use crate::models::notification::*;
use crate::models::notification_template::*;
use crate::rbac::guard::check_permission_for_request;

fn check_perm(auth: &crate::auth::jwt::Claims, code: &str, req: &HttpRequest, conn: &mut diesel::PgConnection)
    -> Result<crate::rbac::data_scope::PermissionContext, AppError> {
    check_permission_for_request(auth, code, req.method().as_str(), req.path(), conn)
}
use crate::schema::{delivery_logs, notification_templates, notifications, users};

// ===================== Template CRUD =====================

pub async fn create_template(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    body: web::Json<CreateTemplateRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "notification.template.create", &req, &mut conn)?;

    let new = NewNotificationTemplate {
        code: body.code.clone(),
        name: body.name.clone(),
        subject_template: body.subject_template.clone(),
        body_template: body.body_template.clone(),
        category: body.category.clone(),
        created_by: auth.0.sub,
    };

    let template: NotificationTemplate = diesel::insert_into(notification_templates::table)
        .values(&new)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Created().json(NotificationTemplateResponse::from(template)))
}

pub async fn list_templates(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "notification.template.read", &req, &mut conn)?;

    let results: Vec<NotificationTemplate> = notification_templates::table
        .filter(notification_templates::is_active.eq(true))
        .select(NotificationTemplate::as_select())
        .order(notification_templates::code.asc())
        .load(&mut conn)?;

    let responses: Vec<NotificationTemplateResponse> =
        results.into_iter().map(NotificationTemplateResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get_template(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let template_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "notification.template.read", &req, &mut conn)?;

    let template: NotificationTemplate = notification_templates::table
        .find(template_id)
        .select(NotificationTemplate::as_select())
        .first(&mut conn)?;

    Ok(HttpResponse::Ok().json(NotificationTemplateResponse::from(template)))
}

pub async fn update_template(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateTemplateRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let template_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "notification.template.update", &req, &mut conn)?;

    let changeset = UpdateNotificationTemplate {
        name: body.name.clone(),
        subject_template: body.subject_template.clone(),
        body_template: body.body_template.clone(),
        category: body.category.clone(),
        is_active: body.is_active,
        updated_at: Utc::now(),
    };

    let template: NotificationTemplate =
        diesel::update(notification_templates::table.find(template_id))
            .set(&changeset)
            .get_result(&mut conn)?;

    Ok(HttpResponse::Ok().json(NotificationTemplateResponse::from(template)))
}

pub async fn delete_template(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let template_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "notification.template.delete", &req, &mut conn)?;

    diesel::update(notification_templates::table.find(template_id))
        .set((
            notification_templates::is_active.eq(false),
            notification_templates::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn)?;

    Ok(HttpResponse::NoContent().finish())
}

// ===================== Send Notifications =====================

pub async fn send_templated(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    body: web::Json<SendNotificationRequest>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "notification.send", &req, &mut conn)?;

    // Look up template
    let template: NotificationTemplate = notification_templates::table
        .filter(notification_templates::code.eq(&body.template_code))
        .filter(notification_templates::is_active.eq(true))
        .select(NotificationTemplate::as_select())
        .first(&mut conn)
        .map_err(|_| {
            AppError::NotFound(format!("Template '{}' not found", body.template_code))
        })?;

    // Variable substitution
    let subject = substitute_variables(&template.subject_template, &body.variables);
    let rendered_body = substitute_variables(&template.body_template, &body.variables);

    let category = parse_category(&template.category)?;

    let new = NewNotification {
        recipient_user_id: body.recipient_user_id,
        template_id: Some(template.id),
        category,
        subject,
        body: rendered_body,
        status: NotificationStatus::Pending,
        reference_type: body.reference_type.clone(),
        reference_id: body.reference_id,
    };

    let notification = create_and_deliver(&mut conn, new)?;
    let after = serde_json::json!({"id": notification.id, "recipient": notification.recipient_user_id, "category": format!("{:?}", notification.category)});
    let _ = crate::audit::service::audit_write(&mut conn, auth.0.sub, "create", "notifications", Some(notification.id), None, Some(&after));
    Ok(HttpResponse::Created().json(NotificationResponse::from(notification)))
}

pub async fn send_direct(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    body: web::Json<SendDirectNotificationRequest>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "notification.send", &req, &mut conn)?;

    let new = NewNotification {
        recipient_user_id: body.recipient_user_id,
        template_id: None,
        category: body.category.clone(),
        subject: body.subject.clone(),
        body: body.body.clone(),
        status: NotificationStatus::Pending,
        reference_type: body.reference_type.clone(),
        reference_id: body.reference_id,
    };

    let notification = create_and_deliver(&mut conn, new)?;
    let after = serde_json::json!({"id": notification.id, "recipient": notification.recipient_user_id});
    let _ = crate::audit::service::audit_write(&mut conn, auth.0.sub, "create", "notifications", Some(notification.id), None, Some(&after));
    Ok(HttpResponse::Created().json(NotificationResponse::from(notification)))
}

pub async fn broadcast(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    body: web::Json<BroadcastRequest>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "notification.broadcast", &req, &mut conn)?;

    // Get all active user IDs
    let user_ids: Vec<Uuid> = users::table
        .filter(users::is_active.eq(true))
        .select(users::id)
        .load(&mut conn)?;

    let mut count = 0usize;
    for uid in &user_ids {
        let new = NewNotification {
            recipient_user_id: *uid,
            template_id: None,
            category: NotificationCategory::SystemAnnouncement,
            subject: body.subject.clone(),
            body: body.body.clone(),
            status: NotificationStatus::Pending,
            reference_type: None,
            reference_id: None,
        };
        create_and_deliver(&mut conn, new)?;
        count += 1;
    }

    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Broadcast sent",
        "recipients": count,
    })))
}

// ===================== Inbox (own notifications) =====================

pub async fn inbox(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    query: web::Query<NotificationQueryParams>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "notification.read", &req, &mut conn)?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    let mut q = notifications::table
        .filter(notifications::recipient_user_id.eq(auth.0.sub))
        .into_boxed();

    if let Some(ref status) = query.status {
        q = q.filter(notifications::status.eq(parse_status(status)?));
    }
    if let Some(ref cat) = query.category {
        q = q.filter(notifications::category.eq(parse_category(cat)?));
    }

    let results: Vec<Notification> = q
        .select(Notification::as_select())
        .order(notifications::created_at.desc())
        .offset(offset)
        .limit(per_page)
        .load(&mut conn)?;

    let responses: Vec<NotificationResponse> =
        results.into_iter().map(NotificationResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get_notification(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let notif_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "notification.read", &req, &mut conn)?;

    let notif: Notification = notifications::table
        .find(notif_id)
        .filter(notifications::recipient_user_id.eq(auth.0.sub))
        .select(Notification::as_select())
        .first(&mut conn)?;

    Ok(HttpResponse::Ok().json(NotificationResponse::from(notif)))
}

pub async fn mark_read(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let notif_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "notification.read", &req, &mut conn)?;

    let updated = diesel::update(
        notifications::table
            .find(notif_id)
            .filter(notifications::recipient_user_id.eq(auth.0.sub)),
    )
    .set((
        notifications::status.eq(NotificationStatus::Read),
        notifications::read_at.eq(Some(Utc::now())),
    ))
    .execute(&mut conn)?;

    if updated == 0 {
        return Err(AppError::NotFound("Notification not found".into()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "read" })))
}

pub async fn mark_all_read(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "notification.read", &req, &mut conn)?;

    let updated = diesel::update(
        notifications::table
            .filter(notifications::recipient_user_id.eq(auth.0.sub))
            .filter(notifications::status.ne(NotificationStatus::Read)),
    )
    .set((
        notifications::status.eq(NotificationStatus::Read),
        notifications::read_at.eq(Some(Utc::now())),
    ))
    .execute(&mut conn)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "marked_read": updated })))
}

pub async fn unread_count(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "notification.read", &req, &mut conn)?;

    let count: i64 = notifications::table
        .filter(notifications::recipient_user_id.eq(auth.0.sub))
        .filter(notifications::status.ne(NotificationStatus::Read))
        .count()
        .get_result(&mut conn)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "unread_count": count })))
}

// ===================== Admin: all notifications =====================

pub async fn admin_list(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    query: web::Query<NotificationQueryParams>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "notification.admin", &req, &mut conn)?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    let mut q = notifications::table.into_boxed();

    if let Some(ref status) = query.status {
        q = q.filter(notifications::status.eq(parse_status(status)?));
    }
    if let Some(ref cat) = query.category {
        q = q.filter(notifications::category.eq(parse_category(cat)?));
    }

    let results: Vec<Notification> = q
        .select(Notification::as_select())
        .order(notifications::created_at.desc())
        .offset(offset)
        .limit(per_page)
        .load(&mut conn)?;

    let responses: Vec<NotificationResponse> =
        results.into_iter().map(NotificationResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

// ===================== Delivery Logs & Retry =====================

pub async fn delivery_logs(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let notif_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "notification.admin", &req, &mut conn)?;

    let logs: Vec<DeliveryLog> = delivery_logs::table
        .filter(delivery_logs::notification_id.eq(notif_id))
        .select(DeliveryLog::as_select())
        .order(delivery_logs::attempt_number.asc())
        .load(&mut conn)?;

    let responses: Vec<DeliveryLogResponse> =
        logs.into_iter().map(DeliveryLogResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn retry(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let notif_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "notification.retry", &req, &mut conn)?;

    let notif: Notification = notifications::table
        .find(notif_id)
        .select(Notification::as_select())
        .first(&mut conn)?;

    if notif.status != NotificationStatus::Failed {
        return Err(AppError::Validation(
            "Only failed notifications can be retried".into(),
        ));
    }

    // Get max attempt number
    let max_attempt: Option<i32> = delivery_logs::table
        .filter(delivery_logs::notification_id.eq(notif_id))
        .select(diesel::dsl::max(delivery_logs::attempt_number))
        .first(&mut conn)?;
    let next_attempt = max_attempt.unwrap_or(0) + 1;

    // Attempt delivery (in-app = always succeeds since it's just a DB record)
    let log = NewDeliveryLog {
        notification_id: notif_id,
        attempt_number: next_attempt,
        result: DeliveryResult::Success,
        error_message: None,
    };

    let delivery: DeliveryLog = diesel::insert_into(delivery_logs::table)
        .values(&log)
        .get_result(&mut conn)?;

    // Mark notification as delivered
    diesel::update(notifications::table.find(notif_id))
        .set(notifications::status.eq(NotificationStatus::Delivered))
        .execute(&mut conn)?;

    Ok(HttpResponse::Ok().json(DeliveryLogResponse::from(delivery)))
}

// ===================== Helpers =====================

/// Performs {{variable}} substitution on a template string.
fn substitute_variables(
    template: &str,
    variables: &std::collections::HashMap<String, String>,
) -> String {
    let mut result = template.to_string();
    for (key, value) in variables {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

/// Creates a notification, attempts in-app delivery, and records the delivery log.
fn create_and_deliver(
    conn: &mut PgConnection,
    new: NewNotification,
) -> Result<Notification, AppError> {
    let notification: Notification = diesel::insert_into(notifications::table)
        .values(&new)
        .get_result(conn)?;

    // In-app delivery: since all notifications are internal DB records,
    // delivery always succeeds. The "delivery" is the act of writing
    // the record — the user sees it when they query their inbox.
    let log = NewDeliveryLog {
        notification_id: notification.id,
        attempt_number: 1,
        result: DeliveryResult::Success,
        error_message: None,
    };
    diesel::insert_into(delivery_logs::table)
        .values(&log)
        .execute(conn)?;

    // Update status to delivered
    diesel::update(notifications::table.find(notification.id))
        .set(notifications::status.eq(NotificationStatus::Delivered))
        .execute(conn)?;

    // Re-fetch with updated status
    let updated: Notification = notifications::table
        .find(notification.id)
        .select(Notification::as_select())
        .first(conn)?;

    Ok(updated)
}

fn parse_status(s: &str) -> Result<NotificationStatus, AppError> {
    match s {
        "pending" => Ok(NotificationStatus::Pending),
        "delivered" => Ok(NotificationStatus::Delivered),
        "read" => Ok(NotificationStatus::Read),
        "failed" => Ok(NotificationStatus::Failed),
        _ => Err(AppError::Validation(format!("Invalid notification status: {}", s))),
    }
}

fn parse_category(s: &str) -> Result<NotificationCategory, AppError> {
    match s {
        "moderation_outcome" => Ok(NotificationCategory::ModerationOutcome),
        "comment_reply" => Ok(NotificationCategory::CommentReply),
        "system_announcement" => Ok(NotificationCategory::SystemAnnouncement),
        "general" => Ok(NotificationCategory::General),
        _ => Err(AppError::Validation(format!("Invalid notification category: {}", s))),
    }
}
