// src/services/event_service.rs

use actix_web::{HttpResponse, HttpRequest, Error, web};
use sea_orm::{
    EntityTrait,
    ColumnTrait,
    QueryFilter,
    DatabaseConnection,
    QueryOrder,
    PaginatorTrait,
    QuerySelect,
};
use serde_json::json;
use uuid::Uuid;

use crate::utils::current_user::get_current_user;
use crate::utils::pagination::{get_pagination_params, set_pagination_headers};

use crate::entity::ride_event::{Entity as RideEventEntity, Column as RideEventColumn};
use crate::entity::driver_event::{Entity as DriverEventEntity, Column as DriverEventColumn};
use crate::entity::ride::{Entity as RideEntity, Column as RideColumn};
use crate::entity::driver::{Entity as DriverEntity, Column as DriverColumn};


/// ---------------------------------------------------------------------------
/// RIDE EVENTS (RIDER ONLY)
/// ---------------------------------------------------------------------------
pub async fn list_ride_events_service(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    ride_id_str: String,
) -> Result<HttpResponse, Error> {
    let user = get_current_user(&req, db.get_ref()).await?;

    let ride_id = Uuid::parse_str(&ride_id_str)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid ride_id"))?;

    // Validate access – must belong to this rider
    let ride = RideEntity::find()
        .filter(RideColumn::Id.eq(ride_id))
        .filter(RideColumn::RiderId.eq(user.id))
        .one(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if ride.is_none() {
        return Err(actix_web::error::ErrorForbidden("You cannot access this ride"));
    }

    let (page, limit, skip) = get_pagination_params(&req);

    // Count
    let total = RideEventEntity::find()
        .filter(RideEventColumn::RideId.eq(ride_id))
        .count(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?
        as i64;

    // Data
    let events = RideEventEntity::find()
        .filter(RideEventColumn::RideId.eq(ride_id))
        .order_by_asc(RideEventColumn::CreatedAt)
        .limit(limit as u64)
        .offset(skip as u64)
        .all(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let total_pages = ((total as f64) / (limit as f64)).ceil() as i64;

    let mut response = HttpResponse::Ok();
    set_pagination_headers(&mut response, total, total_pages, page, limit);

    Ok(response.json(json!({
        "status": 200,
        "message": "Ride events",
        "data": events
    })))
}



/// ---------------------------------------------------------------------------
/// DRIVER EVENTS (DRIVER USER ONLY)
/// ---------------------------------------------------------------------------
pub async fn list_driver_events_service(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    driver_id_str: String,
) -> Result<HttpResponse, Error> {
    let user = get_current_user(&req, db.get_ref()).await?;

    let driver_id = Uuid::parse_str(&driver_id_str)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid driver_id"))?;

    // Verify the driver belongs to this authenticated user
    let driver = DriverEntity::find()
        .filter(DriverColumn::Id.eq(driver_id))
        .filter(DriverColumn::UserId.eq(user.id))
        .one(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if driver.is_none() {
        return Err(actix_web::error::ErrorForbidden(
            "You cannot access this driver",
        ));
    }

    let (page, limit, skip) = get_pagination_params(&req);

    // Count
    let total = DriverEventEntity::find()
        .filter(DriverEventColumn::DriverId.eq(driver_id))
        .count(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?
        as i64;

    // Events
    let items = DriverEventEntity::find()
        .filter(DriverEventColumn::DriverId.eq(driver_id))
        .order_by_desc(DriverEventColumn::CreatedAt)
        .limit(limit as u64)
        .offset(skip as u64)
        .all(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let total_pages = ((total as f64) / (limit as f64)).ceil() as i64;

    let mut response = HttpResponse::Ok();
    set_pagination_headers(&mut response, total, total_pages, page, limit);

    Ok(response.json(json!({
        "status": 200,
        "message": "Driver events",
        "data": items
    })))
}



/// ---------------------------------------------------------------------------
/// RIDER EVENTS — FUTURE EXTENSION
/// ---------------------------------------------------------------------------
pub async fn list_rider_events_service(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    rider_id_str: String,
) -> Result<HttpResponse, Error> {
    let user = get_current_user(&req, db.get_ref()).await?;

    if user.id.to_string() != rider_id_str {
        return Err(actix_web::error::ErrorForbidden(
            "You cannot access another rider's events",
        ));
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "message": "Rider events not implemented yet",
        "data": []
    })))
}
