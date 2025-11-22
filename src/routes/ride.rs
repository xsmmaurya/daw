use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;
use sea_orm::DatabaseConnection;

use crate::handlers::ride_handler;
use crate::middleware::auth_middleware::authenticate;

pub fn configure_ride_routes(cfg: &mut web::ServiceConfig, db: web::Data<DatabaseConnection>) {
    let value = db.clone();
    let auth = HttpAuthentication::bearer(move |req, credentials| {
        let db_clone = value.clone();
        async move { authenticate(req, credentials, db_clone).await }
    });

    cfg.service(
        web::scope("/rides")
            .wrap(auth)
            .app_data(db.clone())
            .route("/request", web::post().to(ride_handler::request_ride_handler)),
    );
}
