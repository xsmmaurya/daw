use actix_web::{web, HttpRequest, HttpResponse, Error, HttpMessage};
use sea_orm::{EntityTrait, ActiveModelTrait, ColumnTrait, QueryFilter, Set, DatabaseConnection};
use serde_json::json;
use uuid::Uuid;

use crate::entity::ride::ActiveModel as RideActiveModel;
use crate::entity::user::{Entity as UserEntity, Column as UserColumn};
use crate::requests::structures::RideRequestPayload;
use crate::requests::validation::validate_ride_request;
use crate::types::request_keys::CurrentUserId;

/// POST /rides/request
pub async fn request_ride_service(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    payload: web::Json<RideRequestPayload>,
) -> Result<HttpResponse, Error> {
    // 1) Get current user id from middleware
    let CurrentUserId(user_id) = req
        .extensions()
        .get::<CurrentUserId>()
        .cloned()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Missing user in context"))?;

    // 2) Load user to get primary tenant_id
    let user = UserEntity::find()
        .filter(UserColumn::Id.eq(user_id))
        .one(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let user = match user {
        Some(u) => u,
        None => return Err(actix_web::error::ErrorUnauthorized("User not found")),
    };

    let tenant_id = user
        .tenant_id
        .ok_or_else(|| actix_web::error::ErrorForbidden("User has no primary tenant"))?;

    // 3) Validate payload
    validate_ride_request(&payload)
        .map_err(|e| actix_web::error::ErrorUnprocessableEntity(e))?;

    let pickup = &payload.pickup;
    let dest = &payload.destination;

    // 4) Insert ride
    let mut am = RideActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        rider_id: Set(user_id),
        driver_id: Set(None),
        pickup_lat: Set(pickup.lat),
        pickup_lon: Set(pickup.lon),
        pickup_address: Set(pickup.address.clone()),
        dest_lat: Set(dest.lat),
        dest_lon: Set(dest.lon),
        dest_address: Set(dest.address.clone()),
        tier: Set(payload.tier.clone()),
        payment_method_id: Set(payload.payment_method_id.clone()),
        status: Set("requested".to_string()),
        ..Default::default()
    };

    let ride = am
        .insert(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // 5) Response (later you’ll kick off dispatch here)
    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "code": 200,
        "message": "Ride requested",
        "data": {
            "ride_id": ride.id,
            "status": ride.status,
            "tenant_id": ride.tenant_id,
            "rider_id": ride.rider_id,
        }
    })))
}
