// /Users/xsm/Documents/workspace/xtras/daw/src/services/tenant_service.rs
use actix_web::{web, HttpRequest, HttpResponse, Error, HttpMessage};
use sea_orm::{ActiveModelTrait, EntityTrait, Set, DatabaseConnection, ColumnTrait, QueryFilter};
use serde_json::json;
use uuid::Uuid;
use crate::dto::tenant::CreateTenantRequest;
use crate::entity::prelude::Tenant as TenantEntity;
use crate::entity::tenant::ActiveModel as TenantActiveModel;
use crate::entity::user::{Entity as UserEntity, ActiveModel as UserActiveModel, Column as UserColumn};
use crate::jresponse::tenant_jresponse::tenant_datum;
use crate::types::request_keys::CurrentUserId;

/// POST /tenants
pub async fn create_tenant_service(
    req: HttpRequest,
    body: web::Json<CreateTenantRequest>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let CurrentUserId(user_id) = req
        .extensions()
        .get::<CurrentUserId>()
        .cloned()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Missing user in context"))?;

    // Create tenant
    let mut am = TenantActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(body.name.clone()),
        slug: Set(body.slug.clone()),
        ..Default::default()
    };

    let tenant = am
        .insert(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // Set this tenant as user's primary tenant
    if let Some(mut user) = UserEntity::find()
        .filter(UserColumn::Id.eq(user_id))
        .one(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?
    {
        let mut user_am: UserActiveModel = user.into();
        user_am.tenant_id = Set(Some(tenant.id));
        user_am
            .update(db.get_ref())
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    }

    let j_tenant = tenant_datum(&tenant);

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "code": 200,
        "message": "Tenant created",
        "data": j_tenant
    })))
}
