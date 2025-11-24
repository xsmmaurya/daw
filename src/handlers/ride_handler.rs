use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::requests::structures::{RideRequestPayload, RideListQuery};
use crate::services::ride_service::{
    request_ride_service,
    get_ride_service,
    list_rides_service,
    accept_ride_service,
    reject_ride_service,
    start_ride_service,
    complete_ride_service,
};

pub async fn request_ride_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    payload: web::Json<RideRequestPayload>,
) -> Result<HttpResponse, actix_web::Error> {
    request_ride_service(req, db, payload).await
}

pub async fn get_ride_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let ride_id = path.into_inner();
    get_ride_service(req, db, ride_id).await
}

pub async fn list_rides_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    query: web::Query<RideListQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    list_rides_service(req, db, query.into_inner()).await
}

pub async fn accept_ride_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let ride_id = path.into_inner();
    accept_ride_service(req, db, ride_id).await
}

pub async fn reject_ride_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let ride_id = path.into_inner();
    reject_ride_service(req, db, ride_id).await
}

pub async fn start_ride_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let ride_id = path.into_inner();
    start_ride_service(req, db, ride_id).await
}

pub async fn complete_ride_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let ride_id = path.into_inner();
    complete_ride_service(req, db, ride_id).await
}
