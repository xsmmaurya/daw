// /Users/xsm/Documents/workspace/xtras/daw/src/handlers/auth_handler.rs
use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;

use crate::dto::auth::{OTPRequest, VerifyOTP};
use crate::services::auth_service::{send_otp_service, verify_otp_service};

pub async fn send_otp_handler(
    req: HttpRequest,
    body: web::Json<OTPRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    send_otp_service(req, body).await
}

pub async fn verify_otp_handler(
    req: HttpRequest,
    body: web::Json<VerifyOTP>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, actix_web::Error> {
    verify_otp_service(req, body, db).await
}
