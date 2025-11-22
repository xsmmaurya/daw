// /Users/xsm/Documents/workspace/xtras/daw/src/routes/user.rs
use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;
use sea_orm::DatabaseConnection;

use crate::handlers::user_handler;
use crate::middleware::auth_middleware::authenticate;

pub fn configure_user_routes(cfg: &mut web::ServiceConfig, db: web::Data<DatabaseConnection>) {
    let value = db.clone();
    let auth = HttpAuthentication::bearer(move |req, credentials| {
        let db_clone = value.clone();
        async move { authenticate(req, credentials, db_clone).await }
    });

    cfg.service(
        web::scope("/users")
            .wrap(auth)
            .app_data(db.clone())
            .route("/me", web::get().to(user_handler::me_handler)),
    );
}
