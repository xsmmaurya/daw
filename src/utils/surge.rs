// src/utils/surge.rs

use anyhow::Result;
use redis::AsyncCommands;
use uuid::Uuid;

use crate::utils::redis_service::get_redis_connection;

/// Very simple geo-cell bucketing: ~0.01° grid
fn cell_for(lat: f64, lon: f64) -> String {
    let lat_bucket = (lat * 100.0).floor() as i32;
    let lon_bucket = (lon * 100.0).floor() as i32;
    format!("{lat_bucket}:{lon_bucket}")
}

fn demand_key(tenant_id: Uuid, cell: &str) -> String {
    format!("surge:demand:{tenant_id}:{cell}")
}

fn supply_key(tenant_id: Uuid, cell: &str) -> String {
    format!("surge:supply:{tenant_id}:{cell}")
}

/// Record a ride request as demand in the pickup cell
pub async fn record_demand(tenant_id: Uuid, lat: f64, lon: f64) -> Result<()> {
    let cell = cell_for(lat, lon);
    let mut conn = get_redis_connection().await?;
    let key = demand_key(tenant_id, &cell);
    let _: () = conn.incr(key, 1_i64).await?;
    Ok(())
}

/// Record an online driver as supply in the cell
pub async fn record_supply(tenant_id: Uuid, lat: f64, lon: f64) -> Result<()> {
    let cell = cell_for(lat, lon);
    let mut conn = get_redis_connection().await?;
    let key = supply_key(tenant_id, &cell);
    let _: () = conn.incr(key, 1_i64).await?;
    Ok(())
}

/// Compute a simple surge multiplier based on demand/supply ratio
pub async fn current_multiplier(tenant_id: Uuid, lat: f64, lon: f64) -> Result<f64> {
    let cell = cell_for(lat, lon);
    let mut conn = get_redis_connection().await?;

    let d_key = demand_key(tenant_id, &cell);
    let s_key = supply_key(tenant_id, &cell);

    let demand: i64 = conn.get(&d_key).await.unwrap_or(0);
    let supply: i64 = conn.get(&s_key).await.unwrap_or(0);

    // naive formula:
    // base 1.0, if demand >> supply, bump up to max 3.0
    let ratio = if supply <= 0 {
        3.0 // no drivers
    } else {
        demand as f64 / supply as f64
    };

    let mut surge = 1.0 + (ratio - 1.0) * 0.5;
    if surge < 1.0 {
        surge = 1.0;
    }
    if surge > 3.0 {
        surge = 3.0;
    }

    Ok(surge)
}
