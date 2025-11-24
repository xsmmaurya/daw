// /Users/xsm/Documents/workspace/xtras/daw/src/routes/tenant.rs
use actix_web::web;
use sea_orm::DatabaseConnection;
use crate::handlers::tenant_handler;
use crate::middleware::auth_middleware::authenticate;
use actix_web_httpauth::middleware::HttpAuthentication;


pub fn configure_tenant_routes(cfg: &mut web::ServiceConfig, db: web::Data<DatabaseConnection>) {
    let value = db.clone();
    let auth = HttpAuthentication::bearer(move |req, credentials| {
        let db_clone = value.clone();
        async move { authenticate(req, credentials, db_clone).await }
    });


    cfg.service(
        web::scope("/tenants")
            .wrap(auth)
            .app_data(db.clone())
            .route("", web::post().to(tenant_handler::create_tenant_handler)),
    );
}
