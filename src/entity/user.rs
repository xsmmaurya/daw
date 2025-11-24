// /Users/xsm/Documents/workspace/xtras/daw/src/entity/user.rs
use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,

    #[sea_orm(unique)]
    pub email: String,

    pub phone_number: Option<String>,

    /// Optional “primary” tenant relationship
    pub tenant_id: Option<Uuid>,

    #[sea_orm(default_value = false)]
    pub deleted: bool,

    #[sea_orm(default_value = false)]
    pub driver: bool,

    #[sea_orm(default_value = false)]
    pub locked: bool,

    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tenant::Entity",
        from = "Column::TenantId",
        to   = "super::tenant::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Tenant,
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
