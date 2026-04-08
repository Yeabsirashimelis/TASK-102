use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use diesel::sql_query;

use crate::db::DbPool;
use crate::errors::AppError;
use crate::observability::metrics;

/// GET /api/v1/health — public endpoint, no auth required.
/// Returns service health with DB connectivity check.
pub async fn health_check(pool: web::Data<DbPool>) -> HttpResponse {
    let db_ok = match pool.get() {
        Ok(mut conn) => sql_query("SELECT 1")
            .execute(&mut conn)
            .is_ok(),
        Err(_) => false,
    };

    let pool_state = pool.state();

    let body = serde_json::json!({
        "status": if db_ok { "healthy" } else { "degraded" },
        "database": if db_ok { "connected" } else { "unreachable" },
        "pool": {
            "connections": pool_state.connections,
            "idle_connections": pool_state.idle_connections,
        },
        "timestamp": chrono::Utc::now(),
    });

    if db_ok {
        HttpResponse::Ok().json(body)
    } else {
        HttpResponse::ServiceUnavailable().json(body)
    }
}

/// GET /api/v1/metrics — requires system.health permission.
/// Returns application metrics snapshot.
pub async fn metrics_endpoint(
    pool: web::Data<DbPool>,
    auth: crate::auth::middleware::AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = crate::rbac::guard::check_permission(&auth.0, "system.health", &mut conn)?;

    let snapshot = metrics::get().snapshot();
    let pool_state = pool.state();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "application": snapshot,
        "pool": {
            "connections": pool_state.connections,
            "idle_connections": pool_state.idle_connections,
        },
    })))
}
