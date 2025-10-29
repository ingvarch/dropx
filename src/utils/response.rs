use worker::{Response, Result, RouteContext};
use crate::models::ErrorResponse;

pub fn error_response(error: &str, message: &str, status: u16) -> Result<Response> {
    Ok(Response::from_json(&ErrorResponse {
        error: error.to_string(),
        message: message.to_string(),
    })?
    .with_status(status))
}

pub fn bad_request(error: &str, message: &str) -> Result<Response> {
    error_response(error, message, 400)
}

pub fn unauthorized(error: &str, message: &str) -> Result<Response> {
    error_response(error, message, 401)
}

pub fn not_found(error: &str, message: &str) -> Result<Response> {
    error_response(error, message, 404)
}

pub fn file_not_found() -> Result<Response> {
    not_found(
        "File not found",
        "The requested file does not exist or has expired",
    )
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
