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

use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

use crate::config::AppConfig;
use crate::db::{init_db, init_redis};
use crate::state::AppState;
use migration::{Migrator, MigratorTrait};

use crate::routes::{
    auth::configure_auth_routes,
    tenant::configure_tenant_routes,
    user::configure_user_routes,
    ride::configure_ride_routes,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let cfg = AppConfig::from_env().expect("failed to load configuration");

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cfg.rust.log))
        .init();

    // tracing_subscriber::fmt()
    //     .with_env_filter(EnvFilter::from_default_env())
    //     .init();

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

    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    println!("Starting server on {addr} in {} environment", cfg.environment);

    HttpServer::new(move || {
        App::new()
            .app_data(shared_state.clone())
            .service(
                web::scope("/api/v1")
                    .configure(|cfg| configure_auth_routes(cfg, db.clone()))
                    .configure(|cfg| configure_tenant_routes(cfg, db.clone()))
                    .configure(|cfg| configure_user_routes(cfg, db.clone()))
                    .configure(|cfg| configure_ride_routes(cfg, db.clone()))
            )
    })
    .bind(addr)?
    .run()
    .await
}

