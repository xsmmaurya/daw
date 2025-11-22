use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;

use crate::requests::structures::RideRequestPayload;
use crate::services::ride_service::request_ride_service;

pub async fn request_ride_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    payload: web::Json<RideRequestPayload>,
) -> Result<HttpResponse, actix_web::Error> {
    request_ride_service(req, db, payload).await
}
