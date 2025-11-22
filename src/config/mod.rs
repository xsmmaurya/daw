// /Users/xsm/Documents/workspace/xtras/daw/src/config/mod.rs

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RustConfig {
    pub log: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub environment: String,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub rust: RustConfig,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let builder = config::Config::builder()
            .add_source(config::File::with_name("config").required(false))
            .add_source(config::Environment::with_prefix("APP").separator("__"));

        let mut settings = builder.build()?;

        settings.set_default("environment", "development")?;
        settings.set_default("server.host", "0.0.0.0")?;
        settings.set_default("server.port", 8080)?;
        settings.set_default("database.url", "postgres://daw:password@localhost:5432/daw")?;
        settings.set_default("redis.url", "redis://127.0.0.1:6379")?;
        settings.set_default("rust.log", "info,actix_web=info")?;

        let cfg: AppConfig = settings.try_deserialize()?;
        Ok(cfg)
    }
}
