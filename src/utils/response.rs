use worker::{Response, Result, RouteContext};
use crate::models::ErrorResponse;
use crate::utils::constants::{
    HTTP_BAD_REQUEST, HTTP_UNAUTHORIZED, HTTP_NOT_FOUND,
    ERR_FILE_NOT_FOUND, ERR_FILE_ID_REQUIRED,
};

pub fn error_response(error: &str, message: &str, status: u16) -> Result<Response> {
    Ok(Response::from_json(&ErrorResponse {
        error: error.to_string(),
        message: message.to_string(),
    })?
    .with_status(status))
}

pub fn bad_request(error: &str, message: &str) -> Result<Response> {
    error_response(error, message, HTTP_BAD_REQUEST)
}

pub fn unauthorized(error: &str, message: &str) -> Result<Response> {
    error_response(error, message, HTTP_UNAUTHORIZED)
}

pub fn not_found(error: &str, message: &str) -> Result<Response> {
    error_response(error, message, HTTP_NOT_FOUND)
}

pub fn file_not_found() -> Result<Response> {
    not_found(
        ERR_FILE_NOT_FOUND,
        "The requested file does not exist or has expired",
    )
}

pub fn get_file_id_param(ctx: &RouteContext<()>) -> Result<&str> {
    get_required_param(ctx, "id", ERR_FILE_ID_REQUIRED)
}

pub fn get_required_param<'a>(
    ctx: &'a RouteContext<()>,
    param: &str,
    error_msg: &str,
) -> Result<&'a str> {
    ctx.param(param)
        .map(|s| s.as_str())
        .ok_or_else(|| {
            worker::Error::RustError(format!("Missing parameter: {}", error_msg))
        })
}
