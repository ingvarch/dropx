use worker::{Request, Response, RouteContext, Result, Headers, console_log};
use crate::services::{StorageService, DatabaseService};
use crate::utils::response::{get_file_id_param, not_found, file_not_found};
use crate::utils::content_disposition::{get_content_disposition, DispositionType};
use crate::utils::constants::{
    HEADER_CONTENT_TYPE, HEADER_CONTENT_DISPOSITION,
    HEADER_CONTENT_LENGTH, HEADER_DOWNLOAD_COUNT, HEADER_EXPIRES_AT,
};

pub async fn handle_download(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    serve_file(&ctx, DispositionType::Auto).await
}

pub async fn handle_get(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    serve_file(&ctx, DispositionType::Attachment).await
}

pub async fn handle_inline(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    serve_file(&ctx, DispositionType::Inline).await
}

async fn serve_file(ctx: &RouteContext<()>, disposition: DispositionType) -> Result<Response> {
    let file_id = get_file_id_param(ctx)?;

    let file_metadata = match DatabaseService::get_file(&ctx.env, file_id).await? {
        Some(metadata) => metadata,
        None => return file_not_found(),
    };

    let bucket = StorageService::get_bucket(&ctx.env)?;
    let file_body = match StorageService::open_file(&bucket, file_id).await? {
        Some(body) => body,
        None => {
            return not_found(
                "File not found in storage",
                "The file exists in the database but not in storage",
            );
        }
    };

    if let Err(e) = DatabaseService::increment_download_count(&ctx.env, file_id).await {
        console_log!("Failed to count download of {}: {:?}", file_id, e);
    }

    let headers = Headers::new();
    headers.set(HEADER_CONTENT_TYPE, &file_metadata.mime_type)?;
    headers.set(
        HEADER_CONTENT_DISPOSITION,
        &get_content_disposition(
            &file_metadata.original_filename,
            disposition,
            &file_metadata.mime_type,
        ),
    )?;
    headers.set(HEADER_CONTENT_LENGTH, &file_metadata.size.to_string())?;
    headers.set(HEADER_DOWNLOAD_COUNT, &file_metadata.download_count.to_string())?;
    headers.set(HEADER_EXPIRES_AT, &file_metadata.expires_at.to_string())?;

    Ok(Response::from_body(file_body)?.with_headers(headers))
}
