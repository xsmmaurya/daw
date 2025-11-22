// /Users/xsm/Documents/workspace/xtras/daw/src/error.rs
use actix_web::{HttpResponse, ResponseError};
use actix_web::http::header;
use serde::Serialize;
use thiserror::Error;
use serde_json::json;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("validation error")]
    Validation {
        field: String,
        message: String,
    },

    #[error("not found")]
    NotFound(String),

    #[error("db error")]
    Db(#[from] sea_orm::DbErr),

    #[error("internal server error")]
    Internal(String),

    #[error("rate limit exceeded")]
    RateLimited {
        retry_after_seconds: i64,
    },
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Validation { field, message } => {
                let body = ErrorResponse {
                    code: "VALIDATION_ERROR".into(),
                    message: message.clone(),
                    details: Some(json!({ "field": field })),
                };
                HttpResponse::UnprocessableEntity().json(body)
            }
            AppError::NotFound(msg) => {
                let body = ErrorResponse {
                    code: "NOT_FOUND".into(),
                    message: msg.clone(),
                    details: None,
                };
                HttpResponse::NotFound().json(body)
            }
            AppError::Db(e) => {
                let body = ErrorResponse {
                    code: "DB_ERROR".into(),
                    message: e.to_string(),
                    details: None,
                };
                HttpResponse::InternalServerError().json(body)
            }
            AppError::Internal(msg) => {
                let body = ErrorResponse {
                    code: "INTERNAL_ERROR".into(),
                    message: msg.clone(),
                    details: None,
                };
                HttpResponse::InternalServerError().json(body)
            }
            AppError::RateLimited { retry_after_seconds } => {
                let body = ErrorResponse {
                    code: "RATE_LIMITED".into(),
                    message: "Too many requests".into(),
                    details: Some(json!({
                        "retry_after_seconds": retry_after_seconds
                    })),
                };

                HttpResponse::TooManyRequests()
                    .insert_header((header::RETRY_AFTER, retry_after_seconds.to_string()))
                    .json(body)
            }
        }
    }
}
