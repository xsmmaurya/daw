// /Users/xsm/Documents/workspace/xtras/daw/src/state.rs
use crate::config::AppConfig;
use sea_orm::DatabaseConnection;
use redis::aio::ConnectionManager;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: DatabaseConnection,
    pub redis: ConnectionManager,
}
