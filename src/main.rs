// /Users/xsm/Documents/workspace/xtras/daw/src/main.rs
mod config;
mod db;
mod entity;
mod error;
mod handlers;
mod requests;
mod routes;
mod services;
mod state;

mod dto;
mod middleware;
mod utils;
mod jresponse;
mod types;

mod qrushes;
mod ws;

use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use std::sync::Arc;
use tracing_subscriber::EnvFilter;
use env_logger::Env;
use tracing::info;
use crate::config::AppConfig;
use crate::db::{init_db, init_redis};
use crate::state::AppState;
use migration::{Migrator, MigratorTrait};

use crate::routes::{
    auth::configure_auth_routes,
    tenant::configure_tenant_routes,
    user::configure_user_routes,
    ride::configure_ride_routes,
    driver::configure_driver_routes,
    event::configure_events_routes,
};
use crate::qrushes::qrush_init::QrushInit;
use crate::ws::init_ws_hub;
use actix_cors::Cors;
use actix_web::http::header;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let cfg = AppConfig::from_env().expect("failed to load configuration");

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cfg.rust.log))
        .init();

    let db_conn = init_db(&cfg.database.url)
        .await
        .expect("failed to connect to database");

    Migrator::up(&db_conn, None).await.expect("migration failed");

    let redis_manager = init_redis(&cfg.redis.url)
        .await
        .expect("failed to connect to redis");

    let db = web::Data::new(db_conn.clone());

    let shared_state = web::Data::new(AppState {
        config: Arc::new(cfg.clone()),
        db: db_conn.clone(),
        redis: redis_manager,
    });

    init_ws_hub();


    QrushInit::initialize(None).await;
    println!("Global Qrush initialization complete!");

    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    println!("Starting server on {addr} in {} environment", cfg.environment);

    HttpServer::new(move || {

        let qrush_worker_config = QrushInit::setup_worker_sync();

        App::new()
            .app_data(shared_state.clone())
            .wrap(build_cors())
            .route("/", web::get().to(health_check))
            // Qrush metrics routes
            .service(
                web::scope("/qrush")
                    .configure(|cfg| QrushInit::configure_routes(cfg))
            )
            .service(
                web::scope("/api/v1")
                    .configure(|cfg| configure_auth_routes(cfg, db.clone()))
                    .configure(|cfg| configure_tenant_routes(cfg, db.clone()))
                    .configure(|cfg| configure_user_routes(cfg, db.clone()))
                    .configure(|cfg| configure_ride_routes(cfg, db.clone()))
                    .configure(|cfg| configure_driver_routes(cfg, db.clone()))
                    .configure(|cfg| configure_events_routes(cfg, db.clone()))
                    .configure(routes::ws::configure_ws_routes)
            )
    })
    .bind(addr)?
    .run()
    .await
}


async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Daw is Running")
}



fn build_cors() -> Cors {
    let origins = std::env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://127.0.0.1:3000,http://localhost:3001,http://127.0.0.1:3001".to_string());

    let cors = origins
        .split(',')
        .fold(Cors::default(), |c, o| c.allowed_origin(o.trim()));

    cors
        .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
        ])
        .allowed_header(header::HeaderName::from_static("x-requested-page"))
        .allowed_header(header::HeaderName::from_static("x-requested-limit"))
        .allowed_header(header::HeaderName::from_static("x-platform"))
        .allowed_header(header::HeaderName::from_static("x-service"))
        .allowed_header(header::HeaderName::from_static("x-api-key"))
        .allowed_header(header::HeaderName::from_static("x-tenant"))
        .expose_headers(vec![
            "X-Total-Count",
            "X-Total-Pages",
            "X-Current-Page",
            "X-Per-Page",
            "X-Requested-Page",
            "X-Requested-Limit",
            "X-Result-Count",
            "X-Has-More",
        ])
        .supports_credentials()
        .max_age(24 * 60 * 60)
}

