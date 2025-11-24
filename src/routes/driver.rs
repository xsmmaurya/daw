// /Users/xsm/Documents/workspace/xtras/daw/src/routes/driver.rs
use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;
use sea_orm::DatabaseConnection;

use crate::handlers::driver_handler;
use crate::middleware::auth_middleware::authenticate;

pub fn configure_driver_routes(cfg: &mut web::ServiceConfig, db: web::Data<DatabaseConnection>) {
    let value = db.clone();
    let auth = HttpAuthentication::bearer(move |req, credentials| {
        let db_clone = value.clone();
        async move { authenticate(req, credentials, db_clone).await }
    });

    cfg.service(
        web::scope("/drivers")
            .wrap(auth)
            .app_data(db.clone())
            .route("/online", web::post().to(driver_handler::driver_online_handler))
            .route("/offline", web::post().to(driver_handler::driver_offline_handler))
            .route("/location", web::post().to(driver_handler::driver_location_handler)),
    );
}
