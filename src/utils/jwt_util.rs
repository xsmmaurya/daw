// /Users/xsm/Documents/workspace/xtras/daw/src/utils/jwt_util.rs
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use std::env;
use actix_web::Error;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct JWTAuthClaims {
    pub sub: Uuid,   // user id
    pub exp: usize,  // expiry timestamp
}

fn jwt_secret() -> String {
    env::var("JWT_SECRET").expect("JWT_SECRET must be set")
}

pub fn generate_jwt_token(user_id: Uuid, ttl_seconds: usize) -> Result<String, Error> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = JWTAuthClaims {
        sub: user_id,
        exp: now + ttl_seconds,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(token)
}

pub fn decode_jwt_token(token: &str) -> Result<JWTAuthClaims, Error> {
    let data = decode::<JWTAuthClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid or expired token"))?;

    Ok(data.claims)
}
