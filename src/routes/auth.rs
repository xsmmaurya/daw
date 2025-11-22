// /Users/xsm/Documents/workspace/xtras/daw/src/routes/auth.rs
use actix_web::web;
use sea_orm::DatabaseConnection;

use crate::handlers::auth_handler;

pub fn configure_auth_routes(cfg: &mut web::ServiceConfig, db: web::Data<DatabaseConnection>) {
    cfg.service(
        web::scope("/auth")
            .app_data(db.clone())
            .route("/otp/send", web::post().to(auth_handler::send_otp_handler))
            .route("/otp/verify", web::post().to(auth_handler::verify_otp_handler)),
    );
}
