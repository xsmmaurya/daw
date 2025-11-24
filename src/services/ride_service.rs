// /Users/xsm/Documents/workspace/xtras/daw/src/services/ride_service.rs
use actix_web::{web, HttpRequest, HttpResponse, Error};
use actix_web::error::ErrorForbidden;
use sea_orm::{
    EntityTrait, ActiveModelTrait, ColumnTrait, QueryFilter, Set, DatabaseConnection, QueryOrder,
    QuerySelect,
};
use serde_json::json;
use uuid::Uuid;

use crate::entity::ride::{
    Entity as RideEntity,
    Column as RideColumn,
    ActiveModel as RideActiveModel,
};

use crate::entity::user::{
    Model as UserModel,
};

use crate::requests::structures::{RideRequestPayload, RideListQuery};
use crate::requests::validation::validate_ride_request;
use crate::jresponse::ride_jresponse::ride_datum;
use qrush::queue::enqueue;
use crate::qrushes::jobs::dispatch_ride_job::DispatchRideJob;
use crate::utils::surge::{record_demand, current_multiplier};
use crate::utils::current_user::get_current_user;
use crate::ws::notify_user; // 🔔 WebSocket notifications
use crate::entity::ride_event::ActiveModel as RideEventActiveModel;

/// Simple Haversine distance in KM (for naive fare on completion)
fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0_f64; // Earth radius in km
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();

    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos()
            * lat2.to_radians().cos()
            * (dlon / 2.0).sin().powi(2);

    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}


fn ensure_rider(user: &UserModel) -> Result<(), Error> {
    if user.driver {
        return Err(ErrorForbidden("Not a rider account"));
    }
    Ok(())
}



/// POST /rides/request
pub async fn request_ride_service(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    payload: web::Json<RideRequestPayload>,
) -> Result<HttpResponse, Error> {
    let user = get_current_user(&req, db.get_ref()).await?;

    ensure_rider(&user);

    let tenant_id = user
        .tenant_id
        .ok_or_else(|| actix_web::error::ErrorForbidden("User has no primary tenant"))?;
    let user_id = user.id;

    validate_ride_request(&payload)
        .map_err(|e| actix_web::error::ErrorUnprocessableEntity(e))?;

    let pickup = &payload.pickup;
    let dest = &payload.destination;

    // 🔹 record demand & compute surge (best-effort)
    let _ = record_demand(tenant_id, pickup.lat, pickup.lon).await;
    let surge_multiplier = current_multiplier(tenant_id, pickup.lat, pickup.lon)
        .await
        .unwrap_or(1.0);

    let mut am = RideActiveModel {
        id: sea_orm::ActiveValue::NotSet,
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

    // record ride event: requested
    let ev = RideEventActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        tenant_id: Set(tenant_id),
        ride_id: Set(ride.id),
        actor_user_id: Set(Some(user_id)),
        kind: Set("ride_requested".to_string()),
        payload: Set(Some(json!({
            "pickup": {
                "lat": pickup.lat,
                "lon": pickup.lon,
                "address": pickup.address,
            },
            "destination": {
                "lat": dest.lat,
                "lon": dest.lon,
                "address": dest.address,
            },
            "tier": payload.tier,
            "payment_method_id": payload.payment_method_id,
            "surge_multiplier": surge_multiplier,
        }))),
        ..Default::default()
    };
    let _ = ev.insert(db.get_ref()).await;



    // 🔹 enqueue dispatch job
    if let Err(e) = enqueue(DispatchRideJob { ride_id: ride.id }).await {
        tracing::error!("Failed to enqueue DispatchRideJob for {}: {:?}", ride.id, e);
    } else {
        tracing::info!("Enqueued DispatchRideJob for {}", ride.id);
    }

    let data = ride_datum(&ride);

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "code": 200,
        "message": "Ride requested",
        "data": {
            "ride": data,
            "pricing": {
                "surge_multiplier": surge_multiplier
            }
        }
    })))
}

