// src/routes/event.rs
use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;
use sea_orm::DatabaseConnection;

use crate::handlers::event_handler;
use crate::middleware::auth_middleware::authenticate;

pub fn configure_events_routes(cfg: &mut web::ServiceConfig, db: web::Data<DatabaseConnection>) {
    let value = db.clone();

    let auth = HttpAuthentication::bearer(move |req, credentials| {
        let db_clone = value.clone();
        async move { authenticate(req, credentials, db_clone).await }
    });

    cfg.service(
        web::scope("/events")
            .wrap(auth)
            .app_data(db.clone())
            .route("/rides/{id}/events",    web::get().to(event_handler::ride_events_handler))
            .route("/drivers/{id}/events",  web::get().to(event_handler::driver_events_handler))
            .route("/riders/{id}/events",   web::get().to(event_handler::rider_events_handler)),
    );
}
