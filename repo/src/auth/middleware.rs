use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest};
use std::future::{ready, Ready};

use crate::auth::jwt::{decode_token, Claims};
use crate::config::AppConfig;
use crate::errors::AppError;

/// Actix extractor that validates the Bearer token and provides Claims.
pub struct AuthenticatedUser(pub Claims);

impl FromRequest for AuthenticatedUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let result = extract_claims(req);
        ready(result)
    }
}

fn extract_claims(req: &HttpRequest) -> Result<AuthenticatedUser, AppError> {
    let config = req
        .app_data::<actix_web::web::Data<AppConfig>>()
        .ok_or_else(|| AppError::Internal("AppConfig not found".into()))?;

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid Authorization header format".into()))?;

    let claims = decode_token(token, &config)?;

    if claims.token_type != "access" {
        return Err(AppError::Unauthorized("Invalid token type".into()));
    }

    Ok(AuthenticatedUser(claims))
}
