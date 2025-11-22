// /Users/xsm/Documents/workspace/xtras/daw/src/jresponse/user_jresponse.rs
use crate::entity::user::Model as UserModel;
use serde_json::{json, Value};

pub fn user_minimal_datum(user: &UserModel) -> Value {
    json!({
        "id": user.id,
        "email": user.email,
        "phone_number": user.phone_number,
        "tenant_id": user.tenant_id,
        "deleted": user.deleted,
        "locked": user.locked,
        "created_at": user.created_at.to_string(),
        "updated_at": user.updated_at.to_string(),
    })
}
