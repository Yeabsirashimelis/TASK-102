use actix_web::{web, App, HttpServer, middleware::Logger};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

mod audit;
mod auth;
mod config;
mod crypto;
mod db;
mod errors;
mod export_worker;
mod handlers;
mod models;
mod observability;
mod pos;
mod rbac;
mod routes;
mod schema;
mod security;
mod storage;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    // Initialize metrics before anything else
    observability::metrics::init();

    let config = config::AppConfig::from_env();
    let pool = db::establish_pool(&config.database_url);

    // Run pending migrations
    {
        let mut conn = pool.get().expect("Failed to get DB connection for migrations");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run database migrations");
    }

    let encryptor = crypto::FieldEncryptor::new(&config.field_encryption_key);

    // Spawn the async export background worker
    let worker_pool = std::sync::Arc::new(pool.clone());
    export_worker::spawn(worker_pool);

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());
    log::info!("Starting RetailOps API on {}", bind_addr);

    HttpServer::new(move || {
        App::new()
            // Middleware layers (outermost first):
            // 1. Structured request logging
            .wrap(Logger::new(
                "%a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T"
            ))
            // 2. Request metrics (counts, active connections)
            .wrap(observability::request_metrics::RequestMetrics)
            // 3. Audit trail for write operations
            .wrap(audit::middleware::AuditMiddleware)
            // 4. CSRF protection for state-changing requests
            .wrap(security::csrf::CsrfMiddleware)
            // Shared application state
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(encryptor.clone()))
            // JSON payload limit (10 MB for file metadata, etc.)
            .app_data(web::JsonConfig::default().limit(10_485_760))
            .configure(routes::configure)
    })
    .bind(&bind_addr)?
    .run()
    .await
}
