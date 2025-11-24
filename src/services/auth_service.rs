// /Users/xsm/Documents/workspace/xtras/daw/src/services/auth_service.rs
use actix_web::{web, HttpRequest, HttpResponse, Error, HttpMessage};
use redis::AsyncCommands;
use sea_orm::{EntityTrait, ActiveModelTrait, Set, ColumnTrait, QueryFilter, DatabaseConnection};
use serde_json::json;
use uuid::Uuid;
use crate::dto::auth::{OTPRequest, VerifyOTP};
use crate::entity::prelude::User as UserEntity;
use crate::entity::user::{ActiveModel as UserActiveModel, Column as UserColumn};
use crate::utils::redis_service::get_redis_connection;
use crate::utils::jwt_util::generate_jwt_token;
use crate::jresponse::user_jresponse::user_minimal_datum;

const OTP_TTL_SECONDS: usize = 300;
const JWT_TTL_SECONDS: usize = 60 * 60 * 24 * 7; // 7 days

fn norm(s: &str) -> String {
    s.trim().to_lowercase()
}

fn generate_otp() -> String {
    use rand::{thread_rng, Rng};
    (0..6)
        .map(|_| thread_rng().gen_range(0..10).to_string())
        .collect()
}

/// POST /auth/otp/send
pub async fn send_otp_service(
    _req: HttpRequest,
    body: web::Json<OTPRequest>,
) -> Result<HttpResponse, Error> {
    let identifier = norm(&body.identifier);

    let mut conn = get_redis_connection()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let otp = if identifier == "test@example.com" {
        "123456".to_string()
    } else {
        generate_otp()
    };

    let key = format!("otp:{}", identifier);

    let _: () = conn
        .set_ex(&key, &otp, (OTP_TTL_SECONDS as i64).try_into().unwrap())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    // In real world, send email / SMS here.

    println!("-----------------------------");
    println!("Generated OTP: {:?}", otp);
    println!("-----------------------------");
    Ok(HttpResponse::Ok().json(json!({
        "message": "OTP sent",
        "identifier": identifier
    })))
}

/// POST /auth/otp/verify
pub async fn verify_otp_service(
    _req: HttpRequest,
    body: web::Json<VerifyOTP>,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let identifier = norm(&body.identifier);
    let driver = &body.driver;

    let mut conn = get_redis_connection()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let key = format!("otp:{}", identifier);
    let stored: Option<String> = conn
        .get(&key)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if stored.is_none() {
        return Err(actix_web::error::ErrorForbidden(
            "OTP not found or expired",
        ));
    }

    if stored.unwrap() != body.otp {
        return Err(actix_web::error::ErrorForbidden("Invalid OTP"));
    }

    // Clean up OTP
    let _: () = conn.del(&key).await.unwrap_or(());

    // Find or create user by email
    let existing_user = UserEntity::find()
        .filter(UserColumn::Email.eq(identifier.clone()))
        .one(db.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let user = if let Some(u) = existing_user {
        if u.deleted {
            return Err(actix_web::error::ErrorForbidden("Account deleted"));
        }
        u
    } else {
        let mut am = UserActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            email: Set(identifier.clone()),
            driver: Set(driver.clone()),
            ..Default::default()
        };

        am.insert(db.get_ref())
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?
    };

    let token = generate_jwt_token(user.id, JWT_TTL_SECONDS)?;

    let j_user = user_minimal_datum(&user);

    Ok(HttpResponse::Ok().json(json!({
        "user": j_user,
        "token": token,
        "expires_in": JWT_TTL_SECONDS
    })))
}
