// /Users/xsm/Documents/workspace/xtras/daw/src/jresponse/ride_jresponse.rs
use crate::entity::ride::Model as RideModel;
use serde_json::{json, Value};

pub fn ride_datum(ride: &RideModel) -> Value {
    json!({
        "id": ride.id,
        "tenant_id": ride.tenant_id,
        "rider_id": ride.rider_id,
        "driver_id": ride.driver_id,
        "pickup": {
            "lat": ride.pickup_lat,
            "lon": ride.pickup_lon,
            "address": ride.pickup_address,
        },
        "destination": {
            "lat": ride.dest_lat,
            "lon": ride.dest_lon,
            "address": ride.dest_address,
        },
        "tier": ride.tier,
        "payment_method_id": ride.payment_method_id,
        "status": ride.status,
        "created_at": ride.created_at.to_string(),
        "updated_at": ride.updated_at.to_string(),
    })
}
