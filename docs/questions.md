# RetailOps Data & Transaction Governance API — Open Questions

This document captures unresolved business-logic questions for the RetailOps backend API (Rust / Actix-web + Diesel, PostgreSQL, single-node Docker, fully offline). Each entry records the ambiguity, our working assumption, and the solution we will implement unless stakeholders direct otherwise.

---

## 1. Order State Machine — Backward Transitions and Partial Payments

**Question:** Can an order move backwards in the state machine (e.g., Paid back to Tendering)? How are partial payments represented — does the order stay in Tendering until the full amount is covered, or is there a distinct "Partially Paid" state?

**My Understanding:** The state machine is strictly forward-only under normal flow. A backward move such as Paid to Tendering would only occur through an explicit Reversal, which has its own dedicated path (Reversal Pending → Reversed). Partial payments are not a separate state; the order remains in Tendering while tender line items accumulate, and it transitions to Paid only once the sum of ledger entries meets or exceeds the order total.

**Solution:** We will enforce forward-only transitions in the order state machine. Any attempt to regress a state will be rejected with a `409 Conflict`. Partial payments will be tracked as individual tender ledger entries against the order while it remains in the Tendering state. A `remaining_balance` field on the order will be recalculated on every tender entry, and the transition to Paid fires automatically when `remaining_balance <= 0`. Reversal is the only mechanism that effectively "undoes" a Paid order, and it follows the Reversal Pending → Reversed path with its own audit trail.

---

## 2. Idempotency Key Scope and TTL

**Question:** Is the idempotency key scoped per-endpoint, per-order, or globally unique? What is the time-to-live before a key can safely be reused? How do we handle a request that arrives with a known key but different payload content?

**My Understanding:** Idempotency keys are scoped per-endpoint (i.e., the same key string may appear on two different endpoints without collision). The TTL should be long enough to survive transient retries but short enough to allow key reuse across business days. A key paired with a different payload is an error, not a replay.

**Solution:** We will scope idempotency keys to the combination of `(endpoint_path, idempotency_key)` and store them in a dedicated `idempotency_keys` table with columns for endpoint, key, request_hash (SHA-256 of the serialized payload), response_status, response_body, and created_at. TTL will be set to 24 hours; a background cleanup job will purge expired rows once per hour. If a request arrives with a recognized key but a different request_hash, the API will return `422 Unprocessable Entity` with a clear error message indicating payload mismatch. Replays with a matching hash will return the stored response without re-executing the write.

---

## 3. Mixed Tenders — Cash Overpayment Change and Partial Gift Card Balances

**Question:** When a customer pays with cash that exceeds the order total, how is change calculated and recorded? When a gift card has insufficient balance to cover the full amount, how is the remaining gift card balance tracked after a partial redemption?

**My Understanding:** Cash overpayment results in a change-due amount that is logged as a separate ledger entry of type "change." Gift cards are internal ledger instruments (no external processing), so partial redemption simply reduces the card's stored balance in our database.

**Solution:** Each tender entry will have a `tendered_amount` and an `applied_amount`. For cash, if `tendered_amount > applied_amount`, the difference is recorded as a `change_due` ledger entry linked to the same order. The order's `remaining_balance` is reduced only by `applied_amount`. For gift cards, the `gift_cards` table will maintain a `current_balance` column. On partial redemption, a row-level lock will debit the card balance by the `applied_amount` and create the corresponding tender ledger entry. If the gift card balance is less than the remaining order balance, the tender entry covers only the available balance and the order stays in Tendering for additional tenders. All balance mutations are wrapped in the same database transaction as the order state change.

---

## 4. End-of-Day Reconciliation — Unclosed Registers

**Question:** If a cashier's register has not been closed by the end of the business day, does the system auto-close at midnight, block the next day's operations until closed, or leave it open indefinitely?

**My Understanding:** Leaving a register open indefinitely creates reconciliation gaps, while auto-closing may hide discrepancies. The safest approach is to auto-close the register at a configurable cutoff time and flag it for manager review, ensuring next-day operations are never blocked.

