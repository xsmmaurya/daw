// src/entity/driver_event.rs
use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "driver_event")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,

    pub tenant_id: Uuid,
    pub driver_id: Uuid,

    /// Usually the driver’s own user_id; null if system changed it.
    pub actor_user_id: Option<Uuid>,

    /// e.g. "driver_went_online", "driver_went_offline"
    pub kind: String,

    /// Extra JSON: lat/lon, reason, etc.
    pub payload: Option<Json>,

    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::driver::Entity",
        from = "Column::DriverId",
        to   = "super::driver::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Driver,

    #[sea_orm(
        belongs_to = "super::tenant::Entity",
        from = "Column::TenantId",
        to   = "super::tenant::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Tenant,
}

impl Related<super::driver::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Driver.def()
    }
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
