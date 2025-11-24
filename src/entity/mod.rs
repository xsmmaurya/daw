// /Users/xsm/Documents/workspace/xtras/daw/src/entity/mod.rs
pub mod tenant;
pub mod user;
pub mod ride;
pub mod driver;
pub mod ride_event;
pub mod driver_event;

pub mod prelude {
    pub use super::tenant::Entity as Tenant;
    pub use super::user::Entity as User;
    pub use super::ride::Entity as Ride;
    pub use super::driver::Entity as Driver;
    pub use super::ride_event::Entity as RideEvent;
    pub use super::driver_event::Entity as DriverEvent;
}