**Solution:** We will implement a scheduled job (configurable, defaulting to midnight local time per location) that auto-closes any open register sessions for that location. Auto-closed sessions will be marked with a `closure_type = 'auto'` flag (as opposed to `'manual'`) and will automatically generate a reconciliation variance record requiring manager review. The system will not block next-day register opens. An in-app notification will be sent to the location manager listing all auto-closed registers. The audit trail will record the auto-close event with a system actor identifier.

---

## 5. Reversal After 24 Hours — Per-Reversal vs. Batch Approval

**Question:** The spec requires manager second-confirmation for reversals after 24 hours. Does this approval apply individually per reversal, or can a manager issue a blanket approval covering a batch of reversals at once?

**My Understanding:** From a governance standpoint, each reversal after the 24-hour window represents a distinct financial risk event. Blanket approval would weaken the control. However, requiring individual approval for large batches (e.g., system-error corrections) could be operationally burdensome.

**Solution:** The default behavior will be per-reversal approval: each reversal older than 24 hours generates its own approval request that a manager must individually confirm. To handle batch scenarios, we will also support a "batch approval" endpoint where a manager can approve multiple pending reversals in a single request, but each reversal ID must be explicitly listed in the payload (no wildcard or "approve all" option). Every approval — individual or batch — will be recorded in the audit trail with the manager's identity, timestamp, and the list of reversal IDs covered. Self-approval is prohibited (the approving manager cannot be the same user who initiated the reversal).

---

## 6. Time-Bounded Delegation — Expiry During Active Sessions

**Question:** If a delegated permission expires while the delegate is in the middle of an action (e.g., halfway through submitting a bulk export that was authorized under the delegation), does the system abort the in-flight operation, allow it to complete, or reject only subsequent requests?

**My Understanding:** Abruptly terminating in-flight database transactions could leave data in an inconsistent state. It is safer to let an already-admitted request complete and enforce expiry only at the next authorization check.

**Solution:** Delegation validity will be checked at request admission time (i.e., in the authorization middleware). Once a request passes the middleware and begins processing, it will be allowed to complete even if the delegation window closes during execution. Any subsequent request from the same delegate will be re-evaluated against the current delegation state and rejected if expired. For long-running async operations (e.g., bulk exports), the delegation check occurs when the job is enqueued, not when individual chunks are processed. The delegation record will include `started_at`, `expires_at`, and an `is_active` flag. An audit entry will be written when a delegation expires, noting any jobs that were admitted before expiry and completed after.

---

## 7. Mandatory Approval Toggles — Configuration Authority and Self-Approval

**Question:** Who has the authority to configure which actions require mandatory approval? Can an approver approve their own request (self-approval)?

**My Understanding:** Only top-level administrators should be able to toggle which actions require approval, since misconfiguration could bypass critical financial controls. Self-approval defeats the purpose of the control and should be prohibited.

**Solution:** Approval toggle configuration will be restricted to users holding a dedicated `system_config:approval_policy` permission, which will be assignable only to the top-tier administrative role. Changes to approval toggles are themselves audited and require a confirmation step. Self-approval will be explicitly blocked: the API will reject any approval request where `approver_user_id == requester_user_id` with a `403 Forbidden` and a descriptive error. In the event that only one administrator exists (bootstrapping scenario), a configurable `allow_self_approval_for_bootstrap` flag (default false) can be enabled via environment variable, and its usage will be recorded in the audit trail with a warning-level log entry.

---

## 8. Dataset Rollback — New Version vs. Destructive Revert

**Question:** When a dataset is rolled back to a prior version, does the system create a new version whose content points to the old data (preserving the full history), or does it destructively remove intermediate versions and revert in place?

**My Understanding:** Given the system's emphasis on immutable version IDs, lineage tracking, and auditability, destructive revert would violate the design principles. Rollback should be a forward-moving operation that creates a new version.

