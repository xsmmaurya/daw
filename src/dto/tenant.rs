// /Users/xsm/Documents/workspace/xtras/daw/src/dto/tenant.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub slug: String,
}
