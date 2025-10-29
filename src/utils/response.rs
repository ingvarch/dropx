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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response_structure() {
        let error = ErrorResponse {
            error: "TestError".to_string(),
            message: "Test message".to_string(),
        };

        assert_eq!(error.error, "TestError");
        assert_eq!(error.message, "Test message");
    }

    #[test]
    fn test_error_response_serialization() {
        let error = ErrorResponse {
            error: "NotFound".to_string(),
            message: "File not found".to_string(),
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"error\":\"NotFound\""));
        assert!(json.contains("\"message\":\"File not found\""));
    }

    #[test]
    fn test_error_response_with_special_chars() {
        let error = ErrorResponse {
            error: "ValidationError".to_string(),
            message: "Invalid filename: \"test<>file.txt\"".to_string(),
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("ValidationError"));
    }

    #[test]
    fn test_error_response_empty_strings() {
        let error = ErrorResponse {
            error: "".to_string(),
            message: "".to_string(),
        };

        assert_eq!(error.error, "");
        assert_eq!(error.message, "");
    }

    #[test]
    fn test_status_codes_are_correct() {
        assert_eq!(HTTP_BAD_REQUEST, 400);
        assert_eq!(HTTP_UNAUTHORIZED, 401);
        assert_eq!(HTTP_NOT_FOUND, 404);
    }

    #[test]
    fn test_error_constants() {
        assert_eq!(ERR_FILE_NOT_FOUND, "File not found");
        assert_eq!(ERR_FILE_ID_REQUIRED, "File ID is required");
    }

    #[test]
    fn test_file_not_found_message() {
        let expected_message = "The requested file does not exist or has expired";
        assert_eq!(expected_message.len(), 48);
        assert!(expected_message.contains("does not exist"));
        assert!(expected_message.contains("expired"));
    }

    #[test]
    fn test_missing_parameter_error_format() {
        let error_msg = "File ID is required";
        let formatted = format!("Missing parameter: {}", error_msg);
        assert_eq!(formatted, "Missing parameter: File ID is required");
    }

    #[test]
    fn test_error_message_construction() {
        let error = "NotFound";
        let message = "Resource does not exist";

        let err_response = ErrorResponse {
            error: error.to_string(),
            message: message.to_string(),
        };

        assert_eq!(err_response.error, "NotFound");
        assert_eq!(err_response.message, "Resource does not exist");
    }

    #[test]
    fn test_different_error_types() {
        let bad_request = ErrorResponse {
            error: "BadRequest".to_string(),
            message: "Invalid input".to_string(),
        };

        let unauthorized = ErrorResponse {
            error: "Unauthorized".to_string(),
            message: "Invalid credentials".to_string(),
        };

        let not_found = ErrorResponse {
            error: "NotFound".to_string(),
            message: "Resource not found".to_string(),
        };

        assert_ne!(bad_request.error, unauthorized.error);
        assert_ne!(unauthorized.error, not_found.error);
    }
}