**Solution:** Rollback will create a new version entry with a new `version_id`, where the data content is a copy of (or pointer to) the target rollback version. The new version's lineage record will set `parent_version_id` to the version being rolled back to, and `transformation_note` will indicate "rollback from version X to version Y." All intermediate versions remain intact and queryable. The `created_by` field will reflect the user who initiated the rollback, and `created_at` will be the current timestamp. This approach preserves full audit history and allows rollback-of-rollback if needed. The rollback action, if configured as a critical action, will require mandatory approval before execution.

---

## 9. Dataset Lineage — Multiple Parents and Merge Conflicts

**Question:** Can a dataset version have multiple parent versions (e.g., a merge of two cleaned datasets into one feature dataset)? If so, how are schema or data conflicts between parents handled?

**My Understanding:** Real-world data pipelines frequently merge multiple sources, so supporting multiple parents is practical. However, the current schema description mentions "parent versions" (plural in the lineage links), suggesting this was anticipated but the conflict resolution strategy is undefined.

**Solution:** We will support multiple parent versions by modeling the lineage relationship as a many-to-many join table (`dataset_version_lineage`) with columns `child_version_id`, `parent_version_id`, and `merge_order`. When a new version is created from multiple parents, the API caller must supply the final merged schema and data — the system will not attempt automatic conflict resolution. The `transformation_note` field should describe the merge strategy used. The field dictionary for the new version must be explicitly provided and will be validated for internal consistency (no duplicate field names, all referenced types are valid). If the caller omits a field dictionary, the request will be rejected with a `400 Bad Request`. This keeps the system deterministic and auditable without embedding opinionated merge logic.

---

## 10. Field Dictionary — Descriptive Metadata vs. Schema Enforcement

**Question:** Is the field dictionary purely descriptive metadata (documentation for analysts), or does it actively enforce schema validation when data is written to a dataset version?

**My Understanding:** Given that the system manages layered datasets with lineage and rollback, purely descriptive metadata would leave data integrity to the caller. Enforcement adds safety but also rigidity. A hybrid approach is likely most practical.

**Solution:** The field dictionary will operate in two modes controlled by a per-dataset `schema_enforcement` flag (default: `descriptive`). In `descriptive` mode, the dictionary is stored as metadata and returned with dataset queries but does not block writes with non-conforming data. In `enforced` mode, every write to the dataset version will be validated against the dictionary: field names must match, data types must be compatible, and required/nullable constraints are checked. Validation failures return `422 Unprocessable Entity` with details of each violation. The mode can be set at dataset creation and changed by users with the appropriate permission. Mode changes are audited. Raw-layer datasets will default to `descriptive` (since raw data may be messy), while cleaned and feature layers will default to `enforced`.

---

## 11. Participant Credential Attachments — Retention and Deletion Policy

**Question:** What is the retention policy for participant credential file attachments? When a participant is removed or a credential expires, are attachments soft-deleted (marked inactive but retained) or hard-purged from disk and database?

**My Understanding:** Credential documents (e.g., certifications, licenses) may be subject to labor-law or corporate-policy retention requirements. Hard-purging could violate compliance obligations, while retaining everything indefinitely creates storage and privacy concerns.

**Solution:** We will implement soft-delete as the default: deleted credential attachments will be flagged with `deleted_at` timestamp and excluded from normal queries, but the file remains on disk and the database record is preserved. A configurable retention period (default: 7 years, overridable per credential type via system configuration) determines how long soft-deleted files are retained. After the retention period, a scheduled cleanup job will hard-purge the file from disk and mark the database record as `purged`. Active (non-deleted) credentials are never automatically purged. All delete and purge events are recorded in the audit trail. An API endpoint will allow authorized administrators to trigger early hard-purge for specific records if legally required (e.g., GDPR-style data erasure requests), with an audit entry and mandatory approval.

---

## 12. Async Export — Behavior on Server Restart Mid-Export

**Question:** If the server restarts while an async export job (up to 250K rows) is in progress, does the job resume from where it left off, restart from the beginning, or fail permanently?

