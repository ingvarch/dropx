use worker::{Request, Response, RouteContext, Result, Headers};
use crate::services::{StorageService, DatabaseService};
use crate::utils::response::{get_file_id_param, not_found, file_not_found};
use crate::utils::constants::{
    HEADER_CONTENT_TYPE, HEADER_CONTENT_DISPOSITION,
    HEADER_CONTENT_LENGTH, HEADER_DOWNLOAD_COUNT, HEADER_EXPIRES_AT,
};

pub async fn handle_download(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let file_id = get_file_id_param(&ctx)?;

    let file_metadata = match DatabaseService::get_file(&ctx.env, file_id).await? {
        Some(metadata) => metadata,
        None => return file_not_found(),
    };

    let _ = DatabaseService::increment_download_count(&ctx.env, file_id).await;

    let bucket = StorageService::get_bucket(&ctx.env)?;
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
    headers.set(HEADER_CONTENT_TYPE, &file_metadata.mime_type)?;
    headers.set(
        HEADER_CONTENT_DISPOSITION,
        &format!("attachment; filename=\"{}\"", file_metadata.original_filename),
    )?;
    headers.set(HEADER_CONTENT_LENGTH, &file_metadata.size.to_string())?;
    headers.set(HEADER_DOWNLOAD_COUNT, &file_metadata.download_count.to_string())?;
    headers.set(HEADER_EXPIRES_AT, &file_metadata.expires_at.to_string())?;

    Ok(Response::from_bytes(file_data)?.with_headers(headers))
}
