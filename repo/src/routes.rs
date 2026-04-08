use actix_web::web;

use crate::handlers::{
    api_scope_handler, approval_handler, attachment_handler, audit_handler, auth_handler,
    dataset_handler, delegation_handler, export_handler, menu_scope_handler,
    notification_handler, order_handler, participant_handler, permission_handler,
    register_handler, report_handler, return_handler, role_handler, team_handler, user_handler,
};
use crate::observability::health;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            // --- Auth (no auth required) ---
            .service(
                web::scope("/auth")
                    .route("/login", web::post().to(auth_handler::login))
                    .route("/refresh", web::post().to(auth_handler::refresh))
                    .route("/bootstrap", web::post().to(auth_handler::bootstrap)),
            )
            // --- Users ---
            .service(
                web::scope("/users")
                    .route("", web::get().to(user_handler::list_users))
                    .route("", web::post().to(user_handler::create_user))
                    .route("/{id}", web::get().to(user_handler::get_user)),
            )
            // --- Roles ---
            .service(
                web::scope("/roles")
                    .route("", web::get().to(role_handler::list))
                    .route("", web::post().to(role_handler::create))
                    .route("/{id}", web::get().to(role_handler::get))
                    .route("/{id}", web::put().to(role_handler::update))
                    .route("/{id}", web::delete().to(role_handler::delete))
                    .route(
                        "/{role_id}/permissions",
                        web::post().to(permission_handler::bind_to_role),
                    )
                    .route(
                        "/{role_id}/permissions/{perm_id}",
                        web::delete().to(permission_handler::unbind_from_role),
                    ),
            )
            // --- Permission Points ---
            .service(
                web::scope("/permissions")
                    .route("", web::get().to(permission_handler::list))
                    .route("", web::post().to(permission_handler::create))
                    .route("/{id}", web::get().to(permission_handler::get))
                    .route("/{id}", web::put().to(permission_handler::update))
                    .route("/{id}", web::delete().to(permission_handler::delete)),
            )
            // --- API Capabilities ---
            .service(
                web::scope("/api-capabilities")
                    .route("", web::get().to(api_scope_handler::list))
                    .route("", web::post().to(api_scope_handler::create))
                    .route("/{id}", web::get().to(api_scope_handler::get))
                    .route("/{id}", web::put().to(api_scope_handler::update))
                    .route("/{id}", web::delete().to(api_scope_handler::delete)),
            )
            // --- Menu Scopes ---
            .service(
                web::scope("/menu-scopes")
                    .route("", web::get().to(menu_scope_handler::list))
                    .route("", web::post().to(menu_scope_handler::create))
                    .route("/{id}", web::get().to(menu_scope_handler::get))
                    .route("/{id}", web::put().to(menu_scope_handler::update))
                    .route("/{id}", web::delete().to(menu_scope_handler::delete)),
            )
            // --- Delegations ---
            .service(
                web::scope("/delegations")
                    .route("", web::get().to(delegation_handler::list))
                    .route("", web::post().to(delegation_handler::create))
                    .route("/{id}/revoke", web::post().to(delegation_handler::revoke)),
            )
            // --- Approvals ---
            .service(
                web::scope("/approvals")
                    .route("", web::get().to(approval_handler::list))
                    .route("", web::post().to(approval_handler::create_approval_request))
                    .route("/{id}", web::get().to(approval_handler::get))
                    .route("/{id}/approve", web::post().to(approval_handler::approve))
                    .route("/{id}/reject", web::post().to(approval_handler::reject)),
            )
            // --- POS Orders ---
            .service(
                web::scope("/orders")
                    .route("", web::post().to(order_handler::create_order))
                    .route("", web::get().to(order_handler::list_orders))
                    .route("/{id}", web::get().to(order_handler::get_order))
                    .route("/{id}", web::put().to(order_handler::update_order))
                    .route(
                        "/{id}/transition",
                        web::post().to(order_handler::transition_order),
                    )
                    .route(
                        "/{id}/payments",
                        web::post().to(order_handler::add_payment),
                    )
                    .route(
                        "/{id}/payments",
                        web::get().to(order_handler::list_payments),
                    )
                    .route(
                        "/{id}/receipts",
                        web::post().to(order_handler::attach_receipt),
                    )
                    .route(
                        "/{id}/returns",
                        web::post().to(return_handler::initiate_return),
                    )
                    .route(
                        "/{id}/exchanges",
                        web::post().to(return_handler::initiate_exchange),
                    )
                    .route(
                        "/{id}/reversals",
                        web::post().to(return_handler::initiate_reversal),
                    )
                    .route(
                        "/{id}/reversals/execute",
                        web::post().to(return_handler::execute_reversal),
                    ),
            )
            // --- Register / End-of-Day ---
            .service(
                web::scope("/registers")
                    .route("/close", web::post().to(register_handler::close_register))
                    .route(
                        "/closings",
                        web::get().to(register_handler::list_closings),
                    )
                    .route(
                        "/closings/{id}",
                        web::get().to(register_handler::get_closing),
                    )
                    .route(
                        "/closings/{id}/confirm",
                        web::post().to(register_handler::confirm_closing),
                    ),
            )
            // --- Participants ---
            .service(
                web::scope("/participants")
                    .route("", web::post().to(participant_handler::create))
                    .route("", web::get().to(participant_handler::list))
                    .route("/bulk/tag", web::post().to(participant_handler::bulk_tag))
                    .route(
                        "/bulk/deactivate",
                        web::post().to(participant_handler::bulk_deactivate),
                    )
                    .route("/{id}", web::get().to(participant_handler::get))
                    .route("/{id}", web::put().to(participant_handler::update))
                    .route("/{id}", web::delete().to(participant_handler::deactivate))
                    .route("/{id}/tags", web::get().to(participant_handler::get_tags))
                    .route("/{id}/tags", web::put().to(participant_handler::set_tags))
                    .route(
                        "/{id}/attachments",
                        web::post().to(attachment_handler::upload),
                    )
                    .route(
                        "/{id}/attachments",
                        web::get().to(attachment_handler::list),
                    )
                    .route(
                        "/{id}/attachments/{attachment_id}",
                        web::get().to(attachment_handler::download),
                    )
                    .route(
                        "/{id}/attachments/{attachment_id}",
                        web::delete().to(attachment_handler::delete),
                    ),
            )
            // --- Teams ---
            .service(
                web::scope("/teams")
                    .route("", web::post().to(team_handler::create))
                    .route("", web::get().to(team_handler::list))
                    .route("/{id}", web::get().to(team_handler::get))
                    .route("/{id}", web::put().to(team_handler::update))
                    .route("/{id}", web::delete().to(team_handler::deactivate))
                    .route("/{id}/members", web::get().to(team_handler::list_members))
                    .route("/{id}/members", web::post().to(team_handler::add_member))
                    .route(
                        "/{id}/members/{participant_id}",
                        web::delete().to(team_handler::remove_member),
                    ),
            )
            // --- Datasets ---
            .service(
                web::scope("/datasets")
                    .route("", web::post().to(dataset_handler::create_dataset))
                    .route("", web::get().to(dataset_handler::list_datasets))
                    .route("/{id}", web::get().to(dataset_handler::get_dataset))
                    .route("/{id}", web::put().to(dataset_handler::update_dataset))
                    .route("/{id}", web::delete().to(dataset_handler::deactivate_dataset))
                    // Versions
                    .route(
                        "/{id}/versions",
                        web::post().to(dataset_handler::create_version),
                    )
                    .route(
                        "/{id}/versions",
                        web::get().to(dataset_handler::list_versions),
                    )
                    .route(
                        "/{id}/versions/{version_id}",
                        web::get().to(dataset_handler::get_version),
                    )
                    // Lineage
                    .route(
                        "/{id}/versions/{version_id}/lineage",
                        web::get().to(dataset_handler::get_lineage),
                    )
                    // Field dictionary
                    .route(
                        "/{id}/versions/{version_id}/fields",
                        web::get().to(dataset_handler::list_field_dictionary),
                    )
                    .route(
                        "/{id}/versions/{version_id}/fields",
                        web::post().to(dataset_handler::add_field_entry),
                    )
                    .route(
                        "/{id}/versions/{version_id}/fields/{field_id}",
                        web::put().to(dataset_handler::update_field_entry),
                    )
                    .route(
                        "/{id}/versions/{version_id}/fields/{field_id}",
                        web::delete().to(dataset_handler::delete_field_entry),
                    )
                    // Rollback
                    .route(
                        "/{id}/rollback",
                        web::post().to(dataset_handler::rollback),
                    )
                    .route(
                        "/{id}/rollback/execute",
                        web::post().to(dataset_handler::execute_rollback),
                    ),
            )
            // --- Notification Templates ---
            .service(
                web::scope("/notification-templates")
                    .route("", web::post().to(notification_handler::create_template))
                    .route("", web::get().to(notification_handler::list_templates))
                    .route("/{id}", web::get().to(notification_handler::get_template))
                    .route("/{id}", web::put().to(notification_handler::update_template))
                    .route(
                        "/{id}",
                        web::delete().to(notification_handler::delete_template),
                    ),
            )
            // --- Notifications ---
            .service(
                web::scope("/notifications")
                    // Send
                    .route("/send", web::post().to(notification_handler::send_templated))
                    .route(
                        "/send-direct",
                        web::post().to(notification_handler::send_direct),
                    )
                    .route(
                        "/broadcast",
                        web::post().to(notification_handler::broadcast),
                    )
                    // Inbox (own)
                    .route("/inbox", web::get().to(notification_handler::inbox))
                    .route(
                        "/inbox/unread-count",
                        web::get().to(notification_handler::unread_count),
                    )
                    .route(
                        "/inbox/read-all",
                        web::post().to(notification_handler::mark_all_read),
                    )
                    .route(
                        "/inbox/{id}",
                        web::get().to(notification_handler::get_notification),
                    )
                    .route(
                        "/inbox/{id}/read",
                        web::post().to(notification_handler::mark_read),
                    )
                    // Admin
                    .route("/admin", web::get().to(notification_handler::admin_list))
                    .route(
                        "/admin/{id}/delivery-logs",
                        web::get().to(notification_handler::delivery_logs),
                    )
                    .route(
                        "/admin/{id}/retry",
                        web::post().to(notification_handler::retry),
                    ),
            )
            // --- Reports & Analytics ---
            .service(
                web::scope("/reports")
                    .route("/kpi-types", web::get().to(report_handler::list_kpi_types))
                    .route("", web::post().to(report_handler::create_definition))
                    .route("", web::get().to(report_handler::list_definitions))
                    .route("/{id}", web::get().to(report_handler::get_definition))
                    .route("/{id}", web::put().to(report_handler::update_definition))
                    .route("/{id}", web::delete().to(report_handler::delete_definition))
                    .route("/{id}/run", web::post().to(report_handler::run_report)),
            )
            // --- Scheduled Reports ---
            .service(
                web::scope("/scheduled-reports")
                    .route("", web::post().to(report_handler::create_schedule))
                    .route("", web::get().to(report_handler::list_schedules))
                    .route("/{id}", web::get().to(report_handler::get_schedule))
                    .route("/{id}", web::put().to(report_handler::update_schedule))
                    .route("/{id}", web::delete().to(report_handler::delete_schedule)),
            )
            // --- Export Jobs ---
            .service(
                web::scope("/exports")
                    .route("", web::post().to(export_handler::request_export))
                    .route("", web::get().to(export_handler::list_jobs))
                    .route("/admin", web::get().to(export_handler::admin_list_jobs))
                    .route("/{id}", web::get().to(export_handler::get_job))
                    .route(
                        "/{id}/progress",
                        web::put().to(export_handler::update_progress),
                    )
                    .route(
                        "/{id}/complete",
                        web::post().to(export_handler::complete_job),
                    )
                    .route("/{id}/fail", web::post().to(export_handler::fail_job))
                    .route(
                        "/{id}/cancel",
                        web::post().to(export_handler::cancel_job),
                    )
                    .route(
                        "/{id}/download",
                        web::get().to(export_handler::download_export),
                    ),
            )
            // --- Audit Trail ---
            .service(
                web::scope("/audit")
                    .route("", web::get().to(audit_handler::list))
                    .route("/{id}", web::get().to(audit_handler::get)),
            )
            // --- Health & Metrics (health is public, metrics requires auth) ---
            .route("/health", web::get().to(health::health_check))
            .route("/metrics", web::get().to(health::metrics_endpoint)),
    );
}