/// GET /rides/{id}
pub async fn get_ride_service(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    ride_id: Uuid,
) -> Result<HttpResponse, Error> {
    let user = get_current_user(&req, db.get_ref()).await?;

    ensure_rider(&user);

    let user_id = user.id;


    // Load ride
    let ride = RideEntity::find()
        .filter(RideColumn::Id.eq(ride_id))
        .one(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let ride = match ride {
        Some(r) => r,
        None => return Err(actix_web::error::ErrorNotFound("Ride not found")),
    };

    // Simple access control: only rider can view (later extend for drivers/admin)
    if ride.rider_id != user_id {
        return Err(actix_web::error::ErrorForbidden(
            "You are not allowed to access this ride",
        ));
    }

    let data = ride_datum(&ride);

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "code": 200,
        "message": "Ride details",
        "data": data
    })))
}

/// GET /rides
pub async fn list_rides_service(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    query: RideListQuery,
) -> Result<HttpResponse, Error> {
    let user = get_current_user(&req, db.get_ref()).await?;

    ensure_rider(&user);


    let user_id = user.id;

    let mut limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    if limit == 0 {
        limit = 20;
    }
    if limit > 100 {
        limit = 100;
    }

    let rides = RideEntity::find()
        .filter(RideColumn::RiderId.eq(user_id))
        .order_by_desc(RideColumn::CreatedAt)
        .limit(limit)
        .offset(offset)
        .all(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let items: Vec<_> = rides.iter().map(ride_datum).collect();

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "code": 200,
        "message": "Rides list",
        "data": {
            "rides": items,
            "limit": limit,
            "offset": offset
        }
    })))
}

/// POST /rides/{id}/accept
pub async fn accept_ride_service(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    ride_id: Uuid,
) -> Result<HttpResponse, Error> {
    let user = get_current_user(&req, db.get_ref()).await?;

    ensure_rider(&user);

    let user_id = user.id;

    let ride = RideEntity::find()
        .filter(RideColumn::Id.eq(ride_id))
        .one(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let ride = match ride {
        Some(r) => r,
        None => return Err(actix_web::error::ErrorNotFound("Ride not found")),
    };

    // driver_user must match assigned driver_id
    if ride.driver_id != Some(user_id) {
        return Err(actix_web::error::ErrorForbidden(
            "You are not the assigned driver for this ride",
        ));
    }

    if ride.status != "assigned" {
        return Err(actix_web::error::ErrorBadRequest(
            "Ride is not in 'assigned' status",
        ));
    }

    let mut ride_am: RideActiveModel = ride.into();
    ride_am.status = Set("accepted".to_string());

    let updated = ride_am
        .update(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let ev = RideEventActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        tenant_id: Set(updated.tenant_id),
        ride_id: Set(updated.id),
        actor_user_id: Set(Some(user_id)),
        kind: Set("ride_accepted".to_string()),
        payload: Set(Some(json!({
            "status": updated.status,
            "driver_user_id": updated.driver_id,
        }))),
        ..Default::default()
    };
    let _ = ev.insert(db.get_ref()).await;

    // 🔔 notify rider & driver
    let payload = json!({
        "ride_id": updated.id,
        "status": updated.status,
        "driver_user_id": updated.driver_id,
    });

    // rider
    let _ = notify_user(updated.rider_id, "ride_accepted", payload.clone()).await;
    // driver (you)
    let _ = notify_user(user_id, "ride_accepted_for_driver", payload.clone()).await;

    let data = ride_datum(&updated);

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "code": 200,
        "message": "Ride accepted",
        "data": data
    })))
}

/// POST /rides/{id}/reject
pub async fn reject_ride_service(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    ride_id: Uuid,
) -> Result<HttpResponse, Error> {
    let user = get_current_user(&req, db.get_ref()).await?;

    ensure_rider(&user);

    let user_id = user.id;

    let ride = RideEntity::find()
        .filter(RideColumn::Id.eq(ride_id))
        .one(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let ride = match ride {
        Some(r) => r,
        None => return Err(actix_web::error::ErrorNotFound("Ride not found")),
    };

    if ride.driver_id != Some(user_id) {
        return Err(actix_web::error::ErrorForbidden(
            "You are not the assigned driver for this ride",
        ));
    }

    if ride.status != "assigned" {
        return Err(actix_web::error::ErrorBadRequest(
            "Ride is not in 'assigned' status",
        ));
    }

    // Simple behaviour for now: clear driver_id, set back to "requested"
    let mut ride_am: RideActiveModel = ride.into();
    ride_am.driver_id = Set(None);
    ride_am.status = Set("requested".to_string());

    let updated = ride_am
        .update(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // 🔔 notify rider that driver rejected
    let payload = json!({
        "ride_id": updated.id,
        "status": updated.status,
    });

    let _ = notify_user(updated.rider_id, "ride_rejected_by_driver", payload.clone()).await;
    let _ = notify_user(user_id, "ride_rejected_for_driver", payload.clone()).await;

    // (Optionally enqueue again for re-dispatch later)

    let data = ride_datum(&updated);

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "code": 200,
        "message": "Ride rejected by driver",
        "data": data
    })))
}

