// /Users/xsm/Documents/workspace/xtras/daw/src/handlers/tenant_handler.rs
use actix_web::{web, HttpRequest, HttpResponse};
use sea_orm::DatabaseConnection;

use crate::dto::tenant::CreateTenantRequest;
use crate::services::tenant_service::create_tenant_service;

pub async fn create_tenant_handler(
    req: HttpRequest,
    body: web::Json<CreateTenantRequest>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, actix_web::Error> {
    create_tenant_service(req, body, db).await
}
