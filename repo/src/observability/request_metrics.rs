use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures_util::future::{ok, LocalBoxFuture, Ready};
use std::rc::Rc;

use super::metrics;

/// Middleware that counts requests and errors in the global metrics.
pub struct RequestMetrics;

impl<S, B> Transform<S, ServiceRequest> for RequestMetrics
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestMetricsService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequestMetricsService {
            service: Rc::new(service),
        })
    }
}

pub struct RequestMetricsService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestMetricsService<S>
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
            let m = metrics::get();
            m.inc_requests();
            m.inc_connections();

            let res = svc.call(req).await;

            m.dec_connections();

            match &res {
                Ok(resp) if resp.response().status().is_server_error() => {
                    m.inc_errors();
                }
                Err(_) => {
                    m.inc_errors();
                }
                _ => {}
            }

            res
        })
    }
}
