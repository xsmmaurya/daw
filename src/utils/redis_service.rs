// /Users/xsm/Documents/workspace/xtras/daw/src/utils/redis_service.rs
use redis::aio::ConnectionManager;
use redis::Client;

use crate::config::AppConfig;

/// Get an async Redis connection manager
pub async fn get_redis_connection() -> anyhow::Result<ConnectionManager> {
    let cfg = AppConfig::from_env()?;
    let client = Client::open(cfg.redis.url.clone())?;
    let manager = ConnectionManager::new(client).await?;
    Ok(manager)
}
