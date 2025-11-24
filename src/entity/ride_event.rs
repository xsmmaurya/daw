// src/entity/ride_event.rs
use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "ride_event")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,

    pub tenant_id: Uuid,
    pub ride_id: Uuid,

    /// Who caused this event (rider/driver/system). Null for system jobs etc.
    pub actor_user_id: Option<Uuid>,

    /// e.g. "ride_requested", "ride_assigned", "ride_accepted",
    /// "ride_started", "ride_completed", "ride_rejected"
    pub kind: String,

    /// Arbitrary extra data as JSON (status, fare, distance, etc.)
    pub payload: Option<Json>,

    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::ride::Entity",
        from = "Column::RideId",
        to = "super::ride::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Ride,

    #[sea_orm(
        belongs_to = "super::tenant::Entity",
        from = "Column::TenantId",
        to   = "super::tenant::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Tenant,
}

impl Related<super::ride::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ride.def()
    }
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
