use worker::{Request, Response, RouteContext, Result, console_log};
use crate::services::{StorageService, DatabaseService};
use crate::utils::response::{get_file_id_param, get_required_param, not_found};
use crate::utils::auth::hash_secret;
use crate::utils::constants::HTTP_OK;

const ERR_DELETE_TOKEN_REQUIRED: &str = "Delete token is required";

pub async fn handle_delete(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let file_id = get_file_id_param(&ctx)?;
    let delete_token = get_required_param(&ctx, "token", ERR_DELETE_TOKEN_REQUIRED)?;

    if !DatabaseService::file_exists(&ctx.env, file_id).await? {
        return not_found("NotFound", "File not found");
    }

    let deleted = DatabaseService::delete_file_with_token(&ctx.env, file_id, &hash_secret(delete_token)).await?;

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
