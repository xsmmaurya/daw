// /Users/xsm/Documents/workspace/xtras/daw/src/handlers/user_handler.rs
use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;

use crate::services::user_service::get_profile_service;

pub async fn me_handler(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, actix_web::Error> {
    get_profile_service(req, db).await
}
