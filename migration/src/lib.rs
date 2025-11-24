// migration/src/lib.rs
pub use sea_orm_migration::prelude::*;

mod m20251122_133745_create_tenants;
mod m20251122_133748_create_users;
mod m20251122_133751_create_rides;
mod m20251122_190000_create_drivers;
mod m20251123_000001_create_ride_events;
mod m20251123_000002_create_driver_events;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251122_133745_create_tenants::Migration),
            Box::new(m20251122_133748_create_users::Migration),
            Box::new(m20251122_133751_create_rides::Migration),
            Box::new(m20251122_190000_create_drivers::Migration),
            Box::new(m20251123_000001_create_ride_events::Migration),
            Box::new(m20251123_000002_create_driver_events::Migration),
        ]
    }
}