/// POST /rides/{id}/start
pub async fn start_ride_service(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    ride_id: Uuid,
) -> Result<HttpResponse, Error> {
    let user = get_current_user(&req, db.get_ref()).await?;

    ensure_rider(&user);

    let user_id = user.id;

    let ride = RideEntity::find()
        .filter(RideColumn::Id.eq(ride_id))
        .one(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let ride = match ride {
        Some(r) => r,
        None => return Err(actix_web::error::ErrorNotFound("Ride not found")),
    };

    if ride.driver_id != Some(user_id) {
        return Err(actix_web::error::ErrorForbidden(
            "You are not the assigned driver for this ride",
        ));
    }

    if ride.status != "accepted" {
        return Err(actix_web::error::ErrorBadRequest(
            "Ride is not in 'accepted' status",
        ));
    }

    let mut ride_am: RideActiveModel = ride.into();
    ride_am.status = Set("in_progress".to_string());

    let updated = ride_am
        .update(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let payload = json!({
        "ride_id": updated.id,
        "status": updated.status,
    });

    let _ = notify_user(updated.rider_id, "ride_started", payload.clone()).await;
    let _ = notify_user(user_id, "ride_started_for_driver", payload.clone()).await;

    let data = ride_datum(&updated);

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "code": 200,
        "message": "Ride started",
        "data": data
    })))
}

/// POST /rides/{id}/complete
pub async fn complete_ride_service(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    ride_id: Uuid,
) -> Result<HttpResponse, Error> {
    let user = get_current_user(&req, db.get_ref()).await?;

    ensure_rider(&user);


    let user_id = user.id;

    let ride = RideEntity::find()
        .filter(RideColumn::Id.eq(ride_id))
        .one(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let ride = match ride {
        Some(r) => r,
        None => return Err(actix_web::error::ErrorNotFound("Ride not found")),
    };

    if ride.driver_id != Some(user_id) {
        return Err(actix_web::error::ErrorForbidden(
            "You are not the assigned driver for this ride",
        ));
    }

    if ride.status != "in_progress" {
        return Err(actix_web::error::ErrorBadRequest(
            "Ride is not in 'in_progress' status",
        ));
    }

    let mut ride_am: RideActiveModel = ride.into();
    ride_am.status = Set("completed".to_string());

    let updated = ride_am
        .update(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;


    // Naive fare calculation: straight-line distance * 20
    let distance_km = haversine_km(
        updated.pickup_lat,
        updated.pickup_lon,
        updated.dest_lat,
        updated.dest_lon,
    );
    let base_per_km = 20.0_f64;
    let fare_amount = (distance_km * base_per_km).round(); // e.g. ₹

    // record even
    let ev = RideEventActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        tenant_id: Set(updated.tenant_id),
        ride_id: Set(updated.id),
        actor_user_id: Set(Some(user_id)),
        kind: Set("ride_completed".to_string()),
        payload: Set(Some(json!({
            "status": updated.status,
            "distance_km": distance_km,
            "fare_amount": fare_amount,
        }))),
        ..Default::default()
    };
    let _ = ev.insert(db.get_ref()).await;

    let payload = json!({
        "ride_id": updated.id,
        "status": updated.status,
        "distance_km": distance_km,
        "fare_amount": fare_amount,
    });

    let _ = notify_user(updated.rider_id, "ride_completed", payload.clone()).await;
    let _ = notify_user(user_id, "ride_completed_for_driver", payload.clone()).await;

    let data = ride_datum(&updated);

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "code": 200,
        "message": "Ride completed",
        "data": data
    })))
}
