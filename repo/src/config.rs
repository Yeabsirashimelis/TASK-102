use base64::Engine;

#[derive(Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_access_ttl_secs: i64,
    pub jwt_refresh_ttl_secs: i64,
    pub field_encryption_key: [u8; 32],
    pub lockout_threshold: i32,
    pub lockout_duration_secs: i64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret =
            std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let key_b64 = std::env::var("FIELD_ENCRYPTION_KEY")
            .expect("FIELD_ENCRYPTION_KEY must be set");
        let key_bytes = base64::engine::general_purpose::STANDARD
            .decode(&key_b64)
            .expect("FIELD_ENCRYPTION_KEY must be valid base64");
        let field_encryption_key: [u8; 32] = key_bytes
            .try_into()
            .expect("FIELD_ENCRYPTION_KEY must be exactly 32 bytes");

        Self {
            database_url,
            jwt_secret,
            jwt_access_ttl_secs: std::env::var("JWT_ACCESS_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(900),
            jwt_refresh_ttl_secs: std::env::var("JWT_REFRESH_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(86400),
            field_encryption_key,
            lockout_threshold: 5,
            lockout_duration_secs: 900,
        }
    }
}
