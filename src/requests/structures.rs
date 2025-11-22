// /Users/xsm/Documents/workspace/xtras/daw/src/requests/structures.rs
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct CoordPayload {
    pub lat: f64,
    pub lon: f64,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RideRequestPayload {
    pub pickup: CoordPayload,
    pub destination: CoordPayload,

    #[validate(length(min = 1))]
    pub tier: String,

    #[validate(length(min = 1))]
    pub payment_method_id: String,

    pub rider_note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TenantContext {
    pub tenant_id: Uuid,
}
