// src/handlers/event_handler.rs
use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;

use crate::services::event_service::{
    list_ride_events_service,
    list_driver_events_service,
    list_rider_events_service,
};

pub async fn ride_events_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let ride_id = path.into_inner();
    list_ride_events_service(req, db, ride_id).await
}

pub async fn driver_events_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let driver_id = path.into_inner();
    list_driver_events_service(req, db, driver_id).await
}

pub async fn rider_events_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let rider_id = path.into_inner();
    list_rider_events_service(req, db, rider_id).await
}
