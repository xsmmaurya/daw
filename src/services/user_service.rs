// /Users/xsm/Documents/workspace/xtras/daw/src/services/user_service.rs
use actix_web::{web, HttpRequest, HttpResponse, Error, HttpMessage};
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, DatabaseConnection};
use serde_json::json;

use crate::entity::user::{Entity as UserEntity, Column as UserColumn};
use crate::jresponse::user_jresponse::user_minimal_datum;
use crate::types::request_keys::CurrentUserId;

/// GET /me
pub async fn get_profile_service(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let CurrentUserId(user_id) = req
        .extensions()
        .get::<CurrentUserId>()
        .cloned()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Missing user in context"))?;

    let user = UserEntity::find()
        .filter(UserColumn::Id.eq(user_id))
        .one(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let user = match user {
        Some(u) => u,
        None => return Err(actix_web::error::ErrorNotFound("User not found")),
    };

    let data = user_minimal_datum(&user);

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "code": 200,
        "message": "Profile fetched successfully",
        "data": data
    })))
}
