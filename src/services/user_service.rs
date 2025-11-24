// /Users/xsm/Documents/workspace/xtras/daw/src/services/user_service.rs
use actix_web::{web, HttpRequest, HttpResponse, Error, HttpMessage};
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, DatabaseConnection};
use serde_json::json;

use crate::entity::user::{Entity as UserEntity, Column as UserColumn};
use crate::jresponse::user_jresponse::user_minimal_datum;
use crate::types::request_keys::CurrentUserId;
use crate::utils::current_user::get_current_user;

/// GET /me
pub async fn get_profile_service(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let user = get_current_user(&req, db.get_ref()).await?;
    let data = user_minimal_datum(&user);

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "code": 200,
        "message": "Profile fetched successfully",
        "data": data
    })))
}
