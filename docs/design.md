# RetailOps Data & Transaction Governance API -- Design Document

Version: 1.0
Date: 2026-04-07
Status: Draft

---

## Table of Contents

1. [Introduction & Scope](#1-introduction--scope)
2. [System Architecture](#2-system-architecture)
3. [Technology Stack](#3-technology-stack)
4. [Database Design](#4-database-design)
5. [Authentication & Session Management](#5-authentication--session-management)
6. [Authorization & RBAC](#6-authorization--rbac)
7. [POS Transaction Engine](#7-pos-transaction-engine)
8. [End-of-Day Reconciliation](#8-end-of-day-reconciliation)
9. [Participant Data Management](#9-participant-data-management)
10. [Data Storage & Version Management](#10-data-storage--version-management)
11. [Notification System](#11-notification-system)
12. [Reporting & Analytics](#12-reporting--analytics)
13. [File Storage](#13-file-storage)
14. [Audit Trail](#14-audit-trail)
15. [Security](#15-security)
16. [Observability](#16-observability)
17. [Deployment & Performance](#17-deployment--performance)

---

## 1. Introduction & Scope

### 1.1 Purpose

RetailOps is a backend API that unifies offline point-of-sale operations with governed, analytics-ready data management for a US-based multi-location retailer. The system provides a single authoritative service for transaction processing, participant management, dataset governance, reporting, and audit compliance -- all operating entirely offline on a single node with no dependency on external networks or cloud services.

### 1.2 Boundaries

- **Backend API only.** There is no frontend, no server-rendered HTML, no browser-facing assets. All interaction occurs through JSON HTTP endpoints consumed by downstream clients (POS terminals, admin tools, reporting dashboards) that are outside the scope of this document.
- **Fully offline.** The system runs on a single Docker node within the retailer's local network. No outbound internet calls, no SaaS integrations, no external authentication providers.
- **Single-node deployment.** No clustering, no distributed consensus, no message queues. PostgreSQL and the Actix-web application server run on the same host.

### 1.3 Target Users

| Role | Description |
|---|---|
| System Admin | Full system configuration, user provisioning, security policy management, audit review |
| Store Manager | Location-level oversight, register reconciliation approval, refund/reversal authorization, local reporting |
| Cashier | POS order entry, tender processing, register close initiation |
| Analyst / Report Viewer | Dataset exploration, report generation, KPI dashboards, data export |
| Program Coordinator | Participant and team management, roster operations, credential tracking |

---

## 2. System Architecture

### 2.1 Component Overview

```
+-------------------------------------------------------------+
|  Docker Container (single node)                             |
|                                                             |
|  +------------------+      +-----------------------------+  |
|  | Actix-web        |      | PostgreSQL 16               |  |
|  | HTTP Server      |<---->| (Diesel ORM, r2d2 pool)     |  |
|  | (port 8080)      |      |                             |  |
|  +------------------+      +-----------------------------+  |
|         |                                                   |
|         v                                                   |
|  +------------------+                                       |
|  | Local File       |                                       |
|  | System           |                                       |
|  | /data/files/     |                                       |
|  | /data/exports/   |                                       |
|  | /data/logs/      |                                       |
|  +------------------+                                       |
+-------------------------------------------------------------+
```

### 2.2 Component Interactions

1. **HTTP Layer (Actix-web):** Receives all inbound requests, performs authentication and authorization middleware checks, deserializes JSON payloads via serde, and routes to handler functions.
2. **ORM Layer (Diesel):** Handlers invoke Diesel query builders that compile to type-checked SQL at build time. All database access flows through a connection pool managed by r2d2.
3. **PostgreSQL:** Single database instance storing all relational data. Provides ACID transactions, row-level locking, and index-backed queries.
4. **Local File System:** Stores uploaded files (receipts, credentials, exports) at rest. The database holds pointer records (path, size, SHA-256) while binary content lives on disk.

### 2.3 Request Lifecycle

1. Actix-web accepts the TCP connection and parses the HTTP request.
2. Rate-limiting middleware checks per-IP and per-session counters.
3. Authentication middleware extracts and validates the session token from the `Authorization` header.
4. Authorization middleware resolves the user's roles, permission points, and data-scope rules against the target endpoint and resource.
5. The handler function executes business logic, issues Diesel queries inside a database transaction where needed, writes files to disk if applicable, and appends an audit-log entry.
6. The response is serialized to JSON and returned with appropriate status codes.

---

## 3. Technology Stack

### 3.1 Core

| Component | Technology | Purpose |
|---|---|---|
| Language | Rust (stable) | Memory safety, performance, compile-time correctness |
| HTTP Framework | Actix-web 4 | Async HTTP server with middleware pipeline |
| ORM | Diesel 2 | Compile-time query validation, migration management |
| Database | PostgreSQL 16 | ACID relational storage |
| Runtime | Tokio | Async I/O runtime backing Actix-web |
| Container | Docker | Single-node deployment and isolation |

### 3.2 Key Crates

| Crate | Purpose |
|---|---|
| `serde` / `serde_json` | JSON serialization and deserialization for all request/response types |
| `argon2` | Password hashing (Argon2id variant) |
| `sha2` | SHA-256 hashing for file fingerprints, audit before/after hashes, row integrity |
| `tokio` | Async runtime (used by Actix-web internally and for background task spawning) |
| `uuid` | v4 UUID generation for primary keys, session tokens, idempotency keys, version IDs |
| `chrono` | Timestamp handling, timezone-aware datetime, duration calculations |
| `calamine` | Reading inbound Excel (.xlsx) files during data import |
| `xlsxwriter` | Writing Excel (.xlsx) exports for reports and bulk data |
| `printpdf` | Generating PDF receipts and report outputs |
| `r2d2` | Database connection pooling for Diesel |
| `rand` | Cryptographically secure random number generation for tokens |
| `diesel_migrations` | Embedded migration runner for schema setup at startup |

---

## 4. Database Design

All primary keys are UUID v4 unless noted. All tables include `created_at TIMESTAMPTZ NOT NULL DEFAULT now()` and `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()` unless explicitly listed otherwise.

### 4.1 Users & Authentication

#### `users`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| username | VARCHAR(64) | UNIQUE, NOT NULL |
| email | VARCHAR(255) | UNIQUE, NOT NULL |
| password_hash | TEXT | NOT NULL |
| display_name | VARCHAR(128) | NOT NULL |
| is_active | BOOLEAN | NOT NULL DEFAULT true |
| locked_until | TIMESTAMPTZ | NULLABLE |
| last_login_at | TIMESTAMPTZ | NULLABLE |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

#### `password_history`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| user_id | UUID | FK -> users(id), NOT NULL |
| password_hash | TEXT | NOT NULL |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

Index: `(user_id, created_at DESC)`. Used to enforce password reuse prevention (last 10 hashes).

#### `login_attempts`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| user_id | UUID | FK -> users(id), NOT NULL |
| ip_address | VARCHAR(45) | NOT NULL |
| success | BOOLEAN | NOT NULL |
| attempted_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

Index: `(user_id, attempted_at DESC)` for lockout window queries.

#### `sessions`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| user_id | UUID | FK -> users(id), NOT NULL |
| token_hash | VARCHAR(64) | UNIQUE, NOT NULL |
| ip_address | VARCHAR(45) | NOT NULL |
| expires_at | TIMESTAMPTZ | NOT NULL |
| revoked | BOOLEAN | NOT NULL DEFAULT false |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

Index: `(token_hash)` for fast lookup on every authenticated request.

### 4.2 Roles & Permissions

#### `roles`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| name | VARCHAR(64) | UNIQUE, NOT NULL |
| description | TEXT | NULLABLE |
| is_system | BOOLEAN | NOT NULL DEFAULT false |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

Seed data: `system_admin`, `store_manager`, `cashier`, `analyst`, `program_coordinator`.

#### `permission_points`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| code | VARCHAR(128) | UNIQUE, NOT NULL |
| description | TEXT | NULLABLE |
| category | VARCHAR(64) | NOT NULL |

Examples: `pos.order.create`, `pos.refund.approve`, `report.export.bulk`, `dataset.version.rollback`, `participant.team.manage`.

#### `api_scopes`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| permission_point_id | UUID | FK -> permission_points(id), NOT NULL |
| http_method | VARCHAR(10) | NOT NULL |
| path_pattern | VARCHAR(256) | NOT NULL |

Maps permission points to concrete HTTP method + path combinations for enforcement in the authorization middleware.

#### `menu_scopes`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| permission_point_id | UUID | FK -> permission_points(id), NOT NULL |
| menu_key | VARCHAR(128) | NOT NULL |

Provides a queryable registry of which permission points gate which logical menu items. Downstream clients query this to determine which capabilities to expose. No UI rendering occurs server-side.

#### `role_permission_bindings`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| role_id | UUID | FK -> roles(id), NOT NULL |
| permission_point_id | UUID | FK -> permission_points(id), NOT NULL |
| granted | BOOLEAN | NOT NULL DEFAULT true |

Unique constraint: `(role_id, permission_point_id)`.

### 4.3 Data Scope & Delegation

#### `data_scope_rules`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| role_id | UUID | FK -> roles(id), NOT NULL |
| user_id | UUID | FK -> users(id), NULLABLE |
| scope_type | VARCHAR(32) | NOT NULL, CHECK IN ('department', 'location', 'individual') |
| scope_value | VARCHAR(128) | NOT NULL |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

When `user_id` is NULL the rule applies to all users with the given role. When `user_id` is set, the rule applies to that specific user and overrides role-level defaults.

#### `delegations`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| delegator_id | UUID | FK -> users(id), NOT NULL |
| delegatee_id | UUID | FK -> users(id), NOT NULL |
| role_id | UUID | FK -> roles(id), NOT NULL |
| scope_type | VARCHAR(32) | NOT NULL |
| scope_value | VARCHAR(128) | NOT NULL |
| start_at | TIMESTAMPTZ | NOT NULL |
| end_at | TIMESTAMPTZ | NOT NULL |
| revoked | BOOLEAN | NOT NULL DEFAULT false |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

Check constraint: `end_at > start_at`. The authorization layer evaluates delegations by checking `now() BETWEEN start_at AND end_at AND revoked = false`.

### 4.4 Approval Workflow

#### `approval_configs`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| action_code | VARCHAR(128) | UNIQUE, NOT NULL |
| required_role_id | UUID | FK -> roles(id), NOT NULL |
| enabled | BOOLEAN | NOT NULL DEFAULT true |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

Defines which actions require approval and which role may approve them. Examples of `action_code`: `pos.refund`, `pos.reversal`, `dataset.rollback`, `report.bulk_export`.

#### `approval_requests`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| approval_config_id | UUID | FK -> approval_configs(id), NOT NULL |
| requester_id | UUID | FK -> users(id), NOT NULL |
| approver_id | UUID | FK -> users(id), NULLABLE |
| status | VARCHAR(20) | NOT NULL DEFAULT 'pending', CHECK IN ('pending', 'approved', 'rejected', 'expired') |
| resource_type | VARCHAR(64) | NOT NULL |
| resource_id | UUID | NOT NULL |
| context_json | JSONB | NULLABLE |
| decided_at | TIMESTAMPTZ | NULLABLE |
| expires_at | TIMESTAMPTZ | NOT NULL |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

Check constraint: `approver_id != requester_id` (enforces self-approval prevention at the database level).

### 4.5 POS Transactions

#### `pos_orders`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| order_number | BIGSERIAL | UNIQUE, NOT NULL |
| location_id | UUID | NOT NULL |
| cashier_id | UUID | FK -> users(id), NOT NULL |
| status | VARCHAR(24) | NOT NULL DEFAULT 'draft', CHECK IN ('draft', 'open', 'tendering', 'paid', 'closed', 'return_initiated', 'returned', 'reversal_pending', 'reversed') |
| original_order_id | UUID | FK -> pos_orders(id), NULLABLE |
| subtotal | BIGINT | NOT NULL DEFAULT 0 |
| tax_total | BIGINT | NOT NULL DEFAULT 0 |
| total | BIGINT | NOT NULL DEFAULT 0 |
| currency | VARCHAR(3) | NOT NULL DEFAULT 'USD' |
| notes | TEXT | NULLABLE |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

All monetary values are stored in cents (BIGINT) to avoid floating-point precision issues. `original_order_id` links returns and reversals to the originating order.

#### `order_lines`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| order_id | UUID | FK -> pos_orders(id), NOT NULL |
| line_number | INTEGER | NOT NULL |
| sku | VARCHAR(64) | NOT NULL |
| description | VARCHAR(256) | NOT NULL |
| quantity | INTEGER | NOT NULL |
| unit_price | BIGINT | NOT NULL |
| discount | BIGINT | NOT NULL DEFAULT 0 |
| tax | BIGINT | NOT NULL DEFAULT 0 |
| line_total | BIGINT | NOT NULL |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

Unique constraint: `(order_id, line_number)`.

#### `tender_entries`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| order_id | UUID | FK -> pos_orders(id), NOT NULL |
| tender_type | VARCHAR(16) | NOT NULL, CHECK IN ('cash', 'card', 'gift_card') |
| amount | BIGINT | NOT NULL |
| reference | VARCHAR(128) | NULLABLE |
| change_due | BIGINT | NOT NULL DEFAULT 0 |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

Functions as a ledger. Each tender entry records one payment application. For cash, `change_due` captures overpayment. For gift cards, `reference` holds the card identifier.

#### `receipts`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| order_id | UUID | FK -> pos_orders(id), NOT NULL |
| file_id | UUID | FK -> files(id), NOT NULL |
| receipt_type | VARCHAR(16) | NOT NULL DEFAULT 'sale', CHECK IN ('sale', 'return', 'reversal') |
| generated_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

#### `idempotency_keys`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| key | VARCHAR(128) | NOT NULL |
| endpoint | VARCHAR(256) | NOT NULL |
| response_hash | VARCHAR(64) | NOT NULL |
| response_status | SMALLINT | NOT NULL |
| response_body | JSONB | NOT NULL |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |
| expires_at | TIMESTAMPTZ | NOT NULL |

Unique constraint: `(key, endpoint)`. Enables safe retries for POS operations.

### 4.6 Register Reconciliation

#### `register_closes`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| location_id | UUID | NOT NULL |
| cashier_id | UUID | FK -> users(id), NOT NULL |
| register_id | VARCHAR(32) | NOT NULL |
| expected_amount | BIGINT | NOT NULL |
| actual_amount | BIGINT | NOT NULL |
| variance | BIGINT | NOT NULL |
| status | VARCHAR(20) | NOT NULL DEFAULT 'pending', CHECK IN ('pending', 'confirmed', 'flagged') |
| manager_confirmed_by | UUID | FK -> users(id), NULLABLE |
| confirmed_at | TIMESTAMPTZ | NULLABLE |
| shift_date | DATE | NOT NULL |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

Index: `(location_id, shift_date)`.

### 4.7 Participants & Teams

#### `participants`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| first_name | VARCHAR(64) | NOT NULL |
| last_name | VARCHAR(64) | NOT NULL |
| email | VARCHAR(255) | NULLABLE |
| phone | VARCHAR(20) | NULLABLE |
| external_id | VARCHAR(64) | UNIQUE, NULLABLE |
| metadata | JSONB | NULLABLE |
| is_active | BOOLEAN | NOT NULL DEFAULT true |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

#### `teams`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| name | VARCHAR(128) | UNIQUE, NOT NULL |
| description | TEXT | NULLABLE |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

#### `team_members`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| team_id | UUID | FK -> teams(id), NOT NULL |
| participant_id | UUID | FK -> participants(id), NOT NULL |
| role_in_team | VARCHAR(32) | NULLABLE |
| joined_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

Unique constraint: `(team_id, participant_id)`.

#### `participant_tags`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| participant_id | UUID | FK -> participants(id), NOT NULL |
| tag | VARCHAR(64) | NOT NULL |

Unique constraint: `(participant_id, tag)`. Index on `(tag)` for tag-based filtering.

### 4.8 Datasets & Versioning

#### `datasets`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| name | VARCHAR(128) | UNIQUE, NOT NULL |
| description | TEXT | NULLABLE |
| owner_id | UUID | FK -> users(id), NOT NULL |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

#### `dataset_versions`

| Column | Type | Constraints |
|---|---|---|
| version_id | UUID | PK |
| dataset_id | UUID | FK -> datasets(id), NOT NULL |
| dataset_type | VARCHAR(16) | NOT NULL, CHECK IN ('raw', 'cleaned', 'feature', 'result') |
| parent_version_ids | UUID[] | NOT NULL DEFAULT '{}' |
| row_count | BIGINT | NULLABLE |
| file_id | UUID | FK -> files(id), NULLABLE |
| transformation_note | TEXT | NULLABLE |
| created_by | UUID | FK -> users(id), NOT NULL |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

`parent_version_ids` is a PostgreSQL UUID array storing zero or more parent version references, enabling a full lineage graph. A rollback creates a new version whose parent points to the target snapshot version.

#### `field_dictionaries`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| dataset_version_id | UUID | FK -> dataset_versions(version_id), NOT NULL |
| field_name | VARCHAR(128) | NOT NULL |
| field_type | VARCHAR(32) | NOT NULL |
| meaning | TEXT | NULLABLE |
| source_system | VARCHAR(64) | NULLABLE |
| last_updated_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

Unique constraint: `(dataset_version_id, field_name)`.

### 4.9 File Storage

#### `files`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| path | VARCHAR(512) | UNIQUE, NOT NULL |
| original_name | VARCHAR(256) | NOT NULL |
| size | BIGINT | NOT NULL |
| mime_type | VARCHAR(64) | NOT NULL |
| sha256 | VARCHAR(64) | NOT NULL |
| uploaded_by | UUID | FK -> users(id), NOT NULL |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

Index on `(sha256)` for duplicate detection.

### 4.10 Notifications

#### `notification_templates`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| code | VARCHAR(64) | UNIQUE, NOT NULL |
| subject_template | VARCHAR(256) | NOT NULL |
| body_template | TEXT | NOT NULL |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

#### `notifications`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| user_id | UUID | FK -> users(id), NOT NULL |
| template_id | UUID | FK -> notification_templates(id), NULLABLE |
| subject | VARCHAR(256) | NOT NULL |
| body | TEXT | NOT NULL |
| is_read | BOOLEAN | NOT NULL DEFAULT false |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

#### `notification_delivery_log`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| notification_id | UUID | FK -> notifications(id), NOT NULL |
| channel | VARCHAR(32) | NOT NULL DEFAULT 'in_app' |
| status | VARCHAR(16) | NOT NULL, CHECK IN ('pending', 'delivered', 'failed') |
| attempt_count | INTEGER | NOT NULL DEFAULT 0 |
| last_attempted_at | TIMESTAMPTZ | NULLABLE |
| delivered_at | TIMESTAMPTZ | NULLABLE |
| error_message | TEXT | NULLABLE |

### 4.11 Reporting

#### `report_definitions`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| name | VARCHAR(128) | NOT NULL |
| description | TEXT | NULLABLE |
| query_template | TEXT | NOT NULL |
| dimensions | JSONB | NOT NULL DEFAULT '[]' |
| filters | JSONB | NOT NULL DEFAULT '[]' |
| chart_config | JSONB | NULLABLE |
| created_by | UUID | FK -> users(id), NOT NULL |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

#### `report_runs`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| report_definition_id | UUID | FK -> report_definitions(id), NOT NULL |
| requested_by | UUID | FK -> users(id), NOT NULL |
| status | VARCHAR(16) | NOT NULL DEFAULT 'queued', CHECK IN ('queued', 'running', 'completed', 'failed') |
| progress_pct | SMALLINT | NOT NULL DEFAULT 0 |
| file_id | UUID | FK -> files(id), NULLABLE |
| parameters | JSONB | NULLABLE |
| error_message | TEXT | NULLABLE |
| started_at | TIMESTAMPTZ | NULLABLE |
| completed_at | TIMESTAMPTZ | NULLABLE |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

### 4.12 Audit & Health

#### `audit_log`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| user_id | UUID | FK -> users(id), NULLABLE |
| action | VARCHAR(64) | NOT NULL |
| resource_type | VARCHAR(64) | NOT NULL |
| resource_id | UUID | NULLABLE |
| before_hash | VARCHAR(64) | NULLABLE |
| after_hash | VARCHAR(64) | NULLABLE |
| metadata | JSONB | NULLABLE |
| ip_address | VARCHAR(45) | NULLABLE |
| timestamp | TIMESTAMPTZ | NOT NULL DEFAULT now() |

This table is append-only. No UPDATE or DELETE operations are permitted at the application level. Indexes: `(user_id, timestamp)`, `(resource_type, resource_id)`, `(action, timestamp)`.

#### `health_metrics`

| Column | Type | Constraints |
|---|---|---|
| id | UUID | PK |
| metric_name | VARCHAR(64) | NOT NULL |
| metric_value | DOUBLE PRECISION | NOT NULL |
| labels | JSONB | NULLABLE |
| recorded_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

Index: `(metric_name, recorded_at DESC)`. Stores locally queryable performance and operational metrics. A scheduled cleanup removes entries older than 7 days.

---

## 5. Authentication & Session Management

### 5.1 Password Hashing

All passwords are hashed using **Argon2id** with the following parameters:

- Memory cost: 64 MB
- Time cost (iterations): 3
- Parallelism: 1
- Output length: 32 bytes

The `argon2` crate generates a random 16-byte salt per hash. Hashes are stored in PHC string format in the `password_hash` column.

### 5.2 Password Policy

| Rule | Requirement |
|---|---|
| Minimum length | 12 characters |
| Uppercase letter | At least 1 |
| Lowercase letter | At least 1 |
| Digit | At least 1 |
| Reuse prevention | Cannot match any of the last 10 hashes in `password_history` |

Password validation occurs at registration and password-change endpoints. The `password_history` table is appended on every successful change.

### 5.3 Account Lockout

After **5 consecutive failed login attempts** within a **15-minute sliding window**, the account is locked for **15 minutes**. The lockout is recorded by setting `users.locked_until` to `now() + 15 minutes`. Subsequent login attempts against a locked account return `HTTP 423 Locked` without evaluating the password, preventing timing oracle attacks during lockout.

A System Admin may manually unlock an account before the lockout period expires.

### 5.4 Session Management

- On successful login, a 256-bit cryptographically random session token is generated using the `rand` crate's `OsRng`.
- The token is hashed with SHA-256 before storage in `sessions.token_hash`. The raw token is returned to the client once and never stored server-side.
- Session expiry defaults to **8 hours**. Each authenticated request that occurs within the final 60 minutes of the session window extends expiry by another 8 hours (sliding window).
- A user may have at most 3 concurrent sessions. Creating a 4th session revokes the oldest.
- Logout revokes the session by setting `sessions.revoked = true`.

### 5.5 Encrypted Sensitive Fields

Sensitive data at rest (e.g., participant phone numbers, email addresses where configured) is encrypted using AES-256-GCM with a locally managed key stored in a file at a path specified by the `ENCRYPTION_KEY_PATH` environment variable. The application reads this key at startup and holds it in memory. See section 15 for details.

### 5.6 Field Masking

API responses apply field masking based on the caller's role. The default masking strategy exposes only the **last 4 characters** of sensitive string fields, replacing preceding characters with asterisks. Example: a phone number `5551234567` becomes `******4567`. Masking rules are configured per-field and per-role in application configuration, not in the database schema.

---

## 6. Authorization & RBAC

### 6.1 Three-Layer Model

```
Role
  └── binds to ──> Permission Point (e.g., pos.refund.approve)
                       └── maps to ──> API Scope (e.g., POST /api/pos/orders/{id}/refund)
                       └── maps to ──> Menu Scope (e.g., pos.refund_panel)
```

**Layer 1 -- Roles:** Named collections of permission grants. Users are assigned one or more roles. System-defined roles are immutable; custom roles may be created by System Admins.

**Layer 2 -- Permission Points:** Fine-grained capabilities identified by dot-separated codes (e.g., `dataset.version.rollback`). Each permission point belongs to a category for organizational grouping.

**Layer 3 -- API Capability Scopes:** Concrete HTTP method + path pairs. The authorization middleware resolves the inbound request's method and path, looks up the required permission point via `api_scopes`, then checks whether the user's roles grant that permission point through `role_permission_bindings`.

### 6.2 Data-Scope Controls

Beyond capability-based access, every query is filtered by the user's data scope:

| Scope Type | Effect |
|---|---|
| `department` | User can only access resources tagged with their assigned department(s) |
| `location` | User can only access resources belonging to their assigned location(s) |
| `individual` | User can only access resources they personally created or own |

Data-scope rules are evaluated as an intersection: a user must satisfy all applicable scope rules. User-specific rules in `data_scope_rules` (where `user_id IS NOT NULL`) override role-level rules for that user.

### 6.3 Time-Bounded Delegation

A user (delegator) with a given role may delegate a subset of their permissions to another user (delegatee) for a defined time window.

- The delegation record specifies `start_at`, `end_at`, `scope_type`, and `scope_value`.
- The authorization middleware evaluates active delegations by checking `now() BETWEEN start_at AND end_at AND revoked = false`.
- A delegator may revoke a delegation early by setting `revoked = true`.
- Expired delegations are never deleted; they remain for audit purposes.
- A delegatee cannot further delegate permissions received via delegation.

### 6.4 Mandatory Approval Workflow

Certain actions are gated by an approval requirement configured in `approval_configs`:

| Action Code | Required Approver Role | Description |
|---|---|---|
| `pos.refund` | Store Manager | Refund on a completed sale |
| `pos.reversal` | Store Manager | Full order reversal |
| `dataset.rollback` | System Admin | Rolling back a dataset version |
| `report.bulk_export` | Store Manager or Analyst | Exporting more than 10K rows |

**Flow:**

1. The requesting user initiates the action. The system checks `approval_configs` for a matching `action_code`.
2. If approval is required and `enabled = true`, an `approval_requests` record is created with status `pending` and an `expires_at` timestamp (default: 24 hours).
3. The system sends a notification to eligible approvers.
4. An approver reviews and sets status to `approved` or `rejected`.
5. If approved, the original action is executed. If rejected or expired, it is abandoned.

**Self-approval prevention:** The database constraint `approver_id != requester_id` on `approval_requests` ensures that the person requesting an action cannot approve it themselves. The application layer enforces this before attempting the INSERT as well.

---

## 7. POS Transaction Engine

### 7.1 Order State Machine

```
                      +-----------+
                      |   Draft   |
                      +-----+-----+
                            |
                      add_line / edit_line
                            |
                      +-----v-----+
                      |   Open    |
                      +-----+-----+
                            |
                      begin_tender
                            |
                      +-----v-----+
                      | Tendering |
                      +-----+-----+
                            |
               tender_complete (sum >= total)
                            |
                      +-----v-----+
                      |   Paid    |
                      +-----+-----+
                            |
                      finalize
                            |
                      +-----v-----+
                      |  Closed   |
                      +-----+-----+
                            |
              +-------------+-------------+
              |                           |
       initiate_return             initiate_reversal
              |                           |
     +--------v--------+       +----------v----------+
     |Return Initiated |       | Reversal Pending    |
     +--------+--------+       +----------+----------+
              |                           |
       complete_return            approve_reversal
              |                           |
     +--------v--------+       +----------v----------+
     |    Returned      |       |     Reversed        |
     +-----------------+       +---------------------+
```

### 7.2 State Transition Rules

| From | To | Trigger | Conditions |
|---|---|---|---|
| Draft | Open | First line item added | At least one line item |
| Open | Tendering | `begin_tender` | Order has lines, subtotal > 0 |
| Tendering | Paid | `tender_complete` | Sum of tender amounts >= order total |
| Paid | Closed | `finalize` | Receipt generated and attached |
| Closed | Return Initiated | `initiate_return` | Within return window, approval obtained |
| Return Initiated | Returned | `complete_return` | Return line items validated, refund tender recorded |
| Closed | Reversal Pending | `initiate_reversal` | Within 24 hours, approval obtained |
| Reversal Pending | Reversed | `approve_reversal` | Manager approval, within 24-hour window |

Invalid transitions return `HTTP 409 Conflict` with the current state and the attempted transition in the response body.

### 7.3 Idempotency Key Implementation

All mutating POS endpoints (`POST`, `PUT`, `PATCH`) require an `Idempotency-Key` header.

1. On receiving a request, the middleware queries `idempotency_keys` for a matching `(key, endpoint)` pair.
2. **If found and not expired:** The stored `response_body` and `response_status` are returned immediately without executing the handler. This makes retries safe.
3. **If not found:** The handler executes normally. On success, the response is stored in `idempotency_keys` with a TTL of **24 hours** (`expires_at = now() + 24h`).
4. **If found but expired:** The expired row is deleted and the request is processed as new.
5. A background task purges expired keys every hour.

The `response_hash` column stores the SHA-256 hash of the serialized response body for quick integrity verification.

### 7.4 Mixed Tender Handling

An order can be paid with multiple tender types in sequence:

1. The cashier transitions the order to `tendering` state.
2. One or more `tender_entries` are created, each specifying `tender_type` and `amount`.
3. **Cash handling:** If a cash tender exceeds the remaining balance, `change_due` is set to `tender_amount - remaining_balance`. The `amount` recorded reflects only the portion applied to the order.
4. **Gift card handling:** The gift card `reference` is recorded. If the gift card balance is insufficient, a partial amount is applied and the remaining balance must be covered by additional tenders.
5. **Card handling:** Card tenders record the authorization `reference`.
6. When the sum of all tender entry amounts equals or exceeds the order total, the state transitions to `paid`.

### 7.5 Receipt Attachment

Upon transition to `paid`, a PDF receipt is generated using `printpdf`:

1. The receipt includes order number, date/time, location, cashier ID, line items, subtotal, tax, total, and tender breakdown.
2. The PDF is written to `/data/files/receipts/{order_id}.pdf`.
3. A `files` record is created with the SHA-256 hash.
4. A `receipts` record links the order to the file.

### 7.6 Return / Exchange / Reversal Flows

**Return:**
1. A return is initiated against a `closed` order by referencing `original_order_id`.
2. A new `pos_orders` record is created with status `return_initiated` and negative line amounts.
3. The approval workflow fires for `pos.refund`.
4. Upon approval, return tender entries are created (refund to original tender method) and the status moves to `returned`.

**Exchange:**
Handled as a paired return + new sale. The return order references the original; the new sale order stands independently. Both are linked by audit metadata.

**Reversal:**
1. A full reversal is initiated against a `closed` order within 24 hours.
2. The order enters `reversal_pending`.
3. The approval workflow fires for `pos.reversal`.
4. Upon manager approval, all tender entries are reversed, the order status becomes `reversed`, and a reversal receipt is generated.

---

## 8. End-of-Day Reconciliation

### 8.1 Register Close Flow

1. The cashier initiates a register close by submitting the `actual_amount` (physical cash count) for their register.
2. The system calculates `expected_amount` by summing all cash tender entries for that register, location, and shift date.
3. `variance = actual_amount - expected_amount`.
4. A `register_closes` record is created with status `pending`.

### 8.2 Variance Calculation

Variance is computed in cents:

```
expected_amount = SUM(tender_entries.amount)
    WHERE tender_type = 'cash'
      AND order.location_id = register_close.location_id
      AND order.cashier_id = register_close.cashier_id
      AND order.created_at::date = register_close.shift_date
      AND order.status IN ('closed', 'returned', 'reversed')

variance = actual_amount - expected_amount
```

Returned and reversed orders are included in the calculation because their tender entries (which may be negative) must offset the day's totals.

### 8.3 Manager Confirmation Threshold

- If `|variance| <= 2000` (i.e., $20.00 or less): The register close is auto-confirmed. Status is set to `confirmed` immediately.
- If `|variance| > 2000`: Status remains `pending` and a notification is sent to the Store Manager. The manager must review and explicitly confirm, at which point `manager_confirmed_by` and `confirmed_at` are populated and status moves to `confirmed`.
- If the manager determines the variance is unacceptable, they may set the status to `flagged` for further investigation.

### 8.4 24-Hour Reversal Rule

Order reversals are only permitted within 24 hours of the original order's `created_at` timestamp. After this window, the reversal endpoint returns `HTTP 422 Unprocessable Entity`. This ensures that register close calculations for prior days are not retroactively altered.

### 8.5 Auto-Close Policy

If a cashier does not initiate a register close by **02:00 local time** the following day, the system creates a `register_closes` record with `actual_amount = 0` and status `flagged`, and sends a notification to the Store Manager. This prevents reconciliation gaps.

---

## 9. Participant Data Management

### 9.1 Profile CRUD

Standard REST endpoints for participant lifecycle:

| Endpoint | Method | Description |
|---|---|---|
| `/api/participants` | GET | List participants with pagination, search, and filter |
| `/api/participants` | POST | Create a new participant |
| `/api/participants/{id}` | GET | Retrieve a single participant |
| `/api/participants/{id}` | PUT | Update participant fields |
| `/api/participants/{id}` | DELETE | Soft-delete (set `is_active = false`) |

### 9.2 Team & Roster Management

| Endpoint | Method | Description |
|---|---|---|
| `/api/teams` | GET | List all teams |
| `/api/teams` | POST | Create a team |
| `/api/teams/{id}` | PUT | Update team details |
| `/api/teams/{id}/members` | GET | List team members |
| `/api/teams/{id}/members` | POST | Add participant to team |
| `/api/teams/{id}/members/{member_id}` | DELETE | Remove participant from team |

### 9.3 Credential File Attachments

Participants may have credential files (certifications, ID scans) attached:

- Upload via `POST /api/participants/{id}/files` (multipart form, subject to file storage rules in section 13).
- List via `GET /api/participants/{id}/files`.
- The `files` table tracks the association through an additional `participant_files` join table or through a resource_type/resource_id pattern in application logic.

### 9.4 Search, Filter & Tag System

**Search:** Full-text search on `first_name`, `last_name`, and `email` fields using PostgreSQL `ILIKE` with parameterized patterns.

**Filters:** Query parameters support filtering by:
- `is_active` (boolean)
- `team_id` (UUID)
- `tag` (exact match on `participant_tags.tag`)
- `created_after` / `created_before` (date range)

**Tags:** Tags are arbitrary strings attached to participants via `participant_tags`. Endpoints:
- `POST /api/participants/{id}/tags` -- Add a tag
- `DELETE /api/participants/{id}/tags/{tag}` -- Remove a tag
- `GET /api/participants?tag=certified` -- Filter by tag

### 9.5 Bulk Operations

- `POST /api/participants/bulk` -- Create multiple participants from a JSON array or uploaded CSV/XLSX file.
- `PATCH /api/participants/bulk` -- Update fields across multiple participants by ID.
- `POST /api/participants/bulk/tag` -- Apply a tag to multiple participants.
- All bulk operations run inside a single database transaction. If any row fails validation, the entire batch is rolled back and the response includes per-row error details.

---

## 10. Data Storage & Version Management

### 10.1 Layered Dataset Types

Datasets progress through a defined lifecycle of types:

| Type | Description |
|---|---|
| `raw` | Original unmodified data as ingested from source systems |
| `cleaned` | Data after validation, deduplication, and normalization |
| `feature` | Derived columns, aggregations, and computed features for analysis |
| `result` | Final analytical output, KPI summaries, or model results |

Each dataset version has a `dataset_type` indicating its position in this pipeline. A version of type `cleaned` will typically have a `raw` version as its parent.

### 10.2 Immutable Version IDs

Every dataset version is assigned a UUID v4 at creation. Versions are **immutable** once created -- they cannot be updated or deleted. This guarantees reproducibility and auditability of the data pipeline.

### 10.3 Lineage Graph

The `parent_version_ids` array in `dataset_versions` records which prior versions contributed to the current version. This forms a directed acyclic graph (DAG):

```
raw_v1 ──> cleaned_v1 ──> feature_v1 ──> result_v1
                    \
                     ──> feature_v2 ──> result_v2
```

The `transformation_note` field on each version describes what processing step was applied. Lineage queries traverse the parent array recursively using PostgreSQL's `WITH RECURSIVE` CTE.

### 10.4 Rollback Mechanism

Rolling back a dataset version does **not** delete any data. Instead:

1. The user identifies a target version to restore.
2. A new version is created with:
   - `parent_version_ids` pointing to the rollback target.
   - `transformation_note` set to `"Rollback to version {target_version_id}"`.
   - `dataset_type` matching the target version's type.
   - `file_id` pointing to the same file as the target version (or a copy if file-level isolation is required).
3. The new version becomes the latest version of the dataset.

This operation requires `dataset.rollback` approval (see section 6.4).

### 10.5 Field Dictionary Per Version

Each dataset version has its own field dictionary (`field_dictionaries`), capturing the schema of that version. Fields include:

- `field_name`: Column or field identifier.
- `field_type`: Data type (e.g., `integer`, `varchar`, `decimal`, `timestamp`).
- `meaning`: Human-readable description of what the field represents.
- `source_system`: Which upstream system produced this field.
- `last_updated_at`: When the dictionary entry was last revised.

This allows schema evolution tracking across versions.

### 10.6 Indexing Strategy

| Table | Index | Purpose |
|---|---|---|
| `dataset_versions` | `(dataset_id, created_at DESC)` | Fetch latest version efficiently |
| `dataset_versions` | GIN on `parent_version_ids` | Lineage traversal (find children of a given version) |
| `field_dictionaries` | `(dataset_version_id, field_name)` | Unique constraint and lookup |
| `datasets` | `(owner_id)` | Filter datasets by owner |

---

## 11. Notification System

### 11.1 Template Engine

Notification templates use a simple variable substitution syntax: `{{variable_name}}`. The template engine replaces placeholders with values from a provided context map at render time.

Example template:
```
Subject: Approval Required: {{action_type}} by {{requester_name}}
Body: A {{action_type}} request (ID: {{request_id}}) requires your approval.
      Resource: {{resource_type}} / {{resource_id}}
      Requested at: {{requested_at}}
```

Templates are stored in `notification_templates` and referenced by `code`. The application resolves the template, substitutes variables, and stores the rendered result in `notifications`.

### 11.2 Trigger Events

Notifications are generated in response to the following system events:

| Event | Recipient(s) | Template Code |
|---|---|---|
| Approval request created | Eligible approvers | `approval_requested` |
| Approval granted | Requester | `approval_granted` |
| Approval rejected | Requester | `approval_rejected` |
| Register close flagged | Store Manager | `register_flagged` |
| Register auto-close triggered | Store Manager, Cashier | `register_auto_close` |
| Report generation completed | Requesting user | `report_completed` |
| Report generation failed | Requesting user | `report_failed` |
| Account locked | User, System Admin | `account_locked` |
| Password expiry approaching | User | `password_expiry_warning` |
| Delegation granted | Delegatee | `delegation_granted` |
| Delegation expiring soon | Delegatee, Delegator | `delegation_expiring` |

### 11.3 In-App Delivery

All notifications are delivered through an **in-app** channel. There are no email, SMS, or push notification integrations (the system is fully offline).

- `GET /api/notifications` -- List notifications for the authenticated user with pagination, optional `is_read` filter.
- `PATCH /api/notifications/{id}/read` -- Mark a notification as read.
- `GET /api/notifications/unread-count` -- Return count of unread notifications.

### 11.4 Retry Policy

In-app notifications are considered delivered upon successful database insertion, so retries are only relevant for future channel extensions. The `notification_delivery_log` tracks each delivery attempt:

- Maximum attempts: 3
- Backoff: 1 minute, 5 minutes, 30 minutes
- After 3 failures, status is set to `failed` and no further attempts are made.

### 11.5 Delivery Log Tracking

Every notification delivery attempt is recorded in `notification_delivery_log` with:
- The channel used
- Success or failure status
- Attempt count
- Timestamps for each attempt and final delivery
- Error message on failure

This provides a complete audit trail for notification delivery.

---

## 12. Reporting & Analytics

### 12.1 KPI Aggregate Queries

The reporting engine supports pre-defined KPI queries that aggregate POS transaction data:

| KPI | Query Description |
|---|---|
| Gross Sales | SUM of `order_lines.line_total` for orders in `closed` status |
| Net Sales | Gross Sales minus returns and reversals |
| Average Transaction Value | Net Sales / COUNT of closed orders |
| Transactions per Hour | COUNT of orders grouped by hour, location |
| Variance Rate | COUNT of flagged register closes / total register closes |
| Return Rate | COUNT of returned orders / COUNT of closed orders |
| Tender Mix | SUM of tender amounts grouped by tender_type |

### 12.2 Configurable Dimensions & Filters

Report definitions (`report_definitions`) store a JSON schema describing available dimensions and filters:

**Dimensions** (grouping axes):
- `location_id`
- `cashier_id`
- `shift_date`
- `hour_of_day`
- `tender_type`
- `department`

**Filters** (restriction predicates):
- Date range (`start_date`, `end_date`)
- Location(s)
- Cashier(s)
- Order status
- Minimum/maximum amount

The `query_template` in `report_definitions` contains parameterized SQL fragments that the engine assembles based on the selected dimensions and filters. All parameters are bound via Diesel's parameterized query interface.

### 12.3 Chart Definition Storage

The `chart_config` JSONB column on `report_definitions` stores chart rendering metadata:

```json
{
  "chart_type": "bar",
  "x_axis": "shift_date",
  "y_axis": "net_sales",
  "group_by": "location_id",
  "sort_order": "asc"
}
```

This metadata is returned to downstream clients for rendering. The API does not render charts itself.

### 12.4 Scheduled Report Generation

Reports can be scheduled for periodic generation:

- A cron-like configuration (stored in report metadata) defines the schedule.
- A background Tokio task checks for due reports every minute and enqueues `report_runs` with status `queued`.
- A worker picks up `queued` runs, transitions them to `running`, executes the query, writes the output file, and transitions to `completed` or `failed`.

### 12.5 Async Export Pipeline

For large exports (up to **250,000 rows**):

1. The user initiates an export via `POST /api/reports/{id}/run` with desired parameters.
2. A `report_runs` record is created with status `queued`.
3. A background Tokio task picks up the job:
   a. Transitions status to `running`.
   b. Executes the query in batches of 5,000 rows using `LIMIT/OFFSET` pagination.
   c. Updates `progress_pct` after each batch: `(rows_processed / estimated_total) * 100`.
   d. Writes rows to a temporary file (CSV or XLSX via `xlsxwriter`).
4. On completion, the file is moved to `/data/files/exports/`, a `files` record is created, and the `report_runs` record is updated to `completed` with the `file_id`.
5. On failure, the error is captured in `error_message` and status is set to `failed`.

**Progress tracking:** Clients poll `GET /api/reports/runs/{run_id}` to check `status` and `progress_pct`.

**Resumability:** If the process is interrupted (e.g., server restart), the run remains in `running` status. On startup, the system detects stale `running` jobs (no progress update for > 5 minutes) and resets them to `queued` for retry.

---

## 13. File Storage

### 13.1 Local Disk Layout

```
/data/
  files/
    receipts/       # Generated POS receipts (PDF)
    exports/        # Report export files (CSV, XLSX)
    uploads/        # User-uploaded files (credentials, imports)
    datasets/       # Dataset version data files
  logs/
    app/            # Application log files
```

All paths stored in the `files.path` column are relative to `/data/files/` to enable volume remapping.

### 13.2 DB Pointer Table

The `files` table (section 4.9) serves as the authoritative registry. No file exists on disk without a corresponding database record. File access is always mediated through the API; direct filesystem access is not exposed.

### 13.3 Size Limit

Maximum file size: **10 MB** per upload. The limit is enforced at the Actix-web layer using a content-length check before reading the body. Requests exceeding the limit receive `HTTP 413 Payload Too Large`.

### 13.4 Allowed Types

| MIME Type | Extension | Use Case |
|---|---|---|
| `application/pdf` | .pdf | Receipts, credential documents, reports |
| `image/jpeg` | .jpg | Credential photos, supporting documents |
| `image/png` | .png | Credential photos, supporting documents |
| `text/csv` | .csv | Data imports, report exports |
| `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet` | .xlsx | Data imports, report exports |

Uploads with unlisted MIME types are rejected with `HTTP 415 Unsupported Media Type`. The MIME type is validated by inspecting both the `Content-Type` header and the file's magic bytes.

### 13.5 SHA-256 Fingerprinting

On upload, the file's SHA-256 hash is computed and stored in `files.sha256`. This hash serves three purposes:

1. **Duplicate detection:** Before writing to disk, the hash is checked against existing `files.sha256` values. If a match exists, the upload is deduplicated by referencing the existing file record (a new `files` row may still be created for distinct logical ownership, but the physical file is not duplicated).
2. **Tampering check:** On file download, the hash is recomputed from disk and compared to the stored value. A mismatch returns `HTTP 500 Internal Server Error` and creates an audit-log entry with action `file.integrity_failure`.
3. **Audit linking:** The `audit_log.after_hash` field can reference file hashes for change tracking.

---

## 14. Audit Trail

### 14.1 Immutable Append-Only Log

The `audit_log` table accepts only INSERT operations at the application level. The Diesel schema definition does not expose `update` or `delete` functions for this table. PostgreSQL-level row security policies may be applied as a defense-in-depth measure to deny UPDATE and DELETE from the application's database role.

### 14.2 Before / After SHA-256 Hashes

For every mutating operation (create, update, delete) on a governed resource, the audit log captures:

- `before_hash`: SHA-256 of the serialized JSON representation of the resource **before** the operation. NULL for create operations.
- `after_hash`: SHA-256 of the serialized JSON representation of the resource **after** the operation. NULL for delete operations.

This enables detection of any discrepancy between the audit trail and the current database state.

### 14.3 Full Row Hashing

Row hashing is performed by:

1. Serializing the row to a canonical JSON representation (keys sorted alphabetically, no whitespace).
2. Computing SHA-256 of the resulting byte string.

This deterministic process ensures that the same row state always produces the same hash, enabling integrity verification at any point in time.

### 14.4 Retention Policy

Audit log entries are retained for a minimum of **7 years** to satisfy retail compliance requirements. No automated deletion occurs. Archival to compressed file exports (written to `/data/files/audit-archive/`) may be performed manually by a System Admin for entries older than 2 years, but the database records are preserved.

### 14.5 Queryable by User / Time / Action / Resource

The audit log supports efficient querying through dedicated indexes:

| Query Pattern | Index |
|---|---|
| All actions by a specific user | `(user_id, timestamp)` |
| All actions on a specific resource | `(resource_type, resource_id)` |
| All actions of a specific type | `(action, timestamp)` |
| Time-range scans | `(timestamp)` |

API endpoint: `GET /api/audit-log` with query parameters `user_id`, `action`, `resource_type`, `resource_id`, `from`, `to`, and standard pagination (`page`, `per_page`). Access is restricted to the System Admin role.

---

## 15. Security

### 15.1 Encryption at Rest

Sensitive fields identified for encryption (e.g., participant phone, participant email) are encrypted using **AES-256-GCM** before storage in PostgreSQL.

- **Key management:** A 256-bit encryption key is stored in a file on the local filesystem at the path specified by the `ENCRYPTION_KEY_PATH` environment variable. The file is readable only by the application's OS user (mode `0400`).
- **Implementation:** Each encrypted field is stored as a base64-encoded string containing the 12-byte nonce prepended to the ciphertext. Decryption extracts the nonce and ciphertext, then decrypts using the in-memory key.
- **Key rotation:** A key rotation utility re-encrypts all sensitive fields with a new key. During rotation, both old and new keys are held in memory. A version byte prefixed to each ciphertext identifies which key was used.

### 15.2 Field Masking Rules Per Role

Field masking is applied at the serialization layer, after data is fetched from the database but before the JSON response is constructed.

| Field | Cashier | Store Manager | System Admin | Analyst |
|---|---|---|---|---|
| Participant phone | `******4567` | `******4567` | Full value | `******4567` |
| Participant email | `****@domain` | Full value | Full value | `****@domain` |
| Tender card reference | `****1234` | `****1234` | Full value | `****1234` |
| User password_hash | Never exposed | Never exposed | Never exposed | Never exposed |

Masking configuration is defined in a static application configuration file loaded at startup. The `password_hash` field is excluded from all API response serializations.

### 15.3 Parameterized Queries

All database queries are constructed through Diesel's query builder, which enforces parameterized query construction at compile time. Raw SQL strings are not used in application code. This eliminates SQL injection by design.

### 15.4 CSRF Considerations

CSRF protection is **not implemented** because the system is a backend API only. There are no browser-rendered forms, no cookies used for authentication (session tokens are passed in the `Authorization` header), and no same-origin policy concerns. API clients authenticate via bearer token headers, which are not automatically attached by browsers.

### 15.5 Rate Limiting

Rate limiting is enforced at the Actix-web middleware layer:

| Scope | Limit | Window |
|---|---|---|
| Per IP (unauthenticated) | 30 requests | 1 minute |
| Per session (authenticated) | 300 requests | 1 minute |
| Login endpoint | 10 requests per IP | 1 minute |

Rate limit state is stored in an in-memory data structure (not in the database) to avoid adding query overhead. Exceeding the limit returns `HTTP 429 Too Many Requests` with a `Retry-After` header.

---

## 16. Observability

### 16.1 Structured JSON Logging

All application logs are emitted as structured JSON objects to stdout, which Docker captures and writes to `/data/logs/app/`. Each log entry contains:

```json
{
  "timestamp": "2026-04-07T14:30:00.000Z",
  "level": "INFO",
  "module": "pos::orders",
  "message": "Order finalized",
  "request_id": "a1b2c3d4-...",
  "user_id": "e5f6a7b8-...",
  "order_id": "c9d0e1f2-...",
  "duration_ms": 42
}
```

Log levels: `ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE`. Production default is `INFO`. The `RUST_LOG` environment variable controls level filtering per module.

### 16.2 Health Endpoint

`GET /health` returns the operational status of the application:

```json
{
  "status": "healthy",
  "uptime_seconds": 86400,
  "database": "connected",
  "filesystem": "writable",
  "timestamp": "2026-04-07T14:30:00.000Z"
}
```

Checks performed:
1. **Database:** Executes `SELECT 1` on the connection pool. If the query fails or times out (> 2 seconds), status is `unhealthy`.
2. **Filesystem:** Attempts to write and delete a temporary file in `/data/files/`. If it fails, status is `degraded`.

Returns `HTTP 200` when healthy, `HTTP 503` when unhealthy.

### 16.3 Metrics Endpoint

`GET /metrics` returns operational metrics as JSON (no Prometheus format, no third-party dependencies):

```json
{
  "request_count_total": 1048576,
  "request_count_by_status": {
    "2xx": 1000000,
    "4xx": 45000,
    "5xx": 3576
  },
  "p95_latency_ms": 187,
  "p99_latency_ms": 290,
  "db_pool_size": 20,
  "db_pool_active": 4,
  "db_pool_idle": 16,
  "active_sessions": 42,
  "uptime_seconds": 86400
}
```

Metrics are computed from in-memory counters and histograms updated on every request by middleware. The `health_metrics` table provides persistent storage for historical metric snapshots recorded every 60 seconds.

**No third-party dependencies:** No Prometheus client, no StatsD, no OpenTelemetry SDK. All metric collection is implemented with standard library atomics and in-memory structures.

---

## 17. Deployment & Performance

### 17.1 Single-Node Docker

The entire system runs in a single Docker container (or a Docker Compose stack with two services: application and database) on one host.

### 17.2 Dockerfile Design

```dockerfile
# Build stage
FROM rust:1.80-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY migrations/ migrations/
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/retailops /usr/local/bin/retailops
COPY --from=builder /app/migrations /app/migrations
RUN mkdir -p /data/files/receipts /data/files/exports /data/files/uploads /data/files/datasets /data/logs/app
EXPOSE 8080
ENV DATABASE_URL=postgres://retailops:password@db:5432/retailops
ENV ENCRYPTION_KEY_PATH=/run/secrets/encryption_key
ENV RUST_LOG=info
CMD ["retailops"]
```

The multi-stage build keeps the runtime image minimal. Only the compiled binary, migrations, and PostgreSQL client library are included.

### 17.3 Docker Compose

```yaml
version: "3.8"
services:
  app:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - app-data:/data
    environment:
      DATABASE_URL: postgres://retailops:${DB_PASSWORD}@db:5432/retailops
      ENCRYPTION_KEY_PATH: /run/secrets/encryption_key
      RUST_LOG: info
    secrets:
      - encryption_key
    depends_on:
      db:
        condition: service_healthy

  db:
    image: postgres:16-alpine
    volumes:
      - pg-data:/var/lib/postgresql/data
    environment:
      POSTGRES_DB: retailops
      POSTGRES_USER: retailops
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U retailops"]
      interval: 5s
      timeout: 3s
      retries: 5

volumes:
  app-data:
  pg-data:

secrets:
  encryption_key:
    file: ./secrets/encryption_key
```

### 17.4 Connection Pooling

Database connections are managed by **r2d2** (the default Diesel pool manager):

| Parameter | Value | Rationale |
|---|---|---|
| `max_size` | 20 | Supports 200 concurrent users with connection reuse |
| `min_idle` | 5 | Avoids cold-start connection latency |
| `connection_timeout` | 5 seconds | Prevents request stalling on pool exhaustion |
| `idle_timeout` | 300 seconds | Reclaims unused connections |

If the application requires async pool management, **deadpool-diesel** may be used as a drop-in replacement to integrate with Tokio's async runtime. The choice between r2d2 (synchronous, used with `web::block`) and deadpool (async-native) is made at build time.

### 17.5 Performance Targets

| Metric | Target |
|---|---|
| p95 response latency | < 300 ms |
| Concurrent users | 200 |
| POS order throughput | 100 orders/minute |
| Report export (250K rows) | < 5 minutes |
| Health check response | < 50 ms |

### 17.6 Resource Sizing

Recommended minimum hardware for the target workload:

| Resource | Minimum | Recommended |
|---|---|---|
| CPU | 4 cores | 8 cores |
| RAM | 8 GB | 16 GB |
| Disk | 100 GB SSD | 250 GB SSD |
| Network | 1 Gbps LAN | 1 Gbps LAN |

PostgreSQL is configured with:
- `shared_buffers`: 2 GB (25% of recommended RAM)
- `effective_cache_size`: 8 GB (50% of recommended RAM)
- `work_mem`: 64 MB
- `maintenance_work_mem`: 512 MB
- `max_connections`: 50 (headroom above pool max_size)

### 17.7 Startup Sequence

1. Read environment variables and encryption key from disk.
2. Initialize the database connection pool.
3. Run pending Diesel migrations (`diesel_migrations::run_pending_migrations`).
4. Seed default roles and permission points if the `roles` table is empty.
5. Start the background Tokio tasks (idempotency key cleanup, report job runner, metric recorder, stale job recovery).
6. Bind the Actix-web HTTP server to `0.0.0.0:8080` and begin accepting requests.
7. Log startup completion with configuration summary (pool size, log level, migration count).

---

*End of design document.*
