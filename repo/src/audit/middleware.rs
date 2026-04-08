use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures_util::future::{ok, LocalBoxFuture, Ready};
use std::rc::Rc;

use crate::db::DbPool;

/// Actix middleware that records audit entries for all state-changing requests
/// (POST, PUT, PATCH, DELETE). Read-only requests (GET, HEAD, OPTIONS) are skipped.
pub struct AuditMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuditMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuditMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuditMiddlewareService {
            service: Rc::new(service),
        })
    }
}

pub struct AuditMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuditMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let method = req.method().to_string();
            let path = req.path().to_string();
            let ip = req
                .peer_addr()
                .map(|a| a.ip().to_string());

            // Extract user_id from JWT claims if present
            let user_id = req
                .headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
                .and_then(|token| {
                    // Decode just the sub claim without full validation
                    // (the auth middleware will validate fully)
                    let config = req.app_data::<actix_web::web::Data<crate::config::AppConfig>>();
                    config.and_then(|c| {
                        crate::auth::jwt::decode_token(token, c).ok()
                    })
                })
                .map(|claims| claims.sub);

            let is_write = matches!(method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE");

            let res = svc.call(req).await?;

            // Only audit write operations
            if is_write {
                let status = res.response().status().as_u16();
                let action = match method.as_str() {
                    "POST" => "create",
                    "PUT" | "PATCH" => "update",
                    "DELETE" => "delete",
                    _ => "write",
                };

                // Extract resource type from path (e.g., /api/v1/orders/... -> orders)
                let resource_type = path
                    .split('/')
                    .filter(|s| !s.is_empty() && *s != "api" && *s != "v1")
                    .next()
                    .unwrap_or("unknown")
                    .to_string();

                // Log asynchronously to not block the response
                if let Some(pool) = res
                    .request()
                    .app_data::<actix_web::web::Data<DbPool>>()
                {
                    if let Ok(mut conn) = pool.get() {
                        let metadata = serde_json::json!({
                            "response_status": status,
                        });
                        let _ = crate::audit::service::record(
                            &mut conn,
                            user_id,
                            action,
                            &resource_type,
                            None,
                            &method,
                            &path,
                            None,
                            None,
                            Some(metadata),
                            ip,
                        );
                    }
                }
            }

            Ok(res)
        })
    }
}
