# RetailOps — Data & Transaction Governance API

Offline-first REST API for multi-location retail operations: POS transactions, participant management, dataset versioning, notifications, reporting, and full audit trail.

## Prerequisites

- Docker Engine 24+ and Docker Compose v2+
- No external network dependencies — fully offline operation

## Quick Start

```bash
docker compose up -d      # Starts PostgreSQL + API
curl localhost:8081/api/v1/health   # Verify healthy

# Create initial admin user (one-time, only works with empty DB)
curl -X POST localhost:8081/api/v1/auth/bootstrap \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"SecurePass123!"}'
```

## Configuration (Environment Variables)

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | — | PostgreSQL connection string |
| `JWT_SECRET` | Yes | — | Secret for signing JWT tokens |
| `FIELD_ENCRYPTION_KEY` | Yes | — | Base64-encoded 32-byte AES-256 key |
| `BIND_ADDR` | No | `0.0.0.0:8080` | Server bind address |
| `RUST_LOG` | No | `info` | Log level (debug/info/warn/error) |
| `FILE_STORAGE_PATH` | No | `./data/uploads` | Local file storage directory |
| `JWT_ACCESS_TTL_SECS` | No | `900` | Access token TTL (seconds) |
| `JWT_REFRESH_TTL_SECS` | No | `86400` | Refresh token TTL (seconds) |

## Database Migrations

Migrations run automatically on startup via embedded Diesel migrations. To inspect:

```bash
docker compose exec db psql -U retailops -d retailops -c "\dt"
```

39 migrations create the full schema (roles, users, orders, participants, datasets, notifications, reports, audit_log).

## Running Tests

```bash
bash run_tests.sh          # Full suite: unit tests + API integration tests
bash unit_tests/run_unit_tests.sh   # Unit tests only (63 tests)
bash API_tests/run_api_tests.sh     # API tests only (112 tests, requires running services)
```

Unit tests run inside a Docker container (no local Rust toolchain needed).
API tests require `docker compose up -d` to be running.

## API Entry Points

| Scope | Base Path | Auth |
|-------|-----------|------|
| Health | `GET /api/v1/health` | Public |
| Metrics | `GET /api/v1/metrics` | `system.health` |
| Auth | `/api/v1/auth/*` | Public |
| Users | `/api/v1/users` | `user.*` |
| Roles | `/api/v1/roles` | `role.*` |
| Permissions | `/api/v1/permissions` | `permission.*` |
| Orders (POS) | `/api/v1/orders` | `order.*` |
| Register Close | `/api/v1/registers` | `register.*` |
| Participants | `/api/v1/participants` | `participant.*` |
| Teams | `/api/v1/teams` | `team.*` |
| Datasets | `/api/v1/datasets` | `dataset.*` |
| Notifications | `/api/v1/notifications` | `notification.*` |
| Reports | `/api/v1/reports` | `report.*` |
| Exports | `/api/v1/exports` | `report.export.*` |
| Audit | `/api/v1/audit` | `audit.read` |

## Security Architecture

- **Authentication**: Local JWT (Argon2id password hashing, AES-256-GCM encrypted at rest)
- **RBAC**: Three-layer enforcement — Role → Permission Point → API Capability
- **Data Scope**: Department / Location / Individual level access controls on all object endpoints
- **CSRF**: Requires `Content-Type: application/json` or `X-CSRF-Token` header on writes
- **Audit**: Immutable log with before/after SHA-256 state hashes on all write operations
- **Account Lockout**: 5 failed attempts → 15 minute lockout
- **Field Masking**: Sensitive fields show only last 4 characters in responses

## Security Enforcement Notes

- **Independent Approver Rule**: The approval decision flow (`POST /approvals/{id}/approve` and `/reject`) rejects attempts where `approver_user_id == requester_user_id` with HTTP 403. This prevents self-approval on all critical actions including reversals, register variance confirmations, dataset rollbacks, and bulk exports.
- **Object-Level Scope on POS Mutations**: All return, exchange, reversal (initiate + execute) handlers enforce `PermissionContext::enforce_scope` against the target order's `cashier_user_id`, `department`, and `location`. Cross-scope users receive 403.
- **Request-Aware API Capability Checks**: Critical mutation endpoints (reversals, register close/confirm, dataset rollback, orders, exports) use `check_permission_for_request(method, path)` which enforces layer-3 RBAC matching against `api_capabilities.http_method` + `path_pattern`. Capabilities seeded in migrations 41 and 44.
- **Hashed Write-Audit Coverage**: Critical write handlers record `before_hash` and `after_hash` via `audit_write()`. Covered: order create/transition/payment, reversal execute, register confirm, dataset rollback, role CRUD, participant CRUD, bulk operations, receipt attach, export complete. Sensitive fields (passwords, encryption keys) are never included in audit payloads.

