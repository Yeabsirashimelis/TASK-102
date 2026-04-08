//! Structured JSON access logger middleware.
//! Each request produces a single JSON log line with standardized fields.

use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures_util::future::{ok, LocalBoxFuture, Ready};
use std::rc::Rc;
use std::time::Instant;

pub struct JsonLogger;

impl<S, B> Transform<S, ServiceRequest> for JsonLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JsonLoggerService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JsonLoggerService {
            service: Rc::new(service),
        })
    }
}

pub struct JsonLoggerService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JsonLoggerService<S>
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
            let start = Instant::now();
            let method = req.method().to_string();
            let path = req.path().to_string();
            let request_id = uuid::Uuid::new_v4().to_string();
            let peer = req.peer_addr().map(|a| a.ip().to_string());

            // Extract user_id from JWT if present
            let user_id = req
                .headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
                .and_then(|token| {
                    let config = req.app_data::<actix_web::web::Data<crate::config::AppConfig>>();
                    config.and_then(|c| crate::auth::jwt::decode_token(token, c).ok())
                })
                .map(|claims| claims.sub.to_string());

            let res = svc.call(req).await?;

            let latency_ms = start.elapsed().as_millis();
            let status = res.response().status().as_u16();

            let error_code = if status >= 400 {
                Some(match status {
                    400 => "VALIDATION_ERROR",
                    401 => "UNAUTHORIZED",
                    403 => "FORBIDDEN",
                    404 => "NOT_FOUND",
                    409 => "CONFLICT",
                    429 => "RATE_LIMITED",
                    _ => "SERVER_ERROR",
                })
            } else {
                None
            };

            // Emit structured JSON log line
            let log_entry = serde_json::json!({
                "request_id": request_id,
                "user_id": user_id,
                "method": method,
                "path": path,
                "status": status,
                "latency_ms": latency_ms,
                "error_code": error_code,
                "peer_ip": peer,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });

            log::info!("{}", log_entry);

            Ok(res)
        })
    }
}
