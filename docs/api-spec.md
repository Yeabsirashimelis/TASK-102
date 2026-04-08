# RetailOps Data & Transaction Governance API Specification

**Version:** 1.0.0
**Stack:** Rust (Actix-web) + Diesel ORM + PostgreSQL
**Deployment:** Single-node Docker, fully offline
**Base URL:** `/api/v1`

---

## Table of Contents

1. [General Conventions](#1-general-conventions)
2. [Authentication](#2-authentication)
3. [Users](#3-users)
4. [Roles & Permissions](#4-roles--permissions)
5. [Data Scope & Delegation](#5-data-scope--delegation)
6. [Approval Workflow](#6-approval-workflow)
7. [POS Orders](#7-pos-orders)
8. [Returns, Exchanges & Reversals](#8-returns-exchanges--reversals)
9. [Register Reconciliation](#9-register-reconciliation)
10. [Participants & Teams](#10-participants--teams)
11. [Datasets & Versions](#11-datasets--versions)
12. [Notifications](#12-notifications)
13. [Reporting & Analytics](#13-reporting--analytics)
14. [Files](#14-files)
15. [Audit Log](#15-audit-log)
16. [Health & Metrics](#16-health--metrics)
17. [Appendix A: Permission Points](#appendix-a-permission-points)
18. [Appendix B: Order State Transitions](#appendix-b-order-state-transitions)
19. [Appendix C: Idempotency Key Rules](#appendix-c-idempotency-key-rules)
20. [Appendix D: Rate Limit Headers](#appendix-d-rate-limit-headers)

---

## 1. General Conventions

### 1.1 Base URL

All endpoints are prefixed with `/api/v1`. Example: `POST /api/v1/auth/login`.

### 1.2 Authentication

Every request (except `POST /api/v1/auth/login` and `GET /api/v1/health`) must include a bearer token in the `Authorization` header:

```
Authorization: Bearer <session_token>
```

Tokens are local session tokens issued by the login endpoint. There is no cookie-based authentication as there is no browser frontend.

### 1.3 Rate Limiting

Rate limits are configurable per-endpoint via the `rate_limits` table. When a limit is exceeded the server responds with `429 Too Many Requests`. See [Appendix D](#appendix-d-rate-limit-headers) for response headers.

### 1.4 Idempotency

Endpoints that perform stock or accounting writes require an `X-Idempotency-Key` header. The key is a client-generated UUID. If the server has already processed a request with the same key it returns the original response. See [Appendix C](#appendix-c-idempotency-key-rules) for full rules.

### 1.5 Standard Error Response

All errors follow a single envelope:

```json
{
  "error": {
    "code": "VALIDATION_FAILED",
    "message": "Human-readable description of the problem.",
    "details": {
      "field": "username",
      "reason": "must be at least 3 characters"
    }
  }
}
```

| Field | Type | Description |
|---|---|---|
| `error.code` | string | Machine-readable error code (e.g. `VALIDATION_FAILED`, `NOT_FOUND`, `UNAUTHORIZED`). |
| `error.message` | string | Human-readable message. |
| `error.details` | object | Optional additional context. Structure varies by error type. |

### 1.6 Pagination

All list endpoints use cursor-based pagination.

| Parameter | Type | Default | Description |
|---|---|---|---|
| `cursor` | string | _(none)_ | Opaque cursor returned by the previous page. Omit for the first page. |
| `limit` | integer | 25 | Number of items per page. Maximum 100. |

Response envelope for paginated endpoints:

```json
{
  "data": [],
  "pagination": {
    "next_cursor": "eyJpZCI6NDJ9",
    "has_more": true
  }
}
```

### 1.7 Soft Delete

All `DELETE` endpoints perform a soft delete by setting the `deleted_at` timestamp on the record. Soft-deleted records are excluded from list queries by default. Administrators may include `?include_deleted=true` to retrieve them.

### 1.8 Field Masking

Sensitive fields (e.g. `ssn`, `tax_id`, `card_number`) are masked for non-admin roles. Only the last 4 characters are visible:

```json
{
  "tax_id": "****5678"
}
```

Admin users with the `sensitive_data.view` permission see the full value.

### 1.9 Approval Workflow

Certain critical actions do not execute immediately. Instead the server returns `202 Accepted` with an `approval_request_id`. The action is held in a pending state until a manager approves or rejects it. The caller must poll or subscribe to notifications for the outcome.

```json
{
  "status": "pending_approval",
  "approval_request_id": "apr_8f3a1b2c"
}
```

### 1.10 Timestamps

All timestamps are in ISO 8601 format with UTC timezone: `2026-04-07T14:30:00Z`.

### 1.11 Common HTTP Status Codes

| Code | Meaning |
|---|---|
| 200 | Success |
| 201 | Created |
| 202 | Accepted (pending approval) |
| 204 | No Content (successful delete) |
| 400 | Bad Request |
| 401 | Unauthorized (missing or invalid token) |
| 403 | Forbidden (insufficient permissions) |
| 404 | Not Found |
| 409 | Conflict (duplicate idempotency key with different payload, or state conflict) |
| 422 | Unprocessable Entity (validation failure) |
| 429 | Too Many Requests |
| 500 | Internal Server Error |

---

## 2. Authentication

### 2.1 POST /api/v1/auth/login

Authenticate with username and password. Returns a session token.

**Required Role:** None (public).

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Body | `username` | string | Yes | Account username. |
| Body | `password` | string | Yes | Account password. |

```json
{
  "username": "jdoe",
  "password": "S3cure!Pass99"
}
```

**Response (200):**

```json
{
  "token": "sess_a1b2c3d4e5f6...",
  "expires_at": "2026-04-07T22:30:00Z",
  "user": {
    "id": "usr_001",
    "username": "jdoe",
    "display_name": "Jane Doe",
    "roles": ["cashier", "manager"]
  }
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing username or password. |
| 401 | Invalid credentials. |
| 403 | Account locked. |
| 429 | Too many login attempts. |

**Notes:**
- Failed login attempts are recorded in the audit log.
- After 5 consecutive failures the account is locked for 15 minutes.
- Rate limit: 10 requests per minute per IP.

---

### 2.2 POST /api/v1/auth/logout

Revoke the current session token.

**Required Role:** Any authenticated user.

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |

**Response (204):** No content.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Token missing or already revoked. |

**Notes:**
- The token is immediately invalidated and cannot be reused.

---

### 2.3 POST /api/v1/auth/change-password

Change the password for the authenticated user.

**Required Role:** Any authenticated user.

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Body | `current_password` | string | Yes | Current password. |
| Body | `new_password` | string | Yes | New password. Must be 12+ characters with at least 1 uppercase letter, 1 lowercase letter, and 1 digit. |

```json
{
  "current_password": "S3cure!Pass99",
  "new_password": "N3wSecur3Pass!"
}
```

**Response (200):**

```json
{
  "message": "Password changed successfully."
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing fields. |
| 401 | Invalid current password or invalid session. |
| 422 | New password does not meet complexity requirements. |

**Notes:**
- Password complexity: minimum 12 characters, at least 1 uppercase letter, 1 lowercase letter, and 1 digit.
- All other active sessions for the user are revoked on password change.

---

### 2.4 GET /api/v1/auth/session

Return information about the current session.

**Required Role:** Any authenticated user.

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |

**Response (200):**

```json
{
  "session_id": "sess_a1b2c3d4e5f6",
  "user_id": "usr_001",
  "username": "jdoe",
  "display_name": "Jane Doe",
  "roles": ["cashier", "manager"],
  "permissions": ["orders.create", "orders.read", "registers.close"],
  "active_delegations": [
    {
      "delegator_id": "usr_010",
      "scope": "location:store_42",
      "end_at": "2026-04-08T00:00:00Z"
    }
  ],
  "created_at": "2026-04-07T14:30:00Z",
  "expires_at": "2026-04-07T22:30:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Token missing, expired, or revoked. |

---

## 3. Users

### 3.1 POST /api/v1/users

Create a new user account.

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Body | `username` | string | Yes | Unique username. 3-64 characters. |
| Body | `display_name` | string | Yes | Full display name. |
| Body | `password` | string | Yes | Initial password (must meet complexity requirements). |
| Body | `role_ids` | string[] | Yes | Array of role IDs to assign. |
| Body | `email` | string | No | Email address. |
| Body | `location_ids` | string[] | No | Locations the user belongs to. |

```json
{
  "username": "asmith",
  "display_name": "Alice Smith",
  "password": "Initi4lP@ssw0rd",
  "role_ids": ["role_cashier"],
  "email": "asmith@internal.local",
  "location_ids": ["loc_01"]
}
```

**Response (201):**

```json
{
  "id": "usr_042",
  "username": "asmith",
  "display_name": "Alice Smith",
  "email": "asmith@internal.local",
  "roles": [
    { "id": "role_cashier", "name": "Cashier" }
  ],
  "location_ids": ["loc_01"],
  "locked": false,
  "created_at": "2026-04-07T15:00:00Z",
  "updated_at": "2026-04-07T15:00:00Z",
  "deleted_at": null
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields. |
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 409 | Username already exists. |
| 422 | Password does not meet complexity requirements, or invalid role IDs. |

---

### 3.2 GET /api/v1/users

List all user accounts.

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Query | `cursor` | string | No | Pagination cursor. |
| Query | `limit` | integer | No | Items per page (default 25, max 100). |
| Query | `include_deleted` | boolean | No | Include soft-deleted users (default false). |
| Query | `locked` | boolean | No | Filter by locked status. |
| Query | `role_id` | string | No | Filter by role assignment. |

**Response (200):**

```json
{
  "data": [
    {
      "id": "usr_001",
      "username": "jdoe",
      "display_name": "Jane Doe",
      "email": "jdoe@internal.local",
      "roles": [
        { "id": "role_manager", "name": "Manager" }
      ],
      "location_ids": ["loc_01", "loc_02"],
      "locked": false,
      "created_at": "2026-01-15T09:00:00Z",
      "updated_at": "2026-03-20T11:00:00Z",
      "deleted_at": null
    }
  ],
  "pagination": {
    "next_cursor": "eyJpZCI6NDJ9",
    "has_more": true
  }
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |

---

### 3.3 GET /api/v1/users/:id

Get a single user by ID.

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | User ID. |

**Response (200):**

```json
{
  "id": "usr_001",
  "username": "jdoe",
  "display_name": "Jane Doe",
  "email": "jdoe@internal.local",
  "roles": [
    { "id": "role_manager", "name": "Manager" }
  ],
  "location_ids": ["loc_01", "loc_02"],
  "locked": false,
  "last_login_at": "2026-04-07T08:12:00Z",
  "created_at": "2026-01-15T09:00:00Z",
  "updated_at": "2026-03-20T11:00:00Z",
  "deleted_at": null
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 404 | User not found. |

---

### 3.4 PUT /api/v1/users/:id

Update a user account.

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | User ID. |
| Body | `display_name` | string | No | Updated display name. |
| Body | `email` | string | No | Updated email. |
| Body | `role_ids` | string[] | No | Updated role assignments (replaces all). |
| Body | `location_ids` | string[] | No | Updated location assignments. |

```json
{
  "display_name": "Jane M. Doe",
  "role_ids": ["role_manager", "role_auditor"]
}
```

**Response (200):**

Returns the full updated user object (same shape as GET /api/v1/users/:id).

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Invalid request body. |
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 404 | User not found. |
| 422 | Invalid role IDs or other validation failure. |

---

### 3.5 DELETE /api/v1/users/:id

Soft-delete a user account.

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | User ID. |

**Response (204):** No content.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 404 | User not found. |

**Notes:**
- Soft-deletes the user (sets `deleted_at`).
- All active sessions for the user are revoked.

---

### 3.6 POST /api/v1/users/:id/lock

Lock a user account, preventing login.

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | User ID. |
| Body | `reason` | string | No | Reason for locking. |

```json
{
  "reason": "Suspected credential compromise."
}
```

**Response (200):**

```json
{
  "id": "usr_042",
  "locked": true,
  "locked_reason": "Suspected credential compromise.",
  "locked_at": "2026-04-07T15:10:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 404 | User not found. |
| 409 | User is already locked. |

**Notes:**
- All active sessions for the locked user are immediately revoked.

---

### 3.7 POST /api/v1/users/:id/unlock

Unlock a previously locked user account.

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | User ID. |

**Response (200):**

```json
{
  "id": "usr_042",
  "locked": false,
  "locked_reason": null,
  "locked_at": null
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 404 | User not found. |
| 409 | User is not locked. |

---

## 4. Roles & Permissions

### 4.1 POST /api/v1/roles

Create a new role.

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Body | `name` | string | Yes | Role display name (e.g. "Store Manager"). |
| Body | `slug` | string | Yes | Unique machine-readable slug (e.g. `store_manager`). |
| Body | `description` | string | No | Description of the role. |

```json
{
  "name": "Store Manager",
  "slug": "store_manager",
  "description": "Full store-level access with approval authority."
}
```

**Response (201):**

```json
{
  "id": "role_store_manager",
  "name": "Store Manager",
  "slug": "store_manager",
  "description": "Full store-level access with approval authority.",
  "created_at": "2026-04-07T15:00:00Z",
  "updated_at": "2026-04-07T15:00:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields. |
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 409 | Slug already exists. |
| 422 | Validation failure. |

---

### 4.2 GET /api/v1/roles

List all roles.

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Query | `cursor` | string | No | Pagination cursor. |
| Query | `limit` | integer | No | Items per page (default 25, max 100). |

**Response (200):**

```json
{
  "data": [
    {
      "id": "role_admin",
      "name": "Admin",
      "slug": "admin",
      "description": "Full system access.",
      "created_at": "2026-01-01T00:00:00Z",
      "updated_at": "2026-01-01T00:00:00Z"
    }
  ],
  "pagination": {
    "next_cursor": null,
    "has_more": false
  }
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |

---

### 4.3 GET /api/v1/roles/:id

Get a single role.

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Role ID. |

**Response (200):**

```json
{
  "id": "role_store_manager",
  "name": "Store Manager",
  "slug": "store_manager",
  "description": "Full store-level access with approval authority.",
  "created_at": "2026-04-07T15:00:00Z",
  "updated_at": "2026-04-07T15:00:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 404 | Role not found. |

---

### 4.4 PUT /api/v1/roles/:id

Update a role.

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Role ID. |
| Body | `name` | string | No | Updated display name. |
| Body | `description` | string | No | Updated description. |

**Response (200):** Returns the full updated role object.

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Invalid request body. |
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 404 | Role not found. |
| 422 | Validation failure. |

---

### 4.5 DELETE /api/v1/roles/:id

Soft-delete a role. Fails if users are still assigned to this role.

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Role ID. |

**Response (204):** No content.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 404 | Role not found. |
| 409 | Role is still assigned to one or more users. |

---

### 4.6 GET /api/v1/roles/:id/permissions

List all permission bindings for a role.

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Role ID. |

**Response (200):**

```json
{
  "role_id": "role_store_manager",
  "permissions": [
    { "slug": "orders.create", "granted": true },
    { "slug": "orders.read", "granted": true },
    { "slug": "orders.return", "granted": true },
    { "slug": "registers.close", "granted": true },
    { "slug": "registers.confirm", "granted": true },
    { "slug": "users.manage", "granted": false }
  ]
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 404 | Role not found. |

---

### 4.7 PUT /api/v1/roles/:id/permissions

Replace all permission bindings for a role.

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Role ID. |
| Body | `permissions` | object[] | Yes | Array of `{ "slug": "string", "granted": bool }`. |

```json
{
  "permissions": [
    { "slug": "orders.create", "granted": true },
    { "slug": "orders.read", "granted": true },
    { "slug": "registers.close", "granted": true }
  ]
}
```

**Response (200):** Returns the updated permission bindings (same shape as GET).

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing or empty permissions array. |
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 404 | Role not found. |
| 422 | Unknown permission slug. |

---

### 4.8 GET /api/v1/permission-points

List all available permission points in the system.

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |

**Response (200):**

```json
{
  "data": [
    {
      "slug": "orders.create",
      "description": "Create new POS orders.",
      "category": "orders"
    },
    {
      "slug": "orders.read",
      "description": "View POS orders.",
      "category": "orders"
    }
  ]
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |

---

### 4.9 CRUD /api/v1/api-scopes

Manage API scope definitions that control which API endpoints a role can access.

**Required Role:** `admin`

#### POST /api/v1/api-scopes

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Body | `name` | string | Yes | Scope name. |
| Body | `slug` | string | Yes | Unique slug. |
| Body | `endpoints` | string[] | Yes | List of endpoint patterns (e.g. `GET /api/v1/orders/*`). |

```json
{
  "name": "Orders Read-Only",
  "slug": "orders_readonly",
  "endpoints": [
    "GET /api/v1/orders",
    "GET /api/v1/orders/*"
  ]
}
```

**Response (201):**

```json
{
  "id": "scope_orders_readonly",
  "name": "Orders Read-Only",
  "slug": "orders_readonly",
  "endpoints": [
    "GET /api/v1/orders",
    "GET /api/v1/orders/*"
  ],
  "created_at": "2026-04-07T15:00:00Z",
  "updated_at": "2026-04-07T15:00:00Z"
}
```

#### GET /api/v1/api-scopes

List all API scopes. Supports pagination (`cursor`, `limit`).

#### GET /api/v1/api-scopes/:id

Get a single API scope by ID.

#### PUT /api/v1/api-scopes/:id

Update an API scope. Accepts the same body fields as POST.

#### DELETE /api/v1/api-scopes/:id

Soft-delete an API scope.

**Error Codes (all operations):**

| Code | Condition |
|---|---|
| 400 | Invalid request body. |
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 404 | Scope not found (GET/PUT/DELETE by ID). |
| 409 | Slug already exists (POST). |
| 422 | Validation failure. |

---

### 4.10 CRUD /api/v1/menu-scopes

Manage menu scope definitions that control which logical menu sections a role can access. These are metadata entries used by API consumers to determine which features are available to a role; they do not affect endpoint authorization directly.

**Required Role:** `admin`

#### POST /api/v1/menu-scopes

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Body | `name` | string | Yes | Menu scope name. |
| Body | `slug` | string | Yes | Unique slug. |
| Body | `menu_keys` | string[] | Yes | List of menu section keys (e.g. `orders`, `reports`). |

```json
{
  "name": "Cashier Menu",
  "slug": "cashier_menu",
  "menu_keys": ["orders", "registers"]
}
```

**Response (201):**

```json
{
  "id": "mscope_cashier_menu",
  "name": "Cashier Menu",
  "slug": "cashier_menu",
  "menu_keys": ["orders", "registers"],
  "created_at": "2026-04-07T15:00:00Z",
  "updated_at": "2026-04-07T15:00:00Z"
}
```

#### GET /api/v1/menu-scopes

List all menu scopes. Supports pagination (`cursor`, `limit`).

#### GET /api/v1/menu-scopes/:id

Get a single menu scope by ID.

#### PUT /api/v1/menu-scopes/:id

Update a menu scope.

#### DELETE /api/v1/menu-scopes/:id

Soft-delete a menu scope.

**Error Codes (all operations):**

| Code | Condition |
|---|---|
| 400 | Invalid request body. |
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 404 | Scope not found. |
| 409 | Slug already exists. |
| 422 | Validation failure. |

---

## 5. Data Scope & Delegation

### 5.1 POST /api/v1/data-scopes

Create a data scope that restricts a user's or role's data visibility (e.g. by location, department, or custom dimension).

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Body | `name` | string | Yes | Scope name. |
| Body | `slug` | string | Yes | Unique slug. |
| Body | `dimension` | string | Yes | Dimension key (e.g. `location`, `department`). |
| Body | `allowed_values` | string[] | Yes | Allowed values for the dimension. |

```json
{
  "name": "West Region Stores",
  "slug": "west_region",
  "dimension": "location",
  "allowed_values": ["loc_01", "loc_02", "loc_03"]
}
```

**Response (201):**

```json
{
  "id": "ds_west_region",
  "name": "West Region Stores",
  "slug": "west_region",
  "dimension": "location",
  "allowed_values": ["loc_01", "loc_02", "loc_03"],
  "created_at": "2026-04-07T15:00:00Z",
  "updated_at": "2026-04-07T15:00:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields. |
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 409 | Slug already exists. |
| 422 | Invalid dimension or empty allowed_values. |

---

### 5.2 GET /api/v1/data-scopes

List all data scopes. Supports pagination.

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Query | `cursor` | string | No | Pagination cursor. |
| Query | `limit` | integer | No | Items per page (default 25, max 100). |

**Response (200):** Paginated list of data scope objects.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |

---

### 5.3 GET /api/v1/data-scopes/:id

Get a single data scope.

**Required Role:** `admin`

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 404 | Data scope not found. |

---

### 5.4 PUT /api/v1/data-scopes/:id

Update a data scope.

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Data scope ID. |
| Body | `name` | string | No | Updated name. |
| Body | `allowed_values` | string[] | No | Updated allowed values. |

**Response (200):** Returns the full updated data scope object.

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Invalid request body. |
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 404 | Data scope not found. |
| 422 | Validation failure. |

---

### 5.5 DELETE /api/v1/data-scopes/:id

Soft-delete a data scope.

**Required Role:** `admin`

**Response (204):** No content.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 404 | Data scope not found. |
| 409 | Scope is referenced by active delegations. |

---

### 5.6 POST /api/v1/delegations

Create a time-bounded delegation, granting another user temporary access to a data scope.

**Required Role:** `manager` or `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Body | `delegate_user_id` | string | Yes | User receiving the delegation. |
| Body | `data_scope_id` | string | Yes | Data scope being delegated. |
| Body | `start_at` | string | Yes | ISO 8601 start time. |
| Body | `end_at` | string | Yes | ISO 8601 end time. Must be after `start_at`. |
| Body | `reason` | string | No | Reason for delegation. |

```json
{
  "delegate_user_id": "usr_042",
  "data_scope_id": "ds_west_region",
  "start_at": "2026-04-07T16:00:00Z",
  "end_at": "2026-04-08T00:00:00Z",
  "reason": "Covering for manager absence."
}
```

**Response (201):**

```json
{
  "id": "del_001",
  "delegator_user_id": "usr_001",
  "delegate_user_id": "usr_042",
  "data_scope_id": "ds_west_region",
  "start_at": "2026-04-07T16:00:00Z",
  "end_at": "2026-04-08T00:00:00Z",
  "reason": "Covering for manager absence.",
  "revoked": false,
  "created_at": "2026-04-07T15:30:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields or `end_at` before `start_at`. |
| 401 | Unauthenticated. |
| 403 | Caller lacks delegation authority. |
| 404 | User or data scope not found. |
| 422 | Validation failure (e.g. delegating to self, overlapping delegation). |

---

### 5.7 GET /api/v1/delegations

List delegations created by or granted to the current user.

**Required Role:** Any authenticated user (sees own delegations). `admin` sees all.

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Query | `cursor` | string | No | Pagination cursor. |
| Query | `limit` | integer | No | Items per page. |
| Query | `direction` | string | No | `granted` (to me) or `created` (by me). Default: both. |

**Response (200):** Paginated list of delegation objects.

---

### 5.8 DELETE /api/v1/delegations/:id

Revoke a delegation early.

**Required Role:** The delegator who created it, or `admin`.

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Delegation ID. |

**Response (204):** No content.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is neither the delegator nor an admin. |
| 404 | Delegation not found. |
| 409 | Already revoked. |

---

### 5.9 GET /api/v1/delegations/active

List currently active delegations (where `now()` is between `start_at` and `end_at` and `revoked` is false).

**Required Role:** Any authenticated user (sees own). `admin` sees all.

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |

**Response (200):**

```json
{
  "data": [
    {
      "id": "del_001",
      "delegator_user_id": "usr_001",
      "delegate_user_id": "usr_042",
      "data_scope_id": "ds_west_region",
      "start_at": "2026-04-07T16:00:00Z",
      "end_at": "2026-04-08T00:00:00Z",
      "reason": "Covering for manager absence."
    }
  ]
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |

---

## 6. Approval Workflow

### 6.1 GET /api/v1/approval-configs

List which actions require approval and their configuration.

**Required Role:** `admin` or `manager`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |

**Response (200):**

```json
{
  "data": [
    {
      "action": "order.reversal_over_24h",
      "requires_approval": true,
      "approver_roles": ["manager", "admin"],
      "auto_reject_after_hours": 48,
      "description": "Reversal of an order older than 24 hours."
    },
    {
      "action": "dataset.version_rollback",
      "requires_approval": true,
      "approver_roles": ["admin"],
      "auto_reject_after_hours": 72,
      "description": "Rollback a dataset version."
    },
    {
      "action": "audit_log.bulk_export",
      "requires_approval": true,
      "approver_roles": ["admin"],
      "auto_reject_after_hours": 24,
      "description": "Bulk export of audit log records."
    }
  ]
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |

---

### 6.2 PUT /api/v1/approval-configs

Update approval configuration.

**Required Role:** `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Body | `configs` | object[] | Yes | Array of config objects. |

```json
{
  "configs": [
    {
      "action": "order.reversal_over_24h",
      "requires_approval": true,
      "approver_roles": ["manager", "admin"],
      "auto_reject_after_hours": 48
    }
  ]
}
```

**Response (200):** Returns the full updated configuration list.

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Invalid request body. |
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 422 | Unknown action slug or invalid approver roles. |

---

### 6.3 GET /api/v1/approval-requests

List pending approval requests assigned to the current user's roles.

**Required Role:** `manager` or `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Query | `cursor` | string | No | Pagination cursor. |
| Query | `limit` | integer | No | Items per page. |
| Query | `status` | string | No | Filter: `pending`, `approved`, `rejected`. Default: `pending`. |
| Query | `action` | string | No | Filter by action type. |

**Response (200):**

```json
{
  "data": [
    {
      "id": "apr_8f3a1b2c",
      "action": "order.reversal_over_24h",
      "status": "pending",
      "requester_user_id": "usr_042",
      "resource_type": "order",
      "resource_id": "ord_999",
      "payload": {
        "reason": "Customer dispute on order from 3 days ago."
      },
      "created_at": "2026-04-07T15:00:00Z",
      "auto_rejects_at": "2026-04-09T15:00:00Z"
    }
  ],
  "pagination": {
    "next_cursor": null,
    "has_more": false
  }
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is not a manager or admin. |

---

### 6.4 GET /api/v1/approval-requests/:id

Get a single approval request with full details.

**Required Role:** `manager`, `admin`, or the original requester.

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Approval request ID. |

**Response (200):**

```json
{
  "id": "apr_8f3a1b2c",
  "action": "order.reversal_over_24h",
  "status": "pending",
  "requester_user_id": "usr_042",
  "resource_type": "order",
  "resource_id": "ord_999",
  "payload": {
    "reason": "Customer dispute on order from 3 days ago."
  },
  "decision": null,
  "decided_by": null,
  "decided_at": null,
  "rejection_reason": null,
  "created_at": "2026-04-07T15:00:00Z",
  "auto_rejects_at": "2026-04-09T15:00:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Approval request not found. |

---

### 6.5 POST /api/v1/approval-requests/:id/approve

Approve a pending request. The approver cannot be the same user who created the request (no self-approval).

**Required Role:** `manager` or `admin` (must be in the `approver_roles` list for the action).

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Approval request ID. |
| Body | `comment` | string | No | Optional approval comment. |

```json
{
  "comment": "Verified with customer service records."
}
```

**Response (200):**

```json
{
  "id": "apr_8f3a1b2c",
  "status": "approved",
  "decided_by": "usr_001",
  "decided_at": "2026-04-07T16:00:00Z",
  "comment": "Verified with customer service records."
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is the requester (self-approval forbidden) or lacks required role. |
| 404 | Approval request not found. |
| 409 | Request is not in `pending` status. |

**Notes:**
- On approval, the originally-requested action is executed automatically.
- An audit log entry is created recording the approval.

---

### 6.6 POST /api/v1/approval-requests/:id/reject

Reject a pending approval request with a reason.

**Required Role:** `manager` or `admin`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Approval request ID. |
| Body | `reason` | string | Yes | Reason for rejection. |

```json
{
  "reason": "Insufficient documentation provided."
}
```

**Response (200):**

```json
{
  "id": "apr_8f3a1b2c",
  "status": "rejected",
  "decided_by": "usr_001",
  "decided_at": "2026-04-07T16:00:00Z",
  "rejection_reason": "Insufficient documentation provided."
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing reason. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Approval request not found. |
| 409 | Request is not in `pending` status. |

---

## 7. POS Orders

### 7.1 POST /api/v1/orders

Create a new POS order.

**Required Role:** `cashier`, `manager`, or `admin`
**Required Permission:** `orders.create`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Header | `X-Idempotency-Key` | string | Yes | Client-generated UUID. |
| Body | `location_id` | string | Yes | Store location ID. |
| Body | `cashier_user_id` | string | No | Defaults to the authenticated user. |
| Body | `customer_ref` | string | No | Optional customer reference. |
| Body | `lines` | object[] | No | Initial order lines (see add-line body below). |
| Body | `notes` | string | No | Order-level notes. |

```json
{
  "location_id": "loc_01",
  "customer_ref": "CUST-5678",
  "lines": [
    {
      "sku": "SKU-001",
      "description": "Widget A",
      "quantity": 2,
      "unit_price_cents": 1999,
      "discount_cents": 0
    }
  ],
  "notes": "Customer requested gift wrapping."
}
```

**Response (201):**

```json
{
  "id": "ord_1001",
  "status": "open",
  "location_id": "loc_01",
  "cashier_user_id": "usr_042",
  "customer_ref": "CUST-5678",
  "lines": [
    {
      "id": "line_001",
      "sku": "SKU-001",
      "description": "Widget A",
      "quantity": 2,
      "unit_price_cents": 1999,
      "discount_cents": 0,
      "line_total_cents": 3998
    }
  ],
  "tenders": [],
  "subtotal_cents": 3998,
  "tax_cents": 320,
  "total_cents": 4318,
  "notes": "Customer requested gift wrapping.",
  "created_at": "2026-04-07T16:00:00Z",
  "updated_at": "2026-04-07T16:00:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 409 | Idempotency key already used with different payload. |
| 422 | Validation failure (invalid SKU, negative quantity, etc.). |
| 429 | Rate limit exceeded. |

**Notes:**
- Requires `X-Idempotency-Key` header. See [Appendix C](#appendix-c-idempotency-key-rules).
- Order is created in `open` state.

---

### 7.2 GET /api/v1/orders

List and search orders.

**Required Role:** `cashier` (own orders only), `manager`, or `admin`
**Required Permission:** `orders.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Query | `cursor` | string | No | Pagination cursor. |
| Query | `limit` | integer | No | Items per page (default 25, max 100). |
| Query | `status` | string | No | Filter by status: `open`, `completed`, `voided`, `returned`. |
| Query | `location_id` | string | No | Filter by location. |
| Query | `cashier_user_id` | string | No | Filter by cashier. |
| Query | `date_from` | string | No | ISO 8601 start date (inclusive). |
| Query | `date_to` | string | No | ISO 8601 end date (inclusive). |
| Query | `customer_ref` | string | No | Filter by customer reference. |

**Response (200):** Paginated list of order summary objects (without nested lines/tenders for performance).

```json
{
  "data": [
    {
      "id": "ord_1001",
      "status": "completed",
      "location_id": "loc_01",
      "cashier_user_id": "usr_042",
      "customer_ref": "CUST-5678",
      "subtotal_cents": 3998,
      "tax_cents": 320,
      "total_cents": 4318,
      "line_count": 1,
      "created_at": "2026-04-07T16:00:00Z",
      "updated_at": "2026-04-07T16:30:00Z"
    }
  ],
  "pagination": {
    "next_cursor": "eyJpZCI6MTAwMX0",
    "has_more": true
  }
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 422 | Invalid filter values. |

---

### 7.3 GET /api/v1/orders/:id

Get full order detail including lines and tenders.

**Required Role:** `cashier` (own orders only), `manager`, or `admin`
**Required Permission:** `orders.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Order ID. |

**Response (200):**

```json
{
  "id": "ord_1001",
  "status": "completed",
  "location_id": "loc_01",
  "cashier_user_id": "usr_042",
  "customer_ref": "CUST-5678",
  "lines": [
    {
      "id": "line_001",
      "sku": "SKU-001",
      "description": "Widget A",
      "quantity": 2,
      "unit_price_cents": 1999,
      "discount_cents": 0,
      "line_total_cents": 3998
    }
  ],
  "tenders": [
    {
      "id": "tndr_001",
      "type": "card",
      "amount_cents": 4318,
      "reference": "****1234",
      "tendered_at": "2026-04-07T16:25:00Z"
    }
  ],
  "receipts": [
    {
      "id": "rcpt_001",
      "file_id": "file_abc",
      "created_at": "2026-04-07T16:26:00Z"
    }
  ],
  "subtotal_cents": 3998,
  "tax_cents": 320,
  "total_cents": 4318,
  "notes": "Customer requested gift wrapping.",
  "created_at": "2026-04-07T16:00:00Z",
  "updated_at": "2026-04-07T16:30:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions or order belongs to another cashier. |
| 404 | Order not found. |

---

### 7.4 PATCH /api/v1/orders/:id

Update order metadata or transition its state.

**Required Role:** `cashier` (own, open orders), `manager`, or `admin`
**Required Permission:** `orders.update`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Order ID. |
| Body | `status` | string | No | New status. Must follow valid state transitions (see [Appendix B](#appendix-b-order-state-transitions)). |
| Body | `notes` | string | No | Updated notes. |
| Body | `customer_ref` | string | No | Updated customer reference. |

```json
{
  "status": "completed"
}
```

**Response (200):** Returns the full updated order object.

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Invalid request body. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Order not found. |
| 409 | Invalid state transition (e.g. `completed` to `open`). |
| 422 | Validation failure. |

**Notes:**
- See [Appendix B](#appendix-b-order-state-transitions) for valid state transitions and required roles.

---

### 7.5 POST /api/v1/orders/:id/lines

Add an order line to an existing order.

**Required Role:** `cashier` (own, open orders), `manager`, or `admin`
**Required Permission:** `orders.update`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Header | `X-Idempotency-Key` | string | Yes | Client-generated UUID. |
| Path | `id` | string | Yes | Order ID. |
| Body | `sku` | string | Yes | Product SKU. |
| Body | `description` | string | Yes | Product description. |
| Body | `quantity` | integer | Yes | Quantity (must be > 0). |
| Body | `unit_price_cents` | integer | Yes | Unit price in cents. |
| Body | `discount_cents` | integer | No | Discount amount in cents (default 0). |

```json
{
  "sku": "SKU-002",
  "description": "Widget B",
  "quantity": 1,
  "unit_price_cents": 2499,
  "discount_cents": 250
}
```

**Response (201):**

```json
{
  "id": "line_002",
  "order_id": "ord_1001",
  "sku": "SKU-002",
  "description": "Widget B",
  "quantity": 1,
  "unit_price_cents": 2499,
  "discount_cents": 250,
  "line_total_cents": 2249
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Order not found. |
| 409 | Order is not in `open` state, or idempotency collision. |
| 422 | Validation failure. |

---

### 7.6 PATCH /api/v1/orders/:id/lines/:lineId

Update an order line.

**Required Role:** `cashier` (own, open orders), `manager`, or `admin`
**Required Permission:** `orders.update`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Order ID. |
| Path | `lineId` | string | Yes | Line ID. |
| Body | `quantity` | integer | No | Updated quantity. |
| Body | `unit_price_cents` | integer | No | Updated unit price. |
| Body | `discount_cents` | integer | No | Updated discount. |

**Response (200):** Returns the updated line object.

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Invalid request body. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Order or line not found. |
| 409 | Order is not in `open` state. |
| 422 | Validation failure. |

---

### 7.7 DELETE /api/v1/orders/:id/lines/:lineId

Remove an order line (soft delete).

**Required Role:** `cashier` (own, open orders), `manager`, or `admin`
**Required Permission:** `orders.update`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Order ID. |
| Path | `lineId` | string | Yes | Line ID. |

**Response (204):** No content.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Order or line not found. |
| 409 | Order is not in `open` state. |

---

### 7.8 POST /api/v1/orders/:id/tenders

Add a tender (payment) entry to an order.

**Required Role:** `cashier`, `manager`, or `admin`
**Required Permission:** `orders.tender`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Header | `X-Idempotency-Key` | string | Yes | Client-generated UUID. |
| Path | `id` | string | Yes | Order ID. |
| Body | `type` | string | Yes | Tender type: `cash`, `card`, `gift_card`. |
| Body | `amount_cents` | integer | Yes | Amount tendered in cents. |
| Body | `reference` | string | No | External reference (e.g. card last-4 digits, gift card number). |

```json
{
  "type": "card",
  "amount_cents": 4318,
  "reference": "****1234"
}
```

**Response (201):**

```json
{
  "id": "tndr_001",
  "order_id": "ord_1001",
  "type": "card",
  "amount_cents": 4318,
  "reference": "****1234",
  "tendered_at": "2026-04-07T16:25:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Order not found. |
| 409 | Order is not in `open` state, or idempotency collision. |
| 422 | Invalid tender type or amount. |

**Notes:**
- Requires `X-Idempotency-Key` header.
- Card references are automatically masked to last 4 digits on storage.

---

### 7.9 POST /api/v1/orders/:id/receipts

Attach a receipt file to an order.

**Required Role:** `cashier`, `manager`, or `admin`
**Required Permission:** `orders.update`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Order ID. |
| Body | `file_id` | string | Yes | ID of a previously uploaded file (see Files module). |

```json
{
  "file_id": "file_abc"
}
```

**Response (201):**

```json
{
  "id": "rcpt_001",
  "order_id": "ord_1001",
  "file_id": "file_abc",
  "created_at": "2026-04-07T16:26:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing file_id. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Order or file not found. |
| 422 | File type not allowed for receipts. |

---

### 7.10 GET /api/v1/orders/:id/receipts

List receipts attached to an order.

**Required Role:** `cashier` (own orders), `manager`, or `admin`
**Required Permission:** `orders.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Order ID. |

**Response (200):**

```json
{
  "data": [
    {
      "id": "rcpt_001",
      "file_id": "file_abc",
      "filename": "receipt_ord1001.pdf",
      "content_type": "application/pdf",
      "created_at": "2026-04-07T16:26:00Z"
    }
  ]
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Order not found. |

---

## 8. Returns, Exchanges & Reversals

### 8.1 POST /api/v1/orders/:id/return

Initiate a return against a completed order.

**Required Role:** `cashier`, `manager`, or `admin`
**Required Permission:** `orders.return`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Header | `X-Idempotency-Key` | string | Yes | Client-generated UUID. |
| Path | `id` | string | Yes | Original order ID. |
| Body | `lines` | object[] | Yes | Lines being returned with quantities. |
| Body | `reason` | string | Yes | Reason for return. |
| Body | `refund_type` | string | Yes | `original_tender`, `store_credit`, `cash`. |

```json
{
  "lines": [
    { "line_id": "line_001", "quantity": 1, "reason": "Defective item." }
  ],
  "reason": "Customer returning defective item.",
  "refund_type": "original_tender"
}
```

**Response (201):**

```json
{
  "id": "ret_001",
  "order_id": "ord_1001",
  "status": "processed",
  "lines": [
    {
      "line_id": "line_001",
      "quantity": 1,
      "refund_amount_cents": 1999,
      "reason": "Defective item."
    }
  ],
  "total_refund_cents": 1999,
  "refund_type": "original_tender",
  "reason": "Customer returning defective item.",
  "created_by": "usr_042",
  "created_at": "2026-04-07T17:00:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Order or line not found. |
| 409 | Order is not in `completed` state, line already fully returned, or idempotency collision. |
| 422 | Return quantity exceeds original quantity. |

**Notes:**
- Requires `X-Idempotency-Key` header.
- Return creates a corresponding accounting entry for the refund.

---

### 8.2 POST /api/v1/orders/:id/exchange

Initiate an exchange on a completed order.

**Required Role:** `cashier`, `manager`, or `admin`
**Required Permission:** `orders.exchange`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Header | `X-Idempotency-Key` | string | Yes | Client-generated UUID. |
| Path | `id` | string | Yes | Original order ID. |
| Body | `return_lines` | object[] | Yes | Lines being returned. |
| Body | `new_lines` | object[] | Yes | Replacement lines. |
| Body | `reason` | string | Yes | Reason for exchange. |

```json
{
  "return_lines": [
    { "line_id": "line_001", "quantity": 1 }
  ],
  "new_lines": [
    {
      "sku": "SKU-003",
      "description": "Widget C",
      "quantity": 1,
      "unit_price_cents": 2499
    }
  ],
  "reason": "Customer wants different color."
}
```

**Response (201):**

```json
{
  "id": "exch_001",
  "order_id": "ord_1001",
  "status": "processed",
  "return_lines": [
    { "line_id": "line_001", "quantity": 1, "refund_amount_cents": 1999 }
  ],
  "new_order_id": "ord_1002",
  "new_lines": [
    {
      "id": "line_010",
      "sku": "SKU-003",
      "description": "Widget C",
      "quantity": 1,
      "unit_price_cents": 2499,
      "line_total_cents": 2499
    }
  ],
  "price_difference_cents": 500,
  "reason": "Customer wants different color.",
  "created_by": "usr_042",
  "created_at": "2026-04-07T17:05:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Order or line not found. |
| 409 | Order not in `completed` state, or idempotency collision. |
| 422 | Validation failure. |

**Notes:**
- Requires `X-Idempotency-Key` header.
- Creates a new order for the replacement items.
- If price difference is positive, additional payment is required on the new order.

---

### 8.3 POST /api/v1/orders/:id/reversal

Initiate a full reversal (void) of an order.

**Required Role:** `manager` or `admin`
**Required Permission:** `orders.reverse`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Header | `X-Idempotency-Key` | string | Yes | Client-generated UUID. |
| Path | `id` | string | Yes | Order ID to reverse. |
| Body | `reason` | string | Yes | Reason for reversal. |

```json
{
  "reason": "Duplicate order entered by mistake."
}
```

**Response (201) or (202):**

If the order was created within the last 24 hours (immediate processing):

```json
{
  "id": "rev_001",
  "order_id": "ord_1001",
  "status": "processed",
  "refund_total_cents": 4318,
  "reason": "Duplicate order entered by mistake.",
  "created_by": "usr_001",
  "created_at": "2026-04-07T17:10:00Z"
}
```

If the order is older than 24 hours (requires approval, returns 202):

```json
{
  "status": "pending_approval",
  "approval_request_id": "apr_9d4e2f1a",
  "message": "Reversal of orders older than 24 hours requires manager approval."
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing reason. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Order not found. |
| 409 | Order already voided/reversed, or idempotency collision. |
| 422 | Order has active returns that must be resolved first. |

**Notes:**
- Requires `X-Idempotency-Key` header.
- Orders older than 24 hours trigger the approval workflow and return 202 Accepted.
- Reversals also require register reconciliation confirmation if the register for that shift has already been closed.

---

## 9. Register Reconciliation

### 9.1 POST /api/v1/registers/close

Submit a register close record at the end of a shift.

**Required Role:** `cashier`, `manager`, or `admin`
**Required Permission:** `registers.close`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Body | `location_id` | string | Yes | Store location ID. |
| Body | `register_id` | string | Yes | Register identifier. |
| Body | `cashier_user_id` | string | Yes | Cashier closing the register. |
| Body | `shift_start` | string | Yes | ISO 8601 shift start time. |
| Body | `shift_end` | string | Yes | ISO 8601 shift end time. |
| Body | `expected_cents` | integer | Yes | System-calculated expected amount in cents. |
| Body | `actual_cents` | integer | Yes | Physical counted amount in cents. |
| Body | `notes` | string | No | Cashier notes. |

```json
{
  "location_id": "loc_01",
  "register_id": "reg_03",
  "cashier_user_id": "usr_042",
  "shift_start": "2026-04-07T08:00:00Z",
  "shift_end": "2026-04-07T16:00:00Z",
  "expected_cents": 523400,
  "actual_cents": 522100,
  "notes": "Two pennies stuck in drawer."
}
```

**Response (201):**

```json
{
  "id": "close_001",
  "location_id": "loc_01",
  "register_id": "reg_03",
  "cashier_user_id": "usr_042",
  "shift_start": "2026-04-07T08:00:00Z",
  "shift_end": "2026-04-07T16:00:00Z",
  "expected_cents": 523400,
  "actual_cents": 522100,
  "variance_cents": -1300,
  "status": "pending_confirmation",
  "requires_confirmation": true,
  "confirmation_reason": "Variance exceeds $20 threshold.",
  "notes": "Two pennies stuck in drawer.",
  "created_at": "2026-04-07T16:05:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 409 | Register close already submitted for this shift. |
| 422 | Validation failure (shift_end before shift_start, etc.). |

**Notes:**
- If the absolute variance exceeds $20.00 (2000 cents), the close record requires manager confirmation.
- If a reversal occurred after the shift started and was older than 24 hours, manager confirmation is also required.

---

### 9.2 GET /api/v1/registers/closes

List register close records.

**Required Role:** `cashier` (own records), `manager`, or `admin`
**Required Permission:** `registers.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Query | `cursor` | string | No | Pagination cursor. |
| Query | `limit` | integer | No | Items per page (default 25, max 100). |
| Query | `location_id` | string | No | Filter by location. |
| Query | `date_from` | string | No | ISO 8601 start date. |
| Query | `date_to` | string | No | ISO 8601 end date. |
| Query | `status` | string | No | `pending_confirmation`, `confirmed`, `flagged`. |

**Response (200):** Paginated list of close record objects.

```json
{
  "data": [
    {
      "id": "close_001",
      "location_id": "loc_01",
      "register_id": "reg_03",
      "cashier_user_id": "usr_042",
      "shift_start": "2026-04-07T08:00:00Z",
      "shift_end": "2026-04-07T16:00:00Z",
      "expected_cents": 523400,
      "actual_cents": 522100,
      "variance_cents": -1300,
      "status": "pending_confirmation",
      "created_at": "2026-04-07T16:05:00Z"
    }
  ],
  "pagination": {
    "next_cursor": null,
    "has_more": false
  }
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |

---

### 9.3 GET /api/v1/registers/closes/:id

Get a single close record with full details.

**Required Role:** `cashier` (own record), `manager`, or `admin`
**Required Permission:** `registers.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Close record ID. |

**Response (200):** Full close record object (same shape as POST response, plus confirmation details if confirmed).

```json
{
  "id": "close_001",
  "location_id": "loc_01",
  "register_id": "reg_03",
  "cashier_user_id": "usr_042",
  "shift_start": "2026-04-07T08:00:00Z",
  "shift_end": "2026-04-07T16:00:00Z",
  "expected_cents": 523400,
  "actual_cents": 522100,
  "variance_cents": -1300,
  "status": "confirmed",
  "requires_confirmation": true,
  "confirmation_reason": "Variance exceeds $20 threshold.",
  "confirmed_by": "usr_001",
  "confirmed_at": "2026-04-07T16:30:00Z",
  "confirmation_notes": "Verified drawer count. Penny shortage acceptable.",
  "notes": "Two pennies stuck in drawer.",
  "created_at": "2026-04-07T16:05:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Close record not found. |

---

### 9.4 POST /api/v1/registers/closes/:id/confirm

Manager second-confirmation for a close record.

**Required Role:** `manager` or `admin`
**Required Permission:** `registers.confirm`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Close record ID. |
| Body | `notes` | string | No | Confirmation notes from the manager. |
| Body | `action` | string | Yes | `confirm` or `flag`. |

```json
{
  "action": "confirm",
  "notes": "Verified drawer count. Penny shortage acceptable."
}
```

**Response (200):**

```json
{
  "id": "close_001",
  "status": "confirmed",
  "confirmed_by": "usr_001",
  "confirmed_at": "2026-04-07T16:30:00Z",
  "confirmation_notes": "Verified drawer count. Penny shortage acceptable."
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing action field. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions or confirming own close record. |
| 404 | Close record not found. |
| 409 | Close record is not in `pending_confirmation` status. |

**Notes:**
- Required when variance exceeds $20.00 or when a reversal older than 24 hours affected the shift.
- Manager cannot confirm their own close record.

---

## 10. Participants & Teams

### 10.1 POST /api/v1/participants

Create a participant record.

**Required Role:** `manager` or `admin`
**Required Permission:** `participants.create`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Body | `first_name` | string | Yes | First name. |
| Body | `last_name` | string | Yes | Last name. |
| Body | `email` | string | No | Email address. |
| Body | `phone` | string | No | Phone number. |
| Body | `department` | string | No | Department. |
| Body | `employee_id` | string | No | External employee ID. |
| Body | `tags` | string[] | No | Initial tags. |
| Body | `metadata` | object | No | Arbitrary key-value metadata. |

```json
{
  "first_name": "Bob",
  "last_name": "Williams",
  "email": "bwilliams@internal.local",
  "department": "Sales",
  "employee_id": "EMP-1234",
  "tags": ["full-time", "trained-register"]
}
```

**Response (201):**

```json
{
  "id": "part_001",
  "first_name": "Bob",
  "last_name": "Williams",
  "email": "bwilliams@internal.local",
  "phone": null,
  "department": "Sales",
  "employee_id": "EMP-1234",
  "tags": [
    { "id": "tag_01", "value": "full-time" },
    { "id": "tag_02", "value": "trained-register" }
  ],
  "metadata": {},
  "created_at": "2026-04-07T15:00:00Z",
  "updated_at": "2026-04-07T15:00:00Z",
  "deleted_at": null
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 409 | Duplicate employee_id. |
| 422 | Validation failure. |

---

### 10.2 GET /api/v1/participants

List participants with pagination.

**Required Role:** Any authenticated user (filtered by data scope).
**Required Permission:** `participants.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Query | `cursor` | string | No | Pagination cursor. |
| Query | `limit` | integer | No | Items per page (default 25, max 100). |

**Response (200):** Paginated list of participant objects.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |

---

### 10.3 GET /api/v1/participants/:id

Get a single participant.

**Required Role:** Any authenticated user (within data scope).
**Required Permission:** `participants.read`

**Response (200):** Full participant object.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Participant not found. |

---

### 10.4 PUT /api/v1/participants/:id

Update a participant.

**Required Role:** `manager` or `admin`
**Required Permission:** `participants.update`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Participant ID. |
| Body | `first_name` | string | No | Updated first name. |
| Body | `last_name` | string | No | Updated last name. |
| Body | `email` | string | No | Updated email. |
| Body | `phone` | string | No | Updated phone. |
| Body | `department` | string | No | Updated department. |
| Body | `metadata` | object | No | Updated metadata (merged). |

**Response (200):** Full updated participant object.

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Invalid request body. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Participant not found. |
| 422 | Validation failure. |

---

### 10.5 DELETE /api/v1/participants/:id

Soft-delete a participant.

**Required Role:** `admin`
**Required Permission:** `participants.delete`

**Response (204):** No content.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Participant not found. |

---

### 10.6 GET /api/v1/participants/search

Search and filter participants.

**Required Role:** Any authenticated user (filtered by data scope).
**Required Permission:** `participants.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Query | `q` | string | No | Free-text search across name, email, employee_id. |
| Query | `tags` | string | No | Comma-separated tag values to filter by. |
| Query | `department` | string | No | Filter by department. |
| Query | `cursor` | string | No | Pagination cursor. |
| Query | `limit` | integer | No | Items per page. |

**Response (200):** Paginated list of matching participant objects.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |

---

### 10.7 POST /api/v1/participants/bulk

Bulk create or update participants.

**Required Role:** `admin`
**Required Permission:** `participants.bulk`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Body | `participants` | object[] | Yes | Array of participant objects. If `id` is present, update; otherwise create. |

```json
{
  "participants": [
    {
      "first_name": "Carol",
      "last_name": "Davis",
      "department": "Sales"
    },
    {
      "id": "part_001",
      "department": "Marketing"
    }
  ]
}
```

**Response (200):**

```json
{
  "created": 1,
  "updated": 1,
  "errors": [],
  "results": [
    { "index": 0, "id": "part_050", "action": "created" },
    { "index": 1, "id": "part_001", "action": "updated" }
  ]
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Invalid request body or empty array. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 422 | Partial validation failures (returned in `errors` array, successful items still processed). |

---

### 10.8 POST /api/v1/participants/:id/tags

Add tags to a participant.

**Required Role:** `manager` or `admin`
**Required Permission:** `participants.update`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Participant ID. |
| Body | `tags` | string[] | Yes | Tags to add. |

```json
{
  "tags": ["certified-forklift"]
}
```

**Response (200):**

```json
{
  "id": "part_001",
  "tags": [
    { "id": "tag_01", "value": "full-time" },
    { "id": "tag_02", "value": "trained-register" },
    { "id": "tag_03", "value": "certified-forklift" }
  ]
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Empty tags array. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Participant not found. |
| 409 | Tag already exists on participant. |

---

### 10.9 DELETE /api/v1/participants/:id/tags/:tagId

Remove a tag from a participant.

**Required Role:** `manager` or `admin`
**Required Permission:** `participants.update`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Participant ID. |
| Path | `tagId` | string | Yes | Tag ID to remove. |

**Response (204):** No content.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Participant or tag not found. |

---

### 10.10 POST /api/v1/participants/:id/credentials

Upload a credential file for a participant (e.g. certification, license).

**Required Role:** `manager` or `admin`
**Required Permission:** `participants.update`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Participant ID. |
| Body | `file_id` | string | Yes | ID of uploaded file. |
| Body | `credential_type` | string | Yes | Type (e.g. `license`, `certification`, `id_document`). |
| Body | `expires_at` | string | No | ISO 8601 expiration date. |
| Body | `notes` | string | No | Notes about the credential. |

```json
{
  "file_id": "file_def",
  "credential_type": "certification",
  "expires_at": "2027-04-07T00:00:00Z",
  "notes": "Food safety certification."
}
```

**Response (201):**

```json
{
  "id": "cred_001",
  "participant_id": "part_001",
  "file_id": "file_def",
  "credential_type": "certification",
  "expires_at": "2027-04-07T00:00:00Z",
  "notes": "Food safety certification.",
  "created_at": "2026-04-07T15:00:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Participant or file not found. |
| 422 | Invalid credential type. |

---

### 10.11 GET /api/v1/participants/:id/credentials

List credentials for a participant.

**Required Role:** Any authenticated user (within data scope).
**Required Permission:** `participants.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Participant ID. |

**Response (200):**

```json
{
  "data": [
    {
      "id": "cred_001",
      "file_id": "file_def",
      "credential_type": "certification",
      "expires_at": "2027-04-07T00:00:00Z",
      "notes": "Food safety certification.",
      "created_at": "2026-04-07T15:00:00Z"
    }
  ]
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Participant not found. |

---

### 10.12 POST /api/v1/teams

Create a team.

**Required Role:** `manager` or `admin`
**Required Permission:** `teams.create`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Body | `name` | string | Yes | Team name. |
| Body | `description` | string | No | Description. |
| Body | `location_id` | string | No | Associated location. |

```json
{
  "name": "Morning Shift A",
  "description": "Weekday morning crew.",
  "location_id": "loc_01"
}
```

**Response (201):**

```json
{
  "id": "team_001",
  "name": "Morning Shift A",
  "description": "Weekday morning crew.",
  "location_id": "loc_01",
  "member_count": 0,
  "created_at": "2026-04-07T15:00:00Z",
  "updated_at": "2026-04-07T15:00:00Z",
  "deleted_at": null
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 409 | Team name already exists at the given location. |
| 422 | Validation failure. |

---

### 10.13 GET /api/v1/teams

List teams with pagination.

**Required Role:** Any authenticated user (filtered by data scope).
**Required Permission:** `teams.read`

**Response (200):** Paginated list of team objects.

---

### 10.14 GET /api/v1/teams/:id

Get a single team.

**Required Role:** Any authenticated user (within data scope).
**Required Permission:** `teams.read`

**Response (200):** Full team object.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Team not found. |

---

### 10.15 PUT /api/v1/teams/:id

Update a team.

**Required Role:** `manager` or `admin`
**Required Permission:** `teams.update`

**Response (200):** Full updated team object.

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Invalid request body. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Team not found. |
| 422 | Validation failure. |

---

### 10.16 DELETE /api/v1/teams/:id

Soft-delete a team.

**Required Role:** `admin`
**Required Permission:** `teams.delete`

**Response (204):** No content.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Team not found. |

---

### 10.17 GET /api/v1/teams/:id/members

List team members (roster).

**Required Role:** Any authenticated user (within data scope).
**Required Permission:** `teams.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Team ID. |
| Query | `cursor` | string | No | Pagination cursor. |
| Query | `limit` | integer | No | Items per page. |

**Response (200):**

```json
{
  "data": [
    {
      "id": "memb_001",
      "participant_id": "part_001",
      "first_name": "Bob",
      "last_name": "Williams",
      "department": "Sales",
      "joined_at": "2026-04-01T00:00:00Z"
    }
  ],
  "pagination": {
    "next_cursor": null,
    "has_more": false
  }
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Team not found. |

---

### 10.18 POST /api/v1/teams/:id/members

Add a member to a team.

**Required Role:** `manager` or `admin`
**Required Permission:** `teams.update`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Team ID. |
| Body | `participant_id` | string | Yes | Participant to add. |

```json
{
  "participant_id": "part_001"
}
```

**Response (201):**

```json
{
  "id": "memb_001",
  "team_id": "team_001",
  "participant_id": "part_001",
  "joined_at": "2026-04-07T15:00:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing participant_id. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Team or participant not found. |
| 409 | Participant is already a member of this team. |

---

### 10.19 DELETE /api/v1/teams/:id/members/:memberId

Remove a member from a team.

**Required Role:** `manager` or `admin`
**Required Permission:** `teams.update`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Team ID. |
| Path | `memberId` | string | Yes | Member ID. |

**Response (204):** No content.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Team or member not found. |

---

## 11. Datasets & Versions

### 11.1 POST /api/v1/datasets

Create a new dataset.

**Required Role:** `manager` or `admin`
**Required Permission:** `datasets.create`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Body | `name` | string | Yes | Dataset name. |
| Body | `description` | string | No | Description. |
| Body | `owner_user_id` | string | No | Owner (defaults to authenticated user). |
| Body | `tags` | string[] | No | Tags for categorization. |

```json
{
  "name": "Q1 2026 Sales Data",
  "description": "Aggregated sales data for Q1 2026.",
  "tags": ["sales", "quarterly"]
}
```

**Response (201):**

```json
{
  "id": "ds_001",
  "name": "Q1 2026 Sales Data",
  "description": "Aggregated sales data for Q1 2026.",
  "owner_user_id": "usr_001",
  "tags": ["sales", "quarterly"],
  "version_count": 0,
  "created_at": "2026-04-07T15:00:00Z",
  "updated_at": "2026-04-07T15:00:00Z",
  "deleted_at": null
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 409 | Dataset name already exists. |
| 422 | Validation failure. |

---

### 11.2 GET /api/v1/datasets

List datasets with pagination.

**Required Role:** Any authenticated user (filtered by data scope).
**Required Permission:** `datasets.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Query | `cursor` | string | No | Pagination cursor. |
| Query | `limit` | integer | No | Items per page (default 25, max 100). |
| Query | `tags` | string | No | Comma-separated tags to filter by. |

**Response (200):** Paginated list of dataset objects.

---

### 11.3 GET /api/v1/datasets/:id

Get a single dataset.

**Required Permission:** `datasets.read`

**Response (200):** Full dataset object.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Dataset not found. |

---

### 11.4 PUT /api/v1/datasets/:id

Update a dataset.

**Required Role:** `manager` or `admin`
**Required Permission:** `datasets.update`

**Response (200):** Full updated dataset object.

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Invalid request body. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Dataset not found. |
| 422 | Validation failure. |

---

### 11.5 DELETE /api/v1/datasets/:id

Soft-delete a dataset and all its versions.

**Required Role:** `admin`
**Required Permission:** `datasets.delete`

**Response (204):** No content.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Dataset not found. |

---

### 11.6 GET /api/v1/datasets/:id/versions

List versions of a dataset. Uses indexed queries for performance.

**Required Permission:** `datasets.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Dataset ID. |
| Query | `cursor` | string | No | Pagination cursor. |
| Query | `limit` | integer | No | Items per page. |
| Query | `type` | string | No | Filter by version type: `raw`, `cleaned`, `feature`, `result`. |

**Response (200):**

```json
{
  "data": [
    {
      "id": "ver_001",
      "dataset_id": "ds_001",
      "version_number": 1,
      "type": "raw",
      "parent_version_ids": [],
      "transformation_note": null,
      "record_count": 15000,
      "created_by": "usr_001",
      "created_at": "2026-04-01T10:00:00Z"
    },
    {
      "id": "ver_002",
      "dataset_id": "ds_001",
      "version_number": 2,
      "type": "cleaned",
      "parent_version_ids": ["ver_001"],
      "transformation_note": "Removed duplicates and null rows.",
      "record_count": 14200,
      "created_by": "usr_001",
      "created_at": "2026-04-02T10:00:00Z"
    }
  ],
  "pagination": {
    "next_cursor": null,
    "has_more": false
  }
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Dataset not found. |

---

### 11.7 POST /api/v1/datasets/:id/versions

Create a new version of a dataset.

**Required Role:** `manager` or `admin`
**Required Permission:** `datasets.version.create`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Dataset ID. |
| Body | `type` | string | Yes | Version type: `raw`, `cleaned`, `feature`, `result`. |
| Body | `parent_version_ids` | string[] | No | IDs of parent versions (for lineage tracking). |
| Body | `transformation_note` | string | No | Description of what transformation was applied. |
| Body | `file_id` | string | No | ID of uploaded data file. |
| Body | `record_count` | integer | No | Number of records in this version. |
| Body | `field_dictionary` | object[] | No | Initial field dictionary entries. |

```json
{
  "type": "cleaned",
  "parent_version_ids": ["ver_001"],
  "transformation_note": "Removed duplicates and null rows.",
  "file_id": "file_xyz",
  "record_count": 14200,
  "field_dictionary": [
    {
      "field_name": "sale_date",
      "data_type": "date",
      "description": "Date of the sale transaction.",
      "nullable": false
    },
    {
      "field_name": "amount_cents",
      "data_type": "integer",
      "description": "Sale amount in cents.",
      "nullable": false
    }
  ]
}
```

**Response (201):**

```json
{
  "id": "ver_002",
  "dataset_id": "ds_001",
  "version_number": 2,
  "type": "cleaned",
  "parent_version_ids": ["ver_001"],
  "transformation_note": "Removed duplicates and null rows.",
  "file_id": "file_xyz",
  "record_count": 14200,
  "created_by": "usr_001",
  "created_at": "2026-04-02T10:00:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Dataset or parent version not found. |
| 422 | Invalid type or parent version from a different dataset. |

---

### 11.8 GET /api/v1/datasets/:datasetId/versions/:versionId

Get version detail with lineage summary and field dictionary.

**Required Permission:** `datasets.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `datasetId` | string | Yes | Dataset ID. |
| Path | `versionId` | string | Yes | Version ID. |

**Response (200):**

```json
{
  "id": "ver_002",
  "dataset_id": "ds_001",
  "version_number": 2,
  "type": "cleaned",
  "parent_version_ids": ["ver_001"],
  "transformation_note": "Removed duplicates and null rows.",
  "file_id": "file_xyz",
  "record_count": 14200,
  "lineage": {
    "depth": 1,
    "parents": [
      {
        "id": "ver_001",
        "version_number": 1,
        "type": "raw"
      }
    ]
  },
  "field_dictionary": [
    {
      "field_name": "sale_date",
      "data_type": "date",
      "description": "Date of the sale transaction.",
      "nullable": false
    },
    {
      "field_name": "amount_cents",
      "data_type": "integer",
      "description": "Sale amount in cents.",
      "nullable": false
    }
  ],
  "created_by": "usr_001",
  "created_at": "2026-04-02T10:00:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Dataset or version not found. |

---

### 11.9 POST /api/v1/datasets/:datasetId/versions/:versionId/rollback

Rollback to a previous version. Creates a new version that points to the old version's data.

**Required Role:** `admin`
**Required Permission:** `datasets.version.rollback`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `datasetId` | string | Yes | Dataset ID. |
| Path | `versionId` | string | Yes | Version ID to rollback to. |
| Body | `reason` | string | Yes | Reason for rollback. |

```json
{
  "reason": "Cleaning step introduced data corruption."
}
```

**Response (202):**

```json
{
  "status": "pending_approval",
  "approval_request_id": "apr_roll_001",
  "message": "Dataset version rollback requires approval."
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing reason. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Dataset or version not found. |
| 409 | Version is already the current version. |

**Notes:**
- Always requires approval. Returns 202 with an `approval_request_id`.
- On approval, a new version is created with the same data as the target version and a `transformation_note` indicating it is a rollback.

---

### 11.10 GET /api/v1/datasets/:datasetId/versions/:versionId/lineage

Get the full lineage graph for a version.

**Required Permission:** `datasets.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `datasetId` | string | Yes | Dataset ID. |
| Path | `versionId` | string | Yes | Version ID. |

**Response (200):**

```json
{
  "version_id": "ver_003",
  "nodes": [
    {
      "id": "ver_001",
      "version_number": 1,
      "type": "raw",
      "created_at": "2026-04-01T10:00:00Z"
    },
    {
      "id": "ver_002",
      "version_number": 2,
      "type": "cleaned",
      "created_at": "2026-04-02T10:00:00Z"
    },
    {
      "id": "ver_003",
      "version_number": 3,
      "type": "feature",
      "created_at": "2026-04-03T10:00:00Z"
    }
  ],
  "edges": [
    { "from": "ver_001", "to": "ver_002" },
    { "from": "ver_002", "to": "ver_003" }
  ]
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Dataset or version not found. |

---

### 11.11 GET /api/v1/datasets/:datasetId/versions/:versionId/field-dictionary

Get the field dictionary for a dataset version.

**Required Permission:** `datasets.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `datasetId` | string | Yes | Dataset ID. |
| Path | `versionId` | string | Yes | Version ID. |

**Response (200):**

```json
{
  "version_id": "ver_002",
  "fields": [
    {
      "field_name": "sale_date",
      "data_type": "date",
      "description": "Date of the sale transaction.",
      "nullable": false,
      "example_value": "2026-03-15"
    },
    {
      "field_name": "amount_cents",
      "data_type": "integer",
      "description": "Sale amount in cents.",
      "nullable": false,
      "example_value": "4599"
    }
  ]
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Dataset or version not found. |

---

### 11.12 PUT /api/v1/datasets/:datasetId/versions/:versionId/field-dictionary

Update the field dictionary for a dataset version.

**Required Role:** `manager` or `admin`
**Required Permission:** `datasets.version.update`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `datasetId` | string | Yes | Dataset ID. |
| Path | `versionId` | string | Yes | Version ID. |
| Body | `fields` | object[] | Yes | Complete field dictionary (replaces existing). |

```json
{
  "fields": [
    {
      "field_name": "sale_date",
      "data_type": "date",
      "description": "Date of the sale transaction.",
      "nullable": false,
      "example_value": "2026-03-15"
    },
    {
      "field_name": "amount_cents",
      "data_type": "integer",
      "description": "Sale amount in cents.",
      "nullable": false,
      "example_value": "4599"
    },
    {
      "field_name": "store_id",
      "data_type": "string",
      "description": "Location identifier.",
      "nullable": false,
      "example_value": "loc_01"
    }
  ]
}
```

**Response (200):** Returns the updated field dictionary (same shape as GET).

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Invalid request body or empty fields array. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Dataset or version not found. |
| 422 | Validation failure (duplicate field names, invalid data types). |

---

## 12. Notifications

### 12.1 POST /api/v1/notification-templates

Create a notification template.

**Required Role:** `admin`
**Required Permission:** `notifications.templates.manage`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Body | `slug` | string | Yes | Unique template slug. |
| Body | `subject` | string | Yes | Notification subject template. Supports `{{variable}}` interpolation. |
| Body | `body` | string | Yes | Notification body template. |
| Body | `channel` | string | Yes | Delivery channel: `in_app`, `email`, `both`. |

```json
{
  "slug": "approval_requested",
  "subject": "Approval Required: {{action}}",
  "body": "User {{requester_name}} has requested approval for {{action}} on {{resource_type}} {{resource_id}}.",
  "channel": "in_app"
}
```

**Response (201):**

```json
{
  "id": "ntpl_001",
  "slug": "approval_requested",
  "subject": "Approval Required: {{action}}",
  "body": "User {{requester_name}} has requested approval for {{action}} on {{resource_type}} {{resource_id}}.",
  "channel": "in_app",
  "created_at": "2026-04-07T15:00:00Z",
  "updated_at": "2026-04-07T15:00:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields. |
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 409 | Slug already exists. |
| 422 | Invalid channel or template syntax. |

---

### 12.2 GET /api/v1/notification-templates

List notification templates.

**Required Role:** `admin`

**Response (200):** Paginated list of template objects.

---

### 12.3 GET /api/v1/notification-templates/:id

Get a single notification template.

**Required Role:** `admin`

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 404 | Template not found. |

---

### 12.4 PUT /api/v1/notification-templates/:id

Update a notification template.

**Required Role:** `admin`

**Response (200):** Full updated template object.

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Invalid request body. |
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 404 | Template not found. |
| 422 | Validation failure. |

---

### 12.5 DELETE /api/v1/notification-templates/:id

Soft-delete a notification template.

**Required Role:** `admin`

**Response (204):** No content.

---

### 12.6 GET /api/v1/notifications

List in-app notifications for the current user.

**Required Role:** Any authenticated user.

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Query | `cursor` | string | No | Pagination cursor. |
| Query | `limit` | integer | No | Items per page. |
| Query | `read` | boolean | No | Filter by read status. |

**Response (200):**

```json
{
  "data": [
    {
      "id": "notif_001",
      "subject": "Approval Required: order.reversal_over_24h",
      "body": "User Alice Smith has requested approval for order.reversal_over_24h on order ord_999.",
      "read": false,
      "created_at": "2026-04-07T15:00:00Z"
    }
  ],
  "pagination": {
    "next_cursor": null,
    "has_more": false
  },
  "unread_count": 1
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |

---

### 12.7 PATCH /api/v1/notifications/:id/read

Mark a single notification as read.

**Required Role:** Any authenticated user (own notifications only).

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Notification ID. |

**Response (200):**

```json
{
  "id": "notif_001",
  "read": true,
  "read_at": "2026-04-07T15:30:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 404 | Notification not found or not owned by caller. |

---

### 12.8 PATCH /api/v1/notifications/read-all

Mark all unread notifications as read for the current user.

**Required Role:** Any authenticated user.

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |

**Response (200):**

```json
{
  "marked_count": 5
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |

---

### 12.9 GET /api/v1/notifications/:id/delivery-log

View delivery attempts and retries for a notification.

**Required Role:** `admin`
**Required Permission:** `notifications.delivery.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | Notification ID. |

**Response (200):**

```json
{
  "notification_id": "notif_001",
  "attempts": [
    {
      "attempt_number": 1,
      "channel": "in_app",
      "status": "delivered",
      "attempted_at": "2026-04-07T15:00:01Z",
      "error": null
    }
  ]
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |
| 404 | Notification not found. |

---

## 13. Reporting & Analytics

### 13.1 GET /api/v1/reports/kpis

Query KPI aggregates with configurable dimensions and filters.

**Required Role:** `manager` or `admin`
**Required Permission:** `reports.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Query | `dimensions` | string | Yes | Comma-separated dimension keys (e.g. `location,date`). |
| Query | `filters` | string | No | JSON-encoded filter object (e.g. `{"location_id":"loc_01","date_from":"2026-04-01"}`). |
| Query | `metrics` | string | No | Comma-separated metric keys (e.g. `total_sales,order_count,avg_order_value`). Defaults to all. |

**Response (200):**

```json
{
  "dimensions": ["location", "date"],
  "metrics": ["total_sales_cents", "order_count", "avg_order_value_cents"],
  "rows": [
    {
      "location": "loc_01",
      "date": "2026-04-07",
      "total_sales_cents": 1523400,
      "order_count": 142,
      "avg_order_value_cents": 10728
    }
  ],
  "summary": {
    "total_sales_cents": 1523400,
    "order_count": 142,
    "avg_order_value_cents": 10728
  }
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing dimensions parameter. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 422 | Unknown dimension or metric key, invalid filter JSON. |

---

### 13.2 POST /api/v1/report-definitions

Create a report definition with configurable dimensions, filters, and chart definitions.

**Required Role:** `manager` or `admin`
**Required Permission:** `reports.definitions.manage`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Body | `name` | string | Yes | Report name. |
| Body | `description` | string | No | Description. |
| Body | `dimensions` | string[] | Yes | Dimension keys. |
| Body | `metrics` | string[] | Yes | Metric keys. |
| Body | `filters` | object | No | Default filter configuration. |
| Body | `chart_type` | string | No | Chart type hint: `bar`, `line`, `pie`, `table`. |
| Body | `export_formats` | string[] | No | Allowed export formats: `excel`, `pdf`. |

```json
{
  "name": "Daily Sales by Location",
  "description": "Daily breakdown of sales per store location.",
  "dimensions": ["location", "date"],
  "metrics": ["total_sales_cents", "order_count"],
  "filters": {
    "date_range": "last_30_days"
  },
  "chart_type": "bar",
  "export_formats": ["excel", "pdf"]
}
```

**Response (201):**

```json
{
  "id": "rdef_001",
  "name": "Daily Sales by Location",
  "description": "Daily breakdown of sales per store location.",
  "dimensions": ["location", "date"],
  "metrics": ["total_sales_cents", "order_count"],
  "filters": { "date_range": "last_30_days" },
  "chart_type": "bar",
  "export_formats": ["excel", "pdf"],
  "created_by": "usr_001",
  "created_at": "2026-04-07T15:00:00Z",
  "updated_at": "2026-04-07T15:00:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 409 | Report name already exists. |
| 422 | Unknown dimension or metric keys. |

---

### 13.3 GET /api/v1/report-definitions

List report definitions.

**Required Role:** `manager` or `admin`
**Required Permission:** `reports.definitions.read`

**Response (200):** Paginated list of report definition objects.

---

### 13.4 GET /api/v1/report-definitions/:id

Get a single report definition.

**Required Permission:** `reports.definitions.read`

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Report definition not found. |

---

### 13.5 PUT /api/v1/report-definitions/:id

Update a report definition.

**Required Permission:** `reports.definitions.manage`

**Response (200):** Full updated report definition object.

---

### 13.6 DELETE /api/v1/report-definitions/:id

Soft-delete a report definition.

**Required Permission:** `reports.definitions.manage`

**Response (204):** No content.

---

### 13.7 POST /api/v1/reports/generate

Trigger asynchronous report generation.

**Required Role:** `manager` or `admin`
**Required Permission:** `reports.generate`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Body | `report_definition_id` | string | Yes | Report definition to generate. |
| Body | `filters` | object | No | Override filters for this run. |
| Body | `export_format` | string | Yes | `excel` or `pdf`. |

```json
{
  "report_definition_id": "rdef_001",
  "filters": {
    "date_from": "2026-03-01",
    "date_to": "2026-03-31"
  },
  "export_format": "excel"
}
```

**Response (202):**

```json
{
  "job_id": "job_001",
  "status": "queued",
  "created_at": "2026-04-07T15:00:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Report definition not found. |
| 422 | Invalid export format or filter values. |
| 429 | Too many concurrent report generation requests. |

**Notes:**
- Report generation is asynchronous. Poll the job status endpoint.
- Rate limit: 5 concurrent generation jobs per user.

---

### 13.8 GET /api/v1/reports/jobs/:jobId

Check the progress of a report generation job.

**Required Role:** Job owner, `manager`, or `admin`
**Required Permission:** `reports.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `jobId` | string | Yes | Job ID. |

**Response (200):**

```json
{
  "job_id": "job_001",
  "status": "running",
  "progress_pct": 45,
  "report_definition_id": "rdef_001",
  "export_format": "excel",
  "created_at": "2026-04-07T15:00:00Z",
  "started_at": "2026-04-07T15:00:05Z",
  "completed_at": null,
  "error": null
}
```

Possible `status` values: `queued`, `running`, `completed`, `failed`.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Job not found. |

---

### 13.9 GET /api/v1/reports/jobs/:jobId/download

Download the completed report export file.

**Required Role:** Job owner, `manager`, or `admin`
**Required Permission:** `reports.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `jobId` | string | Yes | Job ID. |

**Response (200):** Binary file download with appropriate `Content-Type` (`application/vnd.openxmlformats-officedocument.spreadsheetml.sheet` for Excel, `application/pdf` for PDF) and `Content-Disposition` header.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Job not found or not completed. |
| 409 | Job failed or is still running. |

---

### 13.10 GET /api/v1/reports/scheduled

List scheduled reports.

**Required Role:** `manager` or `admin`
**Required Permission:** `reports.scheduled.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Query | `cursor` | string | No | Pagination cursor. |
| Query | `limit` | integer | No | Items per page. |

**Response (200):**

```json
{
  "data": [
    {
      "id": "sched_001",
      "report_definition_id": "rdef_001",
      "cron_expression": "0 6 * * 1",
      "export_format": "excel",
      "filters": { "date_range": "last_7_days" },
      "recipient_user_ids": ["usr_001", "usr_010"],
      "enabled": true,
      "last_run_at": "2026-03-31T06:00:00Z",
      "next_run_at": "2026-04-07T06:00:00Z",
      "created_by": "usr_001",
      "created_at": "2026-03-01T10:00:00Z"
    }
  ],
  "pagination": {
    "next_cursor": null,
    "has_more": false
  }
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |

---

### 13.11 POST /api/v1/reports/scheduled

Create a scheduled report.

**Required Role:** `manager` or `admin`
**Required Permission:** `reports.scheduled.manage`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Body | `report_definition_id` | string | Yes | Report definition ID. |
| Body | `cron_expression` | string | Yes | Cron schedule expression. |
| Body | `export_format` | string | Yes | `excel` or `pdf`. |
| Body | `filters` | object | No | Override filters. |
| Body | `recipient_user_ids` | string[] | Yes | Users who receive the generated report notification. |
| Body | `enabled` | boolean | No | Whether the schedule is active (default true). |

```json
{
  "report_definition_id": "rdef_001",
  "cron_expression": "0 6 * * 1",
  "export_format": "excel",
  "filters": { "date_range": "last_7_days" },
  "recipient_user_ids": ["usr_001", "usr_010"],
  "enabled": true
}
```

**Response (201):**

```json
{
  "id": "sched_002",
  "report_definition_id": "rdef_001",
  "cron_expression": "0 6 * * 1",
  "export_format": "excel",
  "filters": { "date_range": "last_7_days" },
  "recipient_user_ids": ["usr_001", "usr_010"],
  "enabled": true,
  "next_run_at": "2026-04-14T06:00:00Z",
  "created_by": "usr_001",
  "created_at": "2026-04-07T15:00:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing required fields. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | Report definition or recipient user not found. |
| 422 | Invalid cron expression or export format. |

---

## 14. Files

### 14.1 POST /api/v1/files/upload

Upload a file. Accepted types: PDF, JPG, PNG, CSV, XLSX. Maximum size: 10 MB.

**Required Role:** Any authenticated user.
**Required Permission:** `files.upload`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Header | `Content-Type` | string | Yes | `multipart/form-data` |
| Body | `file` | binary | Yes | The file to upload. |
| Body | `description` | string | No | Optional description. |

**Response (201):**

```json
{
  "id": "file_abc",
  "filename": "receipt_scan.pdf",
  "content_type": "application/pdf",
  "size_bytes": 245760,
  "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "description": null,
  "uploaded_by": "usr_042",
  "created_at": "2026-04-07T16:00:00Z",
  "deleted_at": null
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | No file provided. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 422 | File type not allowed or file exceeds 10 MB. |
| 429 | Upload rate limit exceeded. |

**Notes:**
- A SHA-256 fingerprint is computed and stored on upload.
- Duplicate files (same SHA-256) are deduplicated at the storage level but receive unique file IDs.

---

### 14.2 GET /api/v1/files/:id

Download a file.

**Required Role:** Any authenticated user (must have access to the resource the file is attached to).
**Required Permission:** `files.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | File ID. |

**Response (200):** Binary file download with appropriate `Content-Type` and `Content-Disposition` headers.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | File not found or soft-deleted. |

---

### 14.3 GET /api/v1/files/:id/metadata

Get file metadata including checksum.

**Required Role:** Any authenticated user.
**Required Permission:** `files.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | File ID. |

**Response (200):**

```json
{
  "id": "file_abc",
  "filename": "receipt_scan.pdf",
  "content_type": "application/pdf",
  "size_bytes": 245760,
  "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "description": null,
  "uploaded_by": "usr_042",
  "created_at": "2026-04-07T16:00:00Z",
  "deleted_at": null
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | File not found. |

---

### 14.4 DELETE /api/v1/files/:id

Soft-delete a file.

**Required Role:** File uploader, `manager`, or `admin`
**Required Permission:** `files.delete`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Path | `id` | string | Yes | File ID. |

**Response (204):** No content.

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 404 | File not found. |

**Notes:**
- Soft-deletes the file (sets `deleted_at`). The physical file is retained for audit purposes.

---

## 15. Audit Log

### 15.1 GET /api/v1/audit-log

Query the audit log.

**Required Role:** `admin` or `auditor`
**Required Permission:** `audit_log.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Query | `cursor` | string | No | Pagination cursor. |
| Query | `limit` | integer | No | Items per page (default 25, max 100). |
| Query | `user_id` | string | No | Filter by acting user. |
| Query | `action` | string | No | Filter by action (e.g. `order.create`, `user.lock`). |
| Query | `resource_type` | string | No | Filter by resource type (e.g. `order`, `user`, `dataset`). |
| Query | `resource_id` | string | No | Filter by specific resource ID. |
| Query | `time_from` | string | No | ISO 8601 start time (inclusive). |
| Query | `time_to` | string | No | ISO 8601 end time (inclusive). |

**Response (200):**

```json
{
  "data": [
    {
      "id": "audit_001",
      "user_id": "usr_042",
      "username": "asmith",
      "action": "order.create",
      "resource_type": "order",
      "resource_id": "ord_1001",
      "details": {
        "location_id": "loc_01",
        "total_cents": 4318
      },
      "ip_address": "10.0.1.50",
      "timestamp": "2026-04-07T16:00:00Z"
    }
  ],
  "pagination": {
    "next_cursor": "eyJ0cyI6IjIwMjYtMDQtMDdUMTU6NTk6MDAifQ",
    "has_more": true
  }
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 422 | Invalid filter parameters. |

**Notes:**
- Audit log entries are immutable and cannot be modified or deleted.
- Entries are retained indefinitely.

---

### 15.2 POST /api/v1/audit-log/export

Export audit log records to CSV or PDF. Requires approval for bulk exports.

**Required Role:** `admin`
**Required Permission:** `audit_log.export`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |
| Body | `format` | string | Yes | Export format: `csv` or `pdf`. |
| Body | `filters` | object | No | Same filter keys as GET /api/v1/audit-log. |

```json
{
  "format": "csv",
  "filters": {
    "time_from": "2026-04-01T00:00:00Z",
    "time_to": "2026-04-07T23:59:59Z",
    "resource_type": "order"
  }
}
```

**Response (202):**

```json
{
  "status": "pending_approval",
  "approval_request_id": "apr_audit_001",
  "message": "Bulk audit log export requires approval."
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 400 | Missing format. |
| 401 | Unauthenticated. |
| 403 | Insufficient permissions. |
| 422 | Invalid format or filter parameters. |

**Notes:**
- Bulk audit log exports always require approval.
- On approval, the export is generated asynchronously and the requester is notified via the notifications system.

---

## 16. Health & Metrics

### 16.1 GET /api/v1/health

Health check endpoint. No authentication required.

**Required Role:** None (public).

**Request:** No parameters.

**Response (200):**

```json
{
  "status": "healthy",
  "checks": {
    "database": {
      "status": "up",
      "latency_ms": 2
    },
    "disk_space": {
      "status": "ok",
      "free_gb": 45.2,
      "total_gb": 100.0
    },
    "uptime_seconds": 864000
  },
  "version": "1.0.0",
  "timestamp": "2026-04-07T15:00:00Z"
}
```

If any check fails, the overall status is `"degraded"` and the HTTP status code is still 200 (to distinguish from the service being completely down). If the service itself is unresponsive, the caller will receive a connection error.

---

### 16.2 GET /api/v1/metrics

Operational metrics for monitoring.

**Required Role:** `admin`
**Required Permission:** `metrics.read`

**Request:**

| Location | Field | Type | Required | Description |
|---|---|---|---|---|
| Header | `Authorization` | string | Yes | `Bearer <session_token>` |

**Response (200):**

```json
{
  "request_count": {
    "total": 542310,
    "last_1h": 1523,
    "last_24h": 34210
  },
  "latency": {
    "p50_ms": 12,
    "p95_ms": 85,
    "p99_ms": 245
  },
  "database": {
    "pool_size": 20,
    "active_connections": 8,
    "idle_connections": 12,
    "waiting_requests": 0
  },
  "sessions": {
    "active": 24,
    "expired_last_24h": 156
  },
  "collected_at": "2026-04-07T15:00:00Z"
}
```

**Error Codes:**

| Code | Condition |
|---|---|
| 401 | Unauthenticated. |
| 403 | Caller is not admin. |

---

## Appendix A: Permission Points

| Slug | Description | Default Roles |
|---|---|---|
| `auth.change_password` | Change own password. | `cashier`, `manager`, `admin` |
| `users.manage` | Create, update, delete, lock, unlock user accounts. | `admin` |
| `roles.manage` | Create, update, delete roles and permission bindings. | `admin` |
| `data_scopes.manage` | Manage data scope definitions. | `admin` |
| `delegations.create` | Create time-bounded delegations. | `manager`, `admin` |
| `delegations.revoke` | Revoke delegations. | `manager`, `admin` |
| `approval_configs.manage` | Update approval workflow configuration. | `admin` |
| `approval_requests.review` | View and act on approval requests. | `manager`, `admin` |
| `orders.create` | Create new POS orders. | `cashier`, `manager`, `admin` |
| `orders.read` | View POS orders. | `cashier`, `manager`, `admin` |
| `orders.update` | Update orders and order lines. | `cashier`, `manager`, `admin` |
| `orders.tender` | Add tender entries to orders. | `cashier`, `manager`, `admin` |
| `orders.return` | Initiate returns. | `cashier`, `manager`, `admin` |
| `orders.exchange` | Initiate exchanges. | `cashier`, `manager`, `admin` |
| `orders.reverse` | Initiate reversals (full void). | `manager`, `admin` |
| `registers.close` | Submit register close records. | `cashier`, `manager`, `admin` |
| `registers.read` | View register close records. | `cashier`, `manager`, `admin` |
| `registers.confirm` | Manager confirmation of close records. | `manager`, `admin` |
| `participants.create` | Create participant records. | `manager`, `admin` |
| `participants.read` | View participant records. | `cashier`, `manager`, `admin` |
| `participants.update` | Update participant records and tags. | `manager`, `admin` |
| `participants.delete` | Delete participant records. | `admin` |
| `participants.bulk` | Bulk create/update participants. | `admin` |
| `teams.create` | Create teams. | `manager`, `admin` |
| `teams.read` | View teams and rosters. | `cashier`, `manager`, `admin` |
| `teams.update` | Update teams and manage members. | `manager`, `admin` |
| `teams.delete` | Delete teams. | `admin` |
| `datasets.create` | Create datasets. | `manager`, `admin` |
| `datasets.read` | View datasets and versions. | `cashier`, `manager`, `admin` |
| `datasets.update` | Update dataset metadata. | `manager`, `admin` |
| `datasets.delete` | Delete datasets. | `admin` |
| `datasets.version.create` | Create new dataset versions. | `manager`, `admin` |
| `datasets.version.update` | Update version field dictionary. | `manager`, `admin` |
| `datasets.version.rollback` | Rollback dataset versions (triggers approval). | `admin` |
| `notifications.templates.manage` | Manage notification templates. | `admin` |
| `notifications.delivery.read` | View notification delivery logs. | `admin` |
| `reports.read` | View KPIs and report outputs. | `manager`, `admin` |
| `reports.definitions.read` | View report definitions. | `manager`, `admin` |
| `reports.definitions.manage` | Create, update, delete report definitions. | `manager`, `admin` |
| `reports.generate` | Trigger report generation. | `manager`, `admin` |
| `reports.scheduled.read` | View scheduled reports. | `manager`, `admin` |
| `reports.scheduled.manage` | Create and manage scheduled reports. | `manager`, `admin` |
| `files.upload` | Upload files. | `cashier`, `manager`, `admin` |
| `files.read` | Download files and view metadata. | `cashier`, `manager`, `admin` |
| `files.delete` | Delete files. | `manager`, `admin` |
| `audit_log.read` | Query the audit log. | `admin`, `auditor` |
| `audit_log.export` | Export audit log records (triggers approval). | `admin` |
| `sensitive_data.view` | View unmasked sensitive fields. | `admin` |
| `metrics.read` | View operational metrics. | `admin` |
| `api_scopes.manage` | Manage API scope definitions. | `admin` |
| `menu_scopes.manage` | Manage menu scope definitions. | `admin` |

---

## Appendix B: Order State Transitions

| From State | To State | Required Role | Conditions |
|---|---|---|---|
| _(new)_ | `open` | `cashier`, `manager`, `admin` | Order creation. |
| `open` | `completed` | `cashier`, `manager`, `admin` | Total tenders must equal or exceed order total. |
| `open` | `voided` | `manager`, `admin` | Order has no processed tenders, or all tenders are reversible. |
| `completed` | `returned` | `cashier`, `manager`, `admin` | All lines fully returned via the return endpoint. |
| `completed` | `partially_returned` | `cashier`, `manager`, `admin` | Some lines returned, others remain. |
| `completed` | `reversed` | `manager`, `admin` | Full reversal processed. If order is older than 24 hours, requires approval. |
| `partially_returned` | `returned` | `cashier`, `manager`, `admin` | Remaining lines returned. |
| `partially_returned` | `reversed` | `manager`, `admin` | Full reversal of remaining balance. Requires approval if older than 24 hours. |
| `voided` | _(terminal)_ | | No further transitions allowed. |
| `returned` | _(terminal)_ | | No further transitions allowed. |
| `reversed` | _(terminal)_ | | No further transitions allowed. |

**Notes:**
- State transitions are enforced server-side. Invalid transitions return 409 Conflict.
- All state transitions are recorded in the audit log.
- The `completed` to `reversed` transition for orders older than 24 hours returns 202 Accepted and creates an approval request.

---

## Appendix C: Idempotency Key Rules

### Endpoints Requiring X-Idempotency-Key

| Endpoint | Description |
|---|---|
| `POST /api/v1/orders` | Create order. |
| `POST /api/v1/orders/:id/lines` | Add order line. |
| `POST /api/v1/orders/:id/tenders` | Add tender entry. |
| `POST /api/v1/orders/:id/return` | Initiate return. |
| `POST /api/v1/orders/:id/exchange` | Initiate exchange. |
| `POST /api/v1/orders/:id/reversal` | Initiate reversal. |

### Rules

1. **Format:** The key must be a valid UUID v4 string.
2. **TTL:** Idempotency keys are stored for 24 hours. After expiry, the same key may be reused.
3. **Collision behavior:**
   - If a request arrives with an idempotency key that has already been successfully processed, the server returns the original response with the same HTTP status code.
   - If a request arrives with an idempotency key that is currently being processed (in-flight), the server returns `409 Conflict` with error code `IDEMPOTENCY_KEY_IN_PROGRESS`.
   - If a request arrives with an idempotency key that was used with a different request body (payload hash mismatch), the server returns `409 Conflict` with error code `IDEMPOTENCY_KEY_MISMATCH`.
4. **Missing key:** If a required `X-Idempotency-Key` header is missing, the server returns `400 Bad Request` with error code `IDEMPOTENCY_KEY_REQUIRED`.
5. **Storage:** Idempotency records are stored in the `idempotency_keys` table with columns: `key`, `endpoint`, `payload_hash`, `response_status`, `response_body`, `created_at`, `expires_at`.

---

## Appendix D: Rate Limit Headers

All responses include rate limit headers when rate limiting is configured for the endpoint:

| Header | Description |
|---|---|
| `X-RateLimit-Limit` | Maximum number of requests allowed in the current window. |
| `X-RateLimit-Remaining` | Number of requests remaining in the current window. |
| `X-RateLimit-Reset` | Unix timestamp (seconds) when the current window resets. |
| `Retry-After` | Seconds until the client should retry (only present on 429 responses). |

### Default Rate Limits

| Endpoint Pattern | Limit | Window |
|---|---|---|
| `POST /api/v1/auth/login` | 10 | 1 minute (per IP) |
| `POST /api/v1/auth/change-password` | 5 | 15 minutes (per user) |
| `POST /api/v1/files/upload` | 20 | 1 minute (per user) |
| `POST /api/v1/reports/generate` | 5 | concurrent (per user) |
| `POST /api/v1/participants/bulk` | 10 | 1 hour (per user) |
| All other endpoints | 300 | 1 minute (per user) |

Rate limits are configurable via the `rate_limits` database table. Changes take effect immediately without restart.

### 429 Response Example

```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests. Please retry after 45 seconds.",
    "details": {
      "retry_after_seconds": 45
    }
  }
}
```
