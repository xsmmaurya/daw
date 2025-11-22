// /Users/xsm/Documents/workspace/xtras/daw/src/db/mod.rs
use sea_orm::{Database, DatabaseConnection};
use redis::aio::ConnectionManager;
use redis::Client;

pub async fn init_db(db_url: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    Database::connect(db_url).await
}

pub type RedisManager = ConnectionManager;

pub async fn init_redis(redis_url: &str) -> redis::RedisResult<RedisManager> {
    let client = Client::open(redis_url)?;
    let manager = ConnectionManager::new(client).await?;
    Ok(manager)
}