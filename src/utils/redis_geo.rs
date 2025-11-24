// /Users/xsm/Documents/workspace/xtras/daw/src/utils/redis_geo.rs

use uuid::Uuid;
use redis::AsyncCommands;

use crate::utils::redis_service::get_redis_connection;

const GEO_KEY_PREFIX: &str = "drivers:geo:";

fn geo_key_for_tenant(tenant_id: Uuid) -> String {
    format!("{GEO_KEY_PREFIX}{tenant_id}")
}

/// Upsert (add/update) driver location in Redis GEO set
pub async fn upsert_driver_location(
    tenant_id: Uuid,
    user_id: Uuid,
    lat: f64,
    lon: f64,
) -> anyhow::Result<()> {
    let mut conn = get_redis_connection().await?;
    let key = geo_key_for_tenant(tenant_id);
    let member = user_id.to_string();

    // GEOADD key lon lat member
    let _: () = redis::cmd("GEOADD")
        .arg(&key)
        .arg(lon)
        .arg(lat)
        .arg(member)
        .query_async(&mut conn)
        .await?;

    Ok(())
}

/// Remove driver location from Redis GEO set
pub async fn remove_driver_location(
    tenant_id: Uuid,
    user_id: Uuid,
) -> anyhow::Result<()> {
    let mut conn = get_redis_connection().await?;
    let key = geo_key_for_tenant(tenant_id);
    let member = user_id.to_string();

    // ZREM key member
    let _: () = redis::cmd("ZREM")
        .arg(&key)
        .arg(member)
        .query_async(&mut conn)
        .await?;

    Ok(())
}

/// Query nearby driver user_ids via Redis GEOSEARCH (radius in KM)
pub async fn nearby_driver_ids(
    tenant_id: Uuid,
    lat: f64,
    lon: f64,
    radius_km: f64,
    max_results: usize,
) -> anyhow::Result<Vec<Uuid>> {
    let mut conn = get_redis_connection().await?;
    let key = geo_key_for_tenant(tenant_id);

    // GEOSEARCH key FROMLONLAT lon lat BYRADIUS radius km ASC COUNT N
    let raw: Vec<String> = redis::cmd("GEOSEARCH")
        .arg(&key)
        .arg("FROMLONLAT")
        .arg(lon)
        .arg(lat)
        .arg("BYRADIUS")
        .arg(radius_km)
        .arg("km")
        .arg("ASC")
        .arg("COUNT")
        .arg(max_results)
        .query_async(&mut conn)
        .await?;

    let ids = raw
        .into_iter()
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();

    Ok(ids)
}
