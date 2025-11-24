// /Users/xsm/Documents/workspace/xtras/daw/src/handlers/driver_handler.rs
use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;

use crate::requests::structures::DriverLocationPayload;
use crate::services::driver_service::{
    driver_go_online_service,
    driver_go_offline_service,
    driver_update_location_service,
};

pub async fn driver_online_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    payload: web::Json<DriverLocationPayload>,
) -> Result<HttpResponse, actix_web::Error> {
    driver_go_online_service(req, db, payload).await
}

pub async fn driver_offline_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, actix_web::Error> {
    driver_go_offline_service(req, db).await
}

pub async fn driver_location_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    payload: web::Json<DriverLocationPayload>,
) -> Result<HttpResponse, actix_web::Error> {
    driver_update_location_service(req, db, payload).await
}
