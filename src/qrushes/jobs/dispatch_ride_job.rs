// src/qrushes/jobs/dispatch_ride_job.rs
use async_trait::async_trait;
use futures::future::BoxFuture;
use qrush::job::Job;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use serde_json::json;

use sea_orm::{
    EntityTrait,
    ColumnTrait,
    QueryFilter,
    QueryOrder,
    ActiveModelTrait,
    Set,
    DatabaseConnection,
};

use crate::config::AppConfig;
use crate::db::init_db;
use crate::entity::ride::{
    Entity as RideEntity,
    Column as RideColumn,
    ActiveModel as RideActiveModel,
};
use crate::entity::driver::{
    Entity as DriverEntity,
    Column as DriverColumn,
};
use crate::ws::notify_user;
use crate::entity::ride_event::ActiveModel as RideEventActiveModel;


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DispatchRideJob {
    pub ride_id: Uuid,
}

#[async_trait]
impl Job for DispatchRideJob {
    async fn perform(&self) -> Result<()> {
        let cfg = AppConfig::from_env()?;
        let db = init_db(&cfg.database.url).await?;

        dispatch_ride(&db, self.ride_id).await
    }

    fn name(&self) -> &'static str {
        "DispatchRideJob"
    }

    fn queue(&self) -> &'static str {
        "dispatch"
    }
}

impl DispatchRideJob {
    pub fn name() -> &'static str {
        "DispatchRideJob"
    }

    pub fn handler(payload: String) -> BoxFuture<'static, Result<Box<dyn Job>>> {
        Box::pin(async move {
            let job: DispatchRideJob = serde_json::from_str(&payload)?;
            Ok(Box::new(job) as Box<dyn Job>)
        })
    }
}

async fn dispatch_ride(db: &DatabaseConnection, ride_id: Uuid) -> Result<()> {
    // load the ride
    let ride = RideEntity::find()
        .filter(RideColumn::Id.eq(ride_id))
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Ride not found: {}", ride_id))?;

    if ride.status != "requested" {
        println!(
            "Ride {} not in requested state ({}), skipping",
            ride_id, ride.status
        );
        return Ok(());
    }

    // find online driver (for now: any online driver of any tenant)
    let driver = DriverEntity::find()
        .filter(DriverColumn::IsOnline.eq(true))
        .order_by_asc(DriverColumn::UpdatedAt)
        .one(db)
        .await?;

    let Some(driver) = driver else {
        println!("No online driver for tenant {}", ride.tenant_id);
        return Ok(());
    };

    // IMPORTANT: ride.driver_id should store the DRIVER USER_ID (FK to user)
    let mut ride_am: RideActiveModel = ride.clone().into();
    ride_am.driver_id = Set(Some(driver.user_id));
    ride_am.status = Set("assigned".to_string());
    let updated = ride_am.update(db).await?;


    let ev = RideEventActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        tenant_id: Set(updated.tenant_id),
        ride_id: Set(updated.id),
        actor_user_id: Set(None), // system
        kind: Set("ride_assigned".to_string()),
        payload: Set(Some(json!({
            "driver_user_id": updated.driver_id,
        }))),
        ..Default::default()
    };
    let _ = ev.insert(db).await;


    println!(
        "Ride {} assigned to driver_user={} (driver_row={} tenant={})",
        updated.id, driver.user_id, driver.id, driver.tenant_id
    );

    // Build a richer payload used by FE for both rider & driver panels
    let common_payload = json!({
        "ride": {
            "id": updated.id,
            "tenant_id": updated.tenant_id,
            "rider_id": updated.rider_id,
            "driver_user_id": updated.driver_id,
            "status": updated.status,
            "pickup": {
                "lat": updated.pickup_lat,
                "lon": updated.pickup_lon,
                "address": updated.pickup_address,
            },
            "destination": {
                "lat": updated.dest_lat,
                "lon": updated.dest_lon,
                "address": updated.dest_address,
            },
            "tier": updated.tier,
            "payment_method_id": updated.payment_method_id,
        }
    });

    // 🔔 Notify rider: they see driver assigned
    let _ = notify_user(
        updated.rider_id,
        "ride_assigned",
        common_payload.clone(),
    )
    .await;

    // 🔔 Notify driver (user_id): they see "incoming ride request" to accept / reject
    if let Some(driver_user_id) = updated.driver_id {
        let _ = notify_user(
            driver_user_id,
            "ride_assigned_to_driver",
            common_payload,
        )
        .await;
    }

    Ok(())
}
