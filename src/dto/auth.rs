// /Users/xsm/Documents/workspace/xtras/daw/src/dto/auth.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OTPRequest {
    pub identifier: String, // email or phone (we’ll treat as email here)
}

#[derive(Debug, Deserialize)]
pub struct VerifyOTP {
    pub identifier: String,
    pub otp: String,
}