**My Understanding:** True resume capability requires checkpointing partial output, which adds significant complexity. Given the single-node Docker deployment with no distributed queue, a simpler strategy is more appropriate.

**Solution:** Export jobs will be tracked in an `export_jobs` table with status values: `queued`, `processing`, `completed`, `failed`. On server startup, the application will scan for any jobs in `processing` status and transition them to `queued` for a full restart (not resume). Partially written export files will be deleted. The re-queued job will be picked up by the background worker and executed from the beginning. If a job fails three consecutive times (tracked via a `retry_count` column), it will be marked as `failed` and the requesting user will receive an in-app notification with the failure reason. The client can poll the job status endpoint for progress (reported as percentage of rows processed) or request a manual retry. This avoids the complexity of chunk-level checkpointing while still providing resilience.

---

## 13. Notification Retries — Count, Interval, and Exhaustion Behavior

**Question:** How many times should a failed notification delivery be retried? What is the interval between retries? What happens after all retries are exhausted — is the notification silently dropped, marked as failed, or escalated?

**My Understanding:** Since notifications are in-app only (no external network), delivery failures are likely caused by transient issues such as database write contention or application errors, not network unreliability. A modest retry count with exponential backoff should suffice.

**Solution:** Failed notification deliveries will be retried up to 3 times with exponential backoff: first retry after 30 seconds, second after 2 minutes, third after 10 minutes. Each attempt is logged in the `notification_delivery_log` table with the attempt number, timestamp, and failure reason. After all retries are exhausted, the notification will be marked as `failed` in the delivery log (never silently dropped). A daily summary of failed notifications will be made available via the reporting/metrics endpoints so administrators can identify systemic issues. Failed notifications will remain in the database and can be manually re-triggered by an administrator through a dedicated API endpoint. No external escalation is performed since the system is fully offline.

---

## 14. Audit Before-After Hashes — Algorithm and Scope

**Question:** What hashing algorithm is used for the before-after hashes in the audit trail? Are hashes computed over the full serialized row, or only over the fields that changed?

**My Understanding:** The audit trail's purpose is tamper detection and change verification. Hashing only changed fields would miss tampering of "unchanged" columns. A full-row hash is more robust for integrity verification. SHA-256 is the natural choice given it is already used for file attachment fingerprints.

**Solution:** We will use SHA-256 for audit before-after hashes, consistent with the file attachment fingerprinting. Hashes will be computed over the full serialized row (all columns), not just changed fields. The serialization format will be a canonicalized JSON representation of the row (sorted keys, no whitespace, null values explicitly included) to ensure deterministic hash output. The `before_hash` captures the row state before the write, and `after_hash` captures it after. For insert operations, `before_hash` will be null. For delete operations, `after_hash` will be null. A verification utility endpoint will allow administrators to recompute and compare hashes against stored values to detect any post-hoc tampering with audited tables.

---

## 15. Health and Metrics Endpoints — Authentication and Exposed Data

**Question:** Are the `/health` and `/metrics` endpoints authenticated or publicly accessible within the network? What specific metrics are exposed (e.g., request counts, latency percentiles, database connection pool stats, disk usage)?

**My Understanding:** Since the system is fully offline and deployed on a private network, the risk of exposing health endpoints is lower than in a public-facing service. However, metrics can reveal operational details that could aid an insider threat. A tiered approach balances operational convenience with security.

**Solution:** The `/health` endpoint will be unauthenticated and return only a minimal response: `{ "status": "ok" }` with a `200` status (or `503` if the database is unreachable). This allows Docker health checks and basic monitoring without credentials. The `/metrics` endpoint will require authentication (valid session token) and a dedicated `system:view_metrics` permission. It will expose: request count by endpoint, p50/p95/p99 latency percentiles, active database connection pool size (in-use vs. available), background job queue depth (pending/processing/failed), disk usage for the file attachment storage directory, and uptime in seconds. Metrics will be served as JSON (not Prometheus format, since there is no external monitoring stack). No personally identifiable information or business data will be included in metrics output. Both endpoints will be documented in the API specification.
