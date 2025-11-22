// /Users/xsm/Documents/workspace/xtras/daw/src/entity/mod.rs
pub mod tenant;
pub mod user;
pub mod ride;

pub mod prelude {
    pub use super::tenant::Entity as Tenant;
    pub use super::user::Entity as User;
    pub use super::ride::Entity as Ride;
}
