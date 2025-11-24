// /Users/xsm/Documents/workspace/xtras/daw/src/services/driver_service.rs
use actix_web::{web, HttpRequest, HttpResponse, Error};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, TryIntoModel,
};
use serde_json::json;
use uuid::Uuid;
use actix_web::error::ErrorForbidden;
use crate::entity::driver::{
    ActiveModel as DriverActiveModel,
    Column as DriverColumn,
    Entity as DriverEntity,
    Model as DriverModel,
};
use crate::entity::user::{Model as UserModel};
use crate::requests::structures::DriverLocationPayload;
use crate::utils::current_user::get_current_user;
use crate::utils::redis_geo::{remove_driver_location, upsert_driver_location};
use crate::utils::surge::record_supply;
use crate::entity::driver_event::ActiveModel as DriverEventActiveModel;

/// Helper to load the driver model by (tenant_id, user_id)
async fn load_driver_model(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    user_id: Uuid,
) -> Result<DriverModel, Error> {
    DriverEntity::find()
        .filter(DriverColumn::TenantId.eq(tenant_id))
        .filter(DriverColumn::UserId.eq(user_id))
        .one(db)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Driver not found".to_string()))
}


fn ensure_driver(user: &UserModel) -> Result<(), Error> {
    if !user.driver {
        return Err(ErrorForbidden("Not a driver account"));
    }
    Ok(())
}

/// POST /drivers/online
pub async fn driver_go_online_service(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    payload: web::Json<DriverLocationPayload>,
) -> Result<HttpResponse, Error> {
    // unified: always go through current_user util
    let user = get_current_user(&req, db.get_ref()).await?;

    ensure_driver(&user);

    let tenant_id = user
        .tenant_id
        .ok_or_else(|| actix_web::error::ErrorForbidden("User has no primary tenant"))?;

    // check if driver exists
    let existing = DriverEntity::find()
        .filter(DriverColumn::TenantId.eq(tenant_id))
        .filter(DriverColumn::UserId.eq(user.id))
        .one(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let driver: DriverModel = if let Some(d) = existing {
        // UPDATE path
        let mut am: DriverActiveModel = d.into();
        am.is_online = Set(true);
        am.lat = Set(Some(payload.lat));
        am.lon = Set(Some(payload.lon));

        let updated_am = am
            .update(db.get_ref())
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

        updated_am
            .try_into_model()
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?
    } else {
        // INSERT path — let DB generate UUID via default
        let am = DriverActiveModel {
            id: sea_orm::ActiveValue::NotSet, // INSERT, not UPDATE
            tenant_id: Set(tenant_id),
            user_id: Set(user.id),
            is_online: Set(true),
            lat: Set(Some(payload.lat)),
            lon: Set(Some(payload.lon)),
            ..Default::default()
        };

        let inserted_am = am
            .insert(db.get_ref())
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

        inserted_am
    };

    // Record events
    let ev = DriverEventActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        tenant_id: Set(tenant_id),
        driver_id: Set(driver.id),
        actor_user_id: Set(Some(user.id)),
        kind: Set("driver_went_online".to_string()),
        payload: Set(Some(json!({
            "lat": payload.lat,
            "lon": payload.lon,
        }))),
        ..Default::default()
    };
    let _ = ev.insert(db.get_ref()).await;


    // best-effort surge supply update (must NOT cause 500)
    if let Err(e) = record_supply(tenant_id, payload.lat, payload.lon).await {
        tracing::warn!("record_supply failed for driver {}: {}", user.id, e);
    }

    // best-effort GEO upsert (no 500)
    if let Err(e) = upsert_driver_location(tenant_id, user.id, payload.lat, payload.lon).await {
        tracing::warn!("failed to upsert driver location in redis: {}", e);
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "code": 200,
        "message": "Driver is online",
        "data": {
            "driver_id": driver.id,
            "tenant_id": driver.tenant_id,
            "user_id": driver.user_id,
            "is_online": driver.is_online,
            "lat": driver.lat,
            "lon": driver.lon,
        }
    })))
}

/// POST /drivers/offline
pub async fn driver_go_offline_service(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let user = get_current_user(&req, db.get_ref()).await?;

    ensure_driver(&user);

    let tenant_id = user
        .tenant_id
        .ok_or_else(|| actix_web::error::ErrorForbidden("User has no primary tenant"))?;

    let existing = DriverEntity::find()
        .filter(DriverColumn::TenantId.eq(tenant_id))
        .filter(DriverColumn::UserId.eq(user.id))
        .one(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let Some(driver) = existing else {
        // no driver row yet → idempotent offline
        return Ok(HttpResponse::Ok().json(json!({
            "status": 200,
            "code": 200,
            "message": "Driver was not registered; treated as offline",
        })));
    };

    let mut driver_am: DriverActiveModel = driver.into();
    driver_am.is_online = Set(false);

    let updated_am = driver_am
        .update(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let driver: DriverModel = updated_am
        .try_into_model()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;


    // record event
    let ev = DriverEventActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        tenant_id: Set(tenant_id),
        driver_id: Set(driver.id),
        actor_user_id: Set(Some(user.id)),
        kind: Set("driver_went_offline".to_string()),
        payload: Set(None),
        ..Default::default()
    };
    let _ = ev.insert(db.get_ref()).await;


    // best-effort Redis GEO remove
    if let Err(e) = remove_driver_location(tenant_id, user.id).await {
        tracing::warn!("failed to remove driver location from redis: {}", e);
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "code": 200,
        "message": "Driver is offline",
        "data": {
            "driver_id": driver.id,
            "tenant_id": driver.tenant_id,
            "user_id": driver.user_id,
            "is_online": driver.is_online,
            "lat": driver.lat,
            "lon": driver.lon,
        }
    })))
}

/// POST /drivers/location
pub async fn driver_update_location_service(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
    payload: web::Json<DriverLocationPayload>,
) -> Result<HttpResponse, Error> {
    let user = get_current_user(&req, db.get_ref()).await?;

    ensure_driver(&user);

    let tenant_id = user
        .tenant_id
        .ok_or_else(|| actix_web::error::ErrorForbidden("User has no primary tenant"))?;

    let existing = DriverEntity::find()
        .filter(DriverColumn::TenantId.eq(tenant_id))
        .filter(DriverColumn::UserId.eq(user.id))
        .one(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let Some(driver) = existing else {
        return Err(actix_web::error::ErrorForbidden(
            "Driver not registered; call /drivers/online first",
        ));
    };

    if !driver.is_online {
        return Err(actix_web::error::ErrorForbidden(
            "Driver is offline; cannot update location",
        ));
    }

    let mut driver_am: DriverActiveModel = driver.into();
    driver_am.lat = Set(Some(payload.lat));
    driver_am.lon = Set(Some(payload.lon));

    let updated_am = driver_am
        .update(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let driver: DriverModel = updated_am
        .try_into_model()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // best-effort Redis GEO update
    if let Err(e) = upsert_driver_location(tenant_id, user.id, payload.lat, payload.lon).await {
        tracing::warn!("failed to upsert driver location in redis: {}", e);
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "code": 200,
        "message": "Driver location updated",
        "data": {
            "driver_id": driver.id,
            "tenant_id": driver.tenant_id,
            "user_id": driver.user_id,
            "is_online": driver.is_online,
            "lat": driver.lat,
            "lon": driver.lon,
        }
    })))
}
