use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "driver")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,

    pub tenant_id: Uuid,
    pub user_id: Uuid,

    pub is_online: bool,

    pub lat: Option<f64>,
    pub lon: Option<f64>,

    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
