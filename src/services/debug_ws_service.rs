// src/services/debug_ws_service.rs

use actix_web::{HttpRequest, HttpResponse, Error, web};
use serde_json::json;
use sea_orm::DatabaseConnection;

use crate::utils::current_user::get_current_user;
use crate::ws::notify_user;

pub async fn test_ws_notify(
    req: HttpRequest,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let user = get_current_user(&req, db.get_ref()).await?;

    let payload = json!({
        "msg": "Hello from /debug/ws",
        "user_id": user.id,
    });

    let _ = notify_user(user.id, "debug_test", payload).await;

    Ok(HttpResponse::Ok().json({
        json!({"status": 200, "message": "sent debug ws"})
    }))
}
