use worker::{Request, Response, RouteContext, Result, Headers};
use crate::services::{StorageService, DatabaseService};
use crate::utils::response::{get_required_param, not_found};

pub async fn handle_download(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let file_id = get_required_param(&ctx, "id", "File ID is required")?;

    let file_metadata = match DatabaseService::get_file(&ctx.env, file_id).await? {
        Some(metadata) => metadata,
        None => {
            return not_found(
                "File not found",
                "The requested file does not exist or has expired",
            );
        }
    };

    let _ = DatabaseService::increment_download_count(&ctx.env, file_id).await;

    let bucket = ctx.env.bucket("TRANSFER_BUCKET")?;
    let file_data = match StorageService::download_file(&bucket, file_id).await? {
        Some(data) => data,
        None => {
            return not_found(
                "File not found in storage",
                "The file exists in the database but not in storage",
            );
        }
    };

    let headers = Headers::new();
    headers.set("Content-Type", &file_metadata.mime_type)?;
    headers.set(
        "Content-Disposition",
        &format!("attachment; filename=\"{}\"", file_metadata.original_filename),
    )?;
    headers.set("Content-Length", &file_metadata.size.to_string())?;
    headers.set("X-Download-Count", &file_metadata.download_count.to_string())?;
    headers.set("X-Expires-At", &file_metadata.expires_at.to_string())?;

    Ok(Response::from_bytes(file_data)?.with_headers(headers))
}
