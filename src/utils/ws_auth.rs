use actix_web::{Error, HttpRequest};
use uuid::Uuid;
use crate::utils::jwt_util::{
    decode_jwt_token,
    JWTAuthClaims,
};

/// Extract `token` from query string and validate JWT.
/// Returns the claims (or just user_id if you prefer).
pub fn extract_ws_claims(req: &HttpRequest) -> Result<JWTAuthClaims, Error> {
    // 1) Extract token from query ?token=JWT
    let token = req
        .query_string()
        .split('&')
        .find_map(|kv| {
            let mut parts = kv.splitn(2, '=');
            let key = parts.next()?;
            let val = parts.next()?;
            (key == "token").then(|| val.to_string())
        })
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Missing token"))?;

    // 2) Decode + validate JWT (signature, expiry, etc.)
    let claims = decode_jwt_token(&token)
        .map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    // 3) (Optional) extra check: sub must be a UUID if decode_jwt_token doesn’t enforce it
    // If claims.sub is String:
    let user_id = Uuid::parse_str(&claims.sub.to_string())
        .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid sub in token"))?;


    Ok(claims)
}

/// Convenience if you only care about user_id (Uuid) in the WS session
pub fn validate_and_extract(req: &HttpRequest) -> Result<Uuid, Error> {
    let claims = extract_ws_claims(req)?;
    Ok(claims.sub) // or the parsed Uuid if sub is String
}