## Final Compliance Notes

- **Autonomous Async Export Processing**: An in-process background worker (`export_worker.rs`) polls for queued export jobs every 5 seconds and processes them autonomously. Non-bulk jobs transition Queued → Running → Completed without manual `/complete` calls. Approval-gated jobs (>250k rows) only start after approval status is `Approved`. The admin `/complete` endpoint remains available for external worker integration.
- **Atomic Idempotency**: Accounting-impacting writes (payments, reversals) use `reserve_idempotency_key()` to atomically claim the idempotency key via a DB INSERT with PK constraint inside the transaction. If two concurrent requests race on the same key, only the first INSERT succeeds — the second gets a conflict response. After successful mutation, `finalize_idempotency()` updates the placeholder with the real response for replay.
- **Transition Scope Enforcement**: `transition_order` enforces `PermissionContext::enforce_scope` against the target order's `cashier_user_id`, `department`, and `location` before any state mutation. Cross-scope transition attempts receive 403.
- **Critical Write Audit Hash Guarantees**: All critical write paths capture before-state (for updates/mutations) and after-state (for all writes) as SHA-256 hashes in the audit log. This covers order transitions, payments, reversals, register confirmations, dataset rollbacks, role management, participant management, and export completion.
- **Layer-3 RBAC Coverage**: Request-aware `check_permission_for_request(method, path)` is enforced on orders (8 endpoints), exports (9 endpoints), reversals (4 endpoints), register (2 endpoints), dataset rollback (1 endpoint), and approvals (3 endpoints). API capabilities are seeded in migrations 41 and 44.

## Critical Action Approval Gates

These actions require approval workflow (cannot be bypassed):

| Action | Permission | Approver |
|--------|-----------|----------|
| Order Reversal | `order.reverse` | Store Manager |
| Late Reversal (>24h) | `order.reverse_late` | Store Manager |
| Register Variance >$20 | `register.confirm_variance` | Store Manager |
| Dataset Rollback | `dataset.rollback` | System Administrator |
| Bulk Export >250k rows | `report.export.bulk` | System Administrator |

Reversal flow: `POST /orders/{id}/reversals` creates approval request → manager approves → `POST /orders/{id}/reversals/execute` performs financial mutation.

## File Storage

- **Path**: Configured via `FILE_STORAGE_PATH` (default `./data/uploads`)
- **Structure**: `{base}/{category}/{entity_id}/{uuid}.{ext}`
- **Constraints**: Max 10 MB per file; allowed types: PDF, JPG, PNG, CSV, XLSX
- **Integrity**: SHA-256 fingerprint stored per file; duplicates detected
- **Safety**: Path traversal protection; server-managed paths only; no caller-provided absolute paths accepted
- **Receipts**: `POST /orders/{id}/receipts` accepts `multipart/form-data` with a `file` field (PDF/JPG/PNG/CSV/XLSX). Receipt file stored on disk with SHA-256 hash. Duplicate hash detection per order.
- **Exports**: `POST /exports/{id}/complete` accepts `file_content_base64` (base64-encoded artifact content). Server stores the artifact in managed storage — never accepts caller-provided file paths.

## API Contract Notes

- **Receipt attach** (`POST /orders/{id}/receipts`): Changed from JSON body to `multipart/form-data`. Fields: `file` (binary), optional `receipt_data` (JSON metadata).
- **Export complete** (`POST /exports/{id}/complete`): Changed from `file_path` to `file_content_base64`. Server manages all storage paths.
- **Export/job access**: `GET /exports/{id}` and `GET /exports/{id}/download` enforce owner-or-admin access. Non-owners need `report.export.admin` permission.
- **RBAC layer 3**: Request-aware capability enforcement is active on order, export, and approval routes. `check_permission_for_request(method, path)` enforces method+path matching against `api_capabilities`. Migration 41 seeds baseline capabilities for these routes.
- **Approval request creation** (`POST /approvals`): Now requires `approval.request.create` permission (System Administrator or Store Manager only).
- **Export checksum**: Export jobs now include `sha256_hash` field computed from the actual artifact content on completion.

## Docker Deployment

```bash
docker compose up -d          # Start (builds if needed)
docker compose down -v        # Stop and remove volumes
docker compose logs api       # View API logs
```

Single-node deployment. PostgreSQL data persisted in `pgdata` volume; uploads in `uploads` volume.
