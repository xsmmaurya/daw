// src/services/tenant_service.rs
use actix_web::{web, HttpRequest, HttpResponse, Error};
use sea_orm::{ActiveModelTrait, EntityTrait, Set, DatabaseConnection};
use serde_json::json;
use uuid::Uuid;

use crate::dto::tenant::CreateTenantRequest;
use crate::entity::prelude::Tenant as TenantEntity;
use crate::entity::tenant::ActiveModel as TenantActiveModel;
use crate::entity::user::ActiveModel as UserActiveModel;
use crate::jresponse::tenant_jresponse::tenant_datum;
use crate::utils::current_user::get_current_user;

/// POST /tenants
pub async fn create_tenant_service(
    req: HttpRequest,
    body: web::Json<CreateTenantRequest>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    // ✅ Reusable helper: pulls CurrentUserId from extensions, loads user, and
    // rejects deleted/locked accounts.
    let user = get_current_user(&req, db.get_ref()).await?;

    // Create tenant with UUID PK
    let mut am = TenantActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        name: Set(body.name.clone()),
        slug: Set(body.slug.clone()),
        ..Default::default()
    };

    let tenant = am
        .insert(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // Set this tenant as user's primary tenant
    let mut user_am: UserActiveModel = user.into();
    user_am.tenant_id = Set(Some(tenant.id));

    user_am
        .update(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let j_tenant = tenant_datum(&tenant);

    Ok(HttpResponse::Ok().json(json!({
        "status": 200,
        "code": 200,
        "message": "Tenant created",
        "data": j_tenant
    })))
}
