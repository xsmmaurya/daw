// /Users/xsm/Documents/workspace/xtras/daw/src/jresponse/tenant_jresponse.rs
use crate::entity::tenant::Model as TenantModel;
use serde_json::{json, Value};

pub fn tenant_datum(tenant: &TenantModel) -> Value {
    json!({
        "id": tenant.id,
        "name": tenant.name,
        "slug": tenant.slug,
        "created_at": tenant.created_at.to_string(),
        "updated_at": tenant.updated_at.to_string(),
    })
}
