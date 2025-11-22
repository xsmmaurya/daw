// migration/src/lib.rs
pub use sea_orm_migration::prelude::*;

mod m20251122_133745_create_tenants;
mod m20251122_133748_create_users;
mod m20251122_133751_create_rides;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251122_133745_create_tenants::Migration),
            Box::new(m20251122_133748_create_users::Migration),
            Box::new(m20251122_133751_create_rides::Migration),
        ]
    }
}
