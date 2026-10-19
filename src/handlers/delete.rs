use worker::{Request, Response, RouteContext, Result};
use crate::services::{remove_file, StorageService, DatabaseService};
use crate::utils::content_negotiation::ResponseFormat;
use crate::utils::response::{get_file_id_param, get_required_param, negotiated_error_response, negotiated_response};
use crate::utils::auth::{hash_secret, secrets_match};
use crate::utils::constants::{HEADER_ACCEPT, HTTP_NOT_FOUND, ERR_NOT_FOUND, ERR_UNAUTHORIZED, MSG_FILE_DELETED};

const ERR_DELETE_TOKEN_REQUIRED: &str = "Delete token is required";

pub async fn handle_delete(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let format = ResponseFormat::from_accept(req.headers().get(HEADER_ACCEPT)?.as_deref());
    let file_id = get_file_id_param(&ctx)?;
    let delete_token = get_required_param(&ctx, "token", ERR_DELETE_TOKEN_REQUIRED)?;

    let Some(stored_hash) = DatabaseService::get_delete_token_hash(&ctx.env, file_id).await? else {
        return negotiated_error_response(format, ERR_NOT_FOUND, "File not found", HTTP_NOT_FOUND);
    };

    if !secrets_match(&hash_secret(delete_token), &stored_hash) {
        return negotiated_error_response(format, ERR_UNAUTHORIZED, "Invalid delete token", HTTP_NOT_FOUND);
    }

    let bucket = StorageService::get_bucket(&ctx.env)?;
    remove_file(&ctx.env, &bucket, file_id).await?;

    let body = serde_json::json!({ "success": true, "message": MSG_FILE_DELETED });
    negotiated_response(format, &body, format!("{MSG_FILE_DELETED}\n"))
}
