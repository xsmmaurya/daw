// /Users/xsm/Documents/workspace/xtras/daw/src/routes/tenant.rs
use actix_web::web;
use sea_orm::DatabaseConnection;
use crate::handlers::tenant_handler;

pub fn configure_tenant_routes(cfg: &mut web::ServiceConfig, db: web::Data<DatabaseConnection>) {
    cfg.service(
        web::scope("/tenants")
            .app_data(db.clone())
            .route("", web::post().to(tenant_handler::create_tenant_handler)),
    );
}
