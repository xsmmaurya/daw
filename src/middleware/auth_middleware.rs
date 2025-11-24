// /Users/xsm/Documents/workspace/xtras/daw/src/middleware/auth_middleware.rs
use actix_web::{dev::ServiceRequest, Error, Result, web, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, DatabaseConnection};
use crate::entity::prelude::User as UserEntity;
use crate::types::request_keys::CurrentUserId;
use crate::utils::jwt_util::decode_jwt_token;

/// JWT Bearer auth middleware
pub async fn authenticate(
    mut req: ServiceRequest,
    credentials: BearerAuth,
    db: web::Data<DatabaseConnection>,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();

    let claims = match decode_jwt_token(token) {
        Ok(c) => c,
        Err(e) => return Err((e, req)),
    };

    // Load user from DB
    let user = match UserEntity::find()
        .filter(crate::entity::user::Column::Id.eq(claims.sub))
        .one(db.get_ref())
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            let err = actix_web::error::ErrorUnauthorized("User not found");
            return Err((err, req));
        }
        Err(e) => {
            let err = actix_web::error::ErrorInternalServerError(e.to_string());
            return Err((err, req));
        }
    };

    if user.deleted || user.locked {
        let err = actix_web::error::ErrorForbidden("Account disabled");
        return Err((err, req));
    }

    

    // Inject CurrentUserId into request extensions
    req.extensions_mut().insert(CurrentUserId(user.id));

    Ok(req)
}
