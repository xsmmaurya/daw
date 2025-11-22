use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "rides")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,

    pub tenant_id: Uuid,
    pub rider_id: Uuid,
    pub driver_id: Option<Uuid>,

    pub pickup_lat: f64,
    pub pickup_lon: f64,
    pub pickup_address: Option<String>,

    pub dest_lat: f64,
    pub dest_lon: f64,
    pub dest_address: Option<String>,

    pub tier: String,
    pub payment_method_id: String,
    pub status: String,

    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
