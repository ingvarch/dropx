use worker::{Request, Response, RouteContext, Result};
use crate::services::{remove_file, StorageService, DatabaseService};
use crate::utils::response::{get_file_id_param, get_required_param, not_found};
use crate::utils::auth::{hash_secret, secrets_match};
use crate::utils::constants::{HTTP_OK, ERR_NOT_FOUND, ERR_UNAUTHORIZED};

const ERR_DELETE_TOKEN_REQUIRED: &str = "Delete token is required";

pub async fn handle_delete(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let file_id = get_file_id_param(&ctx)?;
    let delete_token = get_required_param(&ctx, "token", ERR_DELETE_TOKEN_REQUIRED)?;

    let Some(stored_hash) = DatabaseService::get_delete_token_hash(&ctx.env, file_id).await? else {
        return not_found(ERR_NOT_FOUND, "File not found");
    };

    if !secrets_match(&hash_secret(delete_token), &stored_hash) {
        return not_found(ERR_UNAUTHORIZED, "Invalid delete token");
    }

    let bucket = StorageService::get_bucket(&ctx.env)?;
    remove_file(&ctx.env, &bucket, file_id).await?;

    Ok(Response::from_json(&serde_json::json!({
        "success": true,
        "message": "File deleted successfully"
    }))?
    .with_status(HTTP_OK))
}
