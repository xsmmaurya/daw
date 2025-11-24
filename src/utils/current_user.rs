// src/utils/current_user.rs

use actix_web::{HttpRequest, Error, HttpMessage};
use actix_web::error::{ErrorUnauthorized, ErrorInternalServerError};
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter};

use crate::types::request_keys::CurrentUserId;
use crate::entity::user::{Entity as UserEntity, Column as UserColumn, Model as UserModel};

/// Extract current authenticated user (via middleware)
/// and fetch UserModel from DB.
pub async fn get_current_user(
    req: &HttpRequest,
    db: &DatabaseConnection,
) -> Result<UserModel, Error> {
    // 1. Extract auth context ID
    let CurrentUserId(user_id) = req
        .extensions()
        .get::<CurrentUserId>()
        .cloned()
        .ok_or_else(|| ErrorUnauthorized("UTILS: Missing user in context"))?;

    // 2. Load from DB
    let user = UserEntity::find()
        .filter(UserColumn::Id.eq(user_id))
        .one(db)
        .await
        .map_err(|e| ErrorInternalServerError(e.to_string()))?;

    let user = user.ok_or_else(|| ErrorUnauthorized("User not found"))?;

    // 3. Reject disabled accounts
    if user.deleted {
        return Err(ErrorUnauthorized("User account deleted"));
    }
    if user.locked {
        return Err(ErrorUnauthorized("User account locked"));
    }

    Ok(user)
}
