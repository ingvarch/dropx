use serde::Serialize;
use worker::{Response, Result, RouteContext};
use crate::models::ErrorResponse;
use crate::utils::content_negotiation::ResponseFormat;
use crate::utils::constants::{
    HEADER_ACCEPT, HEADER_VARY,
    HTTP_BAD_REQUEST, HTTP_UNAUTHORIZED, HTTP_NOT_FOUND,
    ERR_FILE_NOT_FOUND, ERR_FILE_ID_REQUIRED,
};

/// Body of a plain text error: `Error: <message>` and a newline.
fn plain_error_body(message: &str) -> String {
    format!("Error: {message}\n")
}

pub fn error_response(error: &str, message: &str, status: u16) -> Result<Response> {
    Ok(Response::from_json(&ErrorResponse {
        error: error.to_string(),
        message: message.to_string(),
    })?
    .with_status(status))
}

fn with_vary_accept(mut response: Response) -> Result<Response> {
    response.headers_mut().set(HEADER_VARY, HEADER_ACCEPT)?;
    Ok(response)
}

/// Sends `json` as the body for JSON clients and `text` for the rest. Both carry `Vary: Accept`.
pub fn negotiated_response<T: Serialize>(format: ResponseFormat, json: &T, text: String) -> Result<Response> {
    let response = match format {
        ResponseFormat::Json => Response::from_json(json)?,
        ResponseFormat::Text => Response::ok(text)?,
    };
    with_vary_accept(response)
}

/// Sends `json` as the body for JSON clients and `Error: <message>` for the rest. Both carry `Vary: Accept`.
pub fn negotiated_error_with_body<T: Serialize>(format: ResponseFormat, json: &T, message: &str, status: u16) -> Result<Response> {
    Ok(negotiated_response(format, json, plain_error_body(message))?.with_status(status))
}

pub fn negotiated_error_response(format: ResponseFormat, error: &str, message: &str, status: u16) -> Result<Response> {
    let body = ErrorResponse { error: error.to_string(), message: message.to_string() };
    negotiated_error_with_body(format, &body, message, status)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_error_body_is_the_message_after_a_prefix_and_ends_with_a_newline() {
        assert_eq!(plain_error_body("Maximum file size is 25 MB"), "Error: Maximum file size is 25 MB\n");
    }
}
