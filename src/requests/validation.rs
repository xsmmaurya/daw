// /Users/xsm/Documents/workspace/xtras/daw/src/requests/validation.rs
use crate::error::AppError;
use crate::requests::structures::RideRequestPayload;
use validator::Validate;

pub fn validate_ride_request(payload: &RideRequestPayload) -> Result<(), AppError> {
    if let Err(e) = payload.validate() {
        if let Some((field, errors)) = e.field_errors().iter().next() {
            let msg = errors[0]
                .message
                .clone()
                .unwrap_or_else(|| "invalid value".into());
            return Err(AppError::Validation {
                field: field.to_string(),
                message: msg.to_string(),
            });
        }

        return Err(AppError::Validation {
            field: "payload".into(),
            message: "invalid payload".into(),
        });
    }

    Ok(())
}
