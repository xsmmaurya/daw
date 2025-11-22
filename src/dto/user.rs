// /Users/xsm/Documents/workspace/xtras/daw/src/dto/user.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UserProfileRequest {
    pub phone_number: Option<String>,
}
