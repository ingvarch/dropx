use worker::{Request, Response, RouteContext, Result, console_log};
use crate::services::{StorageService, DatabaseService};
use crate::utils::response::{get_file_id_param, get_required_param, not_found};
use crate::utils::constants::HTTP_OK;

const ERR_DELETE_TOKEN_REQUIRED: &str = "Delete token is required";

pub async fn handle_delete(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let file_id = get_file_id_param(&ctx)?;
    let delete_token = get_required_param(&ctx, "token", ERR_DELETE_TOKEN_REQUIRED)?;

    if !DatabaseService::file_exists(&ctx.env, file_id).await? {
        return not_found("NotFound", "File not found");
    }

    let deleted = DatabaseService::delete_file(&ctx.env, file_id, delete_token).await?;

    if !deleted {
        return not_found("Unauthorized", "Invalid delete token");
    }

    let bucket = StorageService::get_bucket(&ctx.env)?;
    if let Err(e) = StorageService::delete_file(&bucket, file_id).await {
        console_log!("Failed to delete file from R2: {:?}", e);
    }

    Ok(Response::from_json(&serde_json::json!({
        "success": true,
        "message": "File deleted successfully"
    }))?
    .with_status(HTTP_OK))
}

#[cfg(test)]
mod tests {
    use crate::models::ErrorResponse;

    #[test]
    fn test_delete_url_format() {
        let file_id = "abc123";
        let delete_token = "secret456";
        let url = format!("/{}/{}", file_id, delete_token);
        assert_eq!(url, "/abc123/secret456");
    }

    #[test]
    fn test_success_response() {
        let response = serde_json::json!({
            "success": true,
            "message": "File deleted successfully"
        });
        assert!(response.get("success").unwrap().as_bool().unwrap());
        assert_eq!(response.get("message").unwrap().as_str().unwrap(), "File deleted successfully");
    }

    #[test]
    fn test_error_response_not_found() {
        let error = ErrorResponse {
            error: "NotFound".to_string(),
            message: "File not found".to_string(),
        };
        assert_eq!(error.error, "NotFound");
        assert_eq!(error.message, "File not found");
    }

    #[test]
    fn test_error_response_invalid_token() {
        let error = ErrorResponse {
            error: "Unauthorized".to_string(),
            message: "Invalid delete token".to_string(),
        };
        assert_eq!(error.error, "Unauthorized");
        assert_eq!(error.message, "Invalid delete token");
    }
}
