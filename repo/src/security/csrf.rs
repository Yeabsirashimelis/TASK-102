use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures_util::future::{ok, LocalBoxFuture, Ready};
use std::rc::Rc;

use crate::errors::AppError;

/// CSRF protection middleware.
///
/// For state-changing requests (POST, PUT, PATCH, DELETE) that are NOT to
/// the auth endpoints, requires either:
/// - An `X-CSRF-Token` header, OR
/// - A `Content-Type: application/json` or `multipart/form-data` header
///   (these cannot be sent cross-origin without CORS preflight).
pub struct CsrfMiddleware;

impl<S, B> Transform<S, ServiceRequest> for CsrfMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CsrfService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CsrfService {
            service: Rc::new(service),
        })
    }
}

pub struct CsrfService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for CsrfService<S>
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
            let method = req.method().as_str();
            let path = req.path().to_string();

            let is_write = matches!(method, "POST" | "PUT" | "PATCH" | "DELETE");
            let is_auth_path = path.starts_with("/api/v1/auth/");
            let is_health =
                path.starts_with("/api/v1/health") || path.starts_with("/api/v1/metrics");

            if is_write && !is_auth_path && !is_health {
                let has_csrf = req.headers().contains_key("X-CSRF-Token");
                let has_safe_content = req
                    .headers()
                    .get("Content-Type")
                    .and_then(|v| v.to_str().ok())
                    .map(|ct| {
                        ct.starts_with("application/json")
                            || ct.starts_with("multipart/form-data")
                    })
                    .unwrap_or(false);

                if !has_csrf && !has_safe_content {
                    return Err(AppError::Forbidden("CSRF token missing".into()).into());
                }
            }

            svc.call(req).await
        })
    }
}
