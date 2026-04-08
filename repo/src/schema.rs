pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "data_scope_enum"))]
    pub struct DataScopeEnum;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "approval_status"))]
    pub struct ApprovalStatusType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "order_status"))]
    pub struct OrderStatusType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "tender_type"))]
    pub struct TenderTypeType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "ledger_entry_kind"))]
    pub struct LedgerEntryKindType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "closing_status"))]
    pub struct ClosingStatusType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "dataset_type"))]
    pub struct DatasetTypeType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "notification_status"))]
    pub struct NotificationStatusType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "notification_category"))]
    pub struct NotificationCategoryType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "delivery_result"))]
    pub struct DeliveryResultType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "schedule_frequency"))]
    pub struct ScheduleFrequencyType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "export_status"))]
    pub struct ExportStatusType;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::DataScopeEnum;

    roles (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        data_scope -> DataScopeEnum,
        scope_value -> Nullable<Varchar>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        password_hash_enc -> Bytea,
        gov_id_enc -> Nullable<Bytea>,
        gov_id_last4 -> Nullable<Varchar>,
        role_id -> Uuid,
        department -> Nullable<Varchar>,
        location -> Nullable<Varchar>,
        is_active -> Bool,
        failed_attempts -> Int4,
        locked_until -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    permission_points (id) {
        id -> Uuid,
        code -> Varchar,
        description -> Nullable<Text>,
        requires_approval -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    role_permissions (id) {
        id -> Uuid,
        role_id -> Uuid,
        permission_point_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    api_capabilities (id) {
        id -> Uuid,
        permission_point_id -> Uuid,
        http_method -> Varchar,
        path_pattern -> Varchar,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    menu_scopes (id) {
        id -> Uuid,
        permission_point_id -> Uuid,
        menu_key -> Varchar,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    delegations (id) {
        id -> Uuid,
        delegator_user_id -> Uuid,
        delegate_user_id -> Uuid,
        permission_point_id -> Uuid,
        source_department -> Nullable<Varchar>,
        target_department -> Nullable<Varchar>,
        starts_at -> Timestamptz,
        ends_at -> Timestamptz,
        is_active -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    approval_policies (id) {
        id -> Uuid,
        permission_point_id -> Uuid,
        min_approvers -> Int4,
        approver_role_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ApprovalStatusType;

    approval_requests (id) {
        id -> Uuid,
        permission_point_id -> Uuid,
        requester_user_id -> Uuid,
        payload -> Jsonb,
        status -> ApprovalStatusType,
        approved_by -> Array<Uuid>,
        rejected_by -> Nullable<Uuid>,
        resolved_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    login_attempts (id) {
        id -> Uuid,
        username -> Varchar,
        success -> Bool,
        ip_address -> Nullable<Varchar>,
        attempted_at -> Timestamptz,
    }
}

// --- POS tables ---

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::OrderStatusType;

    orders (id) {
        id -> Uuid,
        order_number -> Varchar,
        status -> OrderStatusType,
        cashier_user_id -> Uuid,
        location -> Varchar,
        department -> Nullable<Varchar>,
        customer_reference -> Nullable<Varchar>,
        original_order_id -> Nullable<Uuid>,
        subtotal_cents -> Int8,
        tax_cents -> Int8,
        total_cents -> Int8,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    order_line_items (id) {
        id -> Uuid,
        order_id -> Uuid,
        sku -> Varchar,
        description -> Varchar,
        quantity -> Int4,
        unit_price_cents -> Int8,
        tax_cents -> Int8,
        line_total_cents -> Int8,
        original_line_item_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TenderTypeType;
    use super::sql_types::LedgerEntryKindType;

    ledger_entries (id) {
        id -> Uuid,
        order_id -> Uuid,
        tender_type -> TenderTypeType,
        entry_kind -> LedgerEntryKindType,
        amount_cents -> Int8,
        reference_code -> Nullable<Varchar>,
        idempotency_key -> Uuid,
        created_by -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    receipts (id) {
        id -> Uuid,
        order_id -> Uuid,
        receipt_number -> Varchar,
        receipt_data -> Jsonb,
        printed_at -> Timestamptz,
        created_by -> Uuid,
        file_path -> Nullable<Varchar>,
        content_type -> Nullable<Varchar>,
        file_size_bytes -> Nullable<Int8>,
        sha256_hash -> Nullable<Varchar>,
    }
}

diesel::table! {
    idempotency_keys (key) {
        key -> Uuid,
        resource_type -> Varchar,
        resource_id -> Uuid,
        response_status -> Int2,
        response_body -> Jsonb,
        created_at -> Timestamptz,
        expires_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ClosingStatusType;

    register_closings (id) {
        id -> Uuid,
        location -> Varchar,
        cashier_user_id -> Uuid,
        closing_date -> Date,
        expected_cash_cents -> Int8,
        actual_cash_cents -> Int8,
        expected_card_cents -> Int8,
        actual_card_cents -> Int8,
        expected_gift_card_cents -> Int8,
        actual_gift_card_cents -> Int8,
        variance_cents -> Int8,
        status -> ClosingStatusType,
        approval_request_id -> Nullable<Uuid>,
        notes -> Nullable<Text>,
        closed_at -> Timestamptz,
        confirmed_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

// --- Participant tables ---

diesel::table! {
    participants (id) {
        id -> Uuid,
        first_name -> Varchar,
        last_name -> Varchar,
        email -> Nullable<Varchar>,
        phone -> Nullable<Varchar>,
        department -> Nullable<Varchar>,
        location -> Nullable<Varchar>,
        employee_id -> Nullable<Varchar>,
        notes -> Nullable<Text>,
        is_active -> Bool,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    teams (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        department -> Nullable<Varchar>,
        location -> Nullable<Varchar>,
        is_active -> Bool,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    team_members (id) {
        id -> Uuid,
        team_id -> Uuid,
        participant_id -> Uuid,
        role_label -> Nullable<Varchar>,
        joined_at -> Timestamptz,
        left_at -> Nullable<Timestamptz>,
        is_active -> Bool,
    }
}

diesel::table! {
    tags (id) {
        id -> Uuid,
        name -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    participant_tags (id) {
        id -> Uuid,
        participant_id -> Uuid,
        tag_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    file_attachments (id) {
        id -> Uuid,
        participant_id -> Uuid,
        file_name -> Varchar,
        file_path -> Varchar,
        content_type -> Varchar,
        file_size_bytes -> Int8,
        sha256_hash -> Varchar,
        uploaded_by -> Uuid,
        created_at -> Timestamptz,
    }
}

// --- Dataset tables ---

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::DatasetTypeType;

    datasets (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        dataset_type -> DatasetTypeType,
        is_active -> Bool,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    dataset_versions (id) {
        id -> Uuid,
        dataset_id -> Uuid,
        version_number -> Int4,
        storage_path -> Varchar,
        file_size_bytes -> Nullable<Int8>,
        sha256_hash -> Nullable<Varchar>,
        row_count -> Nullable<Int8>,
        transformation_note -> Nullable<Text>,
        is_current -> Bool,
        created_by -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    version_lineage (id) {
        id -> Uuid,
        child_version_id -> Uuid,
        parent_version_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    field_dictionaries (id) {
        id -> Uuid,
        version_id -> Uuid,
        field_name -> Varchar,
        field_type -> Varchar,
        meaning -> Nullable<Text>,
        source_system -> Nullable<Varchar>,
        last_updated_at -> Timestamptz,
    }
}

// --- Notification tables ---

diesel::table! {
    notification_templates (id) {
        id -> Uuid,
        code -> Varchar,
        name -> Varchar,
        subject_template -> Varchar,
        body_template -> Text,
        category -> Varchar,
        is_active -> Bool,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::NotificationStatusType;
    use super::sql_types::NotificationCategoryType;

    notifications (id) {
        id -> Uuid,
        recipient_user_id -> Uuid,
        template_id -> Nullable<Uuid>,
        category -> NotificationCategoryType,
        subject -> Varchar,
        body -> Text,
        status -> NotificationStatusType,
        reference_type -> Nullable<Varchar>,
        reference_id -> Nullable<Uuid>,
        read_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::DeliveryResultType;

    delivery_logs (id) {
        id -> Uuid,
        notification_id -> Uuid,
        attempt_number -> Int4,
        result -> DeliveryResultType,
        error_message -> Nullable<Text>,
        attempted_at -> Timestamptz,
    }
}

// --- Audit table ---

diesel::table! {
    audit_log (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        action -> Varchar,
        resource_type -> Varchar,
        resource_id -> Nullable<Uuid>,
        http_method -> Varchar,
        http_path -> Varchar,
        before_hash -> Nullable<Varchar>,
        after_hash -> Nullable<Varchar>,
        metadata -> Nullable<Jsonb>,
        ip_address -> Nullable<Varchar>,
        created_at -> Timestamptz,
    }
}

// --- Reporting tables ---

diesel::table! {
    report_definitions (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        kpi_type -> Varchar,
        dimensions -> Jsonb,
        filters -> Jsonb,
        chart_config -> Nullable<Jsonb>,
        is_active -> Bool,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ScheduleFrequencyType;

    scheduled_reports (id) {
        id -> Uuid,
        report_definition_id -> Uuid,
        frequency -> ScheduleFrequencyType,
        export_format -> Varchar,
        next_run_at -> Timestamptz,
        last_run_at -> Nullable<Timestamptz>,
        is_active -> Bool,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ExportStatusType;

    export_jobs (id) {
        id -> Uuid,
        report_definition_id -> Uuid,
        export_format -> Varchar,
        status -> ExportStatusType,
        total_rows -> Nullable<Int8>,
        processed_rows -> Int8,
        progress_pct -> Int2,
        file_path -> Nullable<Varchar>,
        file_size_bytes -> Nullable<Int8>,
        error_message -> Nullable<Text>,
        approval_request_id -> Nullable<Uuid>,
        requested_by -> Uuid,
        started_at -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        sha256_hash -> Nullable<Varchar>,
    }
}

// --- Joinable declarations ---

diesel::joinable!(users -> roles (role_id));
diesel::joinable!(role_permissions -> roles (role_id));
diesel::joinable!(role_permissions -> permission_points (permission_point_id));
diesel::joinable!(api_capabilities -> permission_points (permission_point_id));
diesel::joinable!(menu_scopes -> permission_points (permission_point_id));
diesel::joinable!(delegations -> permission_points (permission_point_id));
diesel::joinable!(approval_policies -> permission_points (permission_point_id));
diesel::joinable!(approval_policies -> roles (approver_role_id));
diesel::joinable!(approval_requests -> permission_points (permission_point_id));
diesel::joinable!(approval_requests -> users (requester_user_id));

diesel::joinable!(order_line_items -> orders (order_id));
diesel::joinable!(ledger_entries -> orders (order_id));
diesel::joinable!(receipts -> orders (order_id));
diesel::joinable!(register_closings -> approval_requests (approval_request_id));

diesel::joinable!(team_members -> teams (team_id));
diesel::joinable!(team_members -> participants (participant_id));
diesel::joinable!(participant_tags -> participants (participant_id));
diesel::joinable!(participant_tags -> tags (tag_id));
diesel::joinable!(file_attachments -> participants (participant_id));

diesel::joinable!(dataset_versions -> datasets (dataset_id));
diesel::joinable!(field_dictionaries -> dataset_versions (version_id));

diesel::joinable!(notifications -> notification_templates (template_id));
diesel::joinable!(delivery_logs -> notifications (notification_id));

diesel::joinable!(scheduled_reports -> report_definitions (report_definition_id));
diesel::joinable!(export_jobs -> report_definitions (report_definition_id));
diesel::joinable!(export_jobs -> approval_requests (approval_request_id));

diesel::allow_tables_to_appear_in_same_query!(
    roles,
    users,
    permission_points,
    role_permissions,
    api_capabilities,
    menu_scopes,
    delegations,
    approval_policies,
    approval_requests,
    login_attempts,
    orders,
    order_line_items,
    ledger_entries,
    receipts,
    idempotency_keys,
    register_closings,
    participants,
    teams,
    team_members,
    tags,
    participant_tags,
    file_attachments,
    datasets,
    dataset_versions,
    version_lineage,
    field_dictionaries,
    notification_templates,
    notifications,
    delivery_logs,
    report_definitions,
    scheduled_reports,
    export_jobs,
    audit_log,
);
