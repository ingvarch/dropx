use worker::{Request, Response, RouteContext, Result, Headers, console_log};
use crate::services::{StorageService, DatabaseService};
use crate::utils::response::{get_file_id_param, not_found, file_not_found};
use crate::utils::content_disposition::{get_content_disposition, DispositionType};
use crate::utils::constants::{
    HEADER_CONTENT_TYPE, HEADER_CONTENT_DISPOSITION,
    HEADER_CONTENT_LENGTH, HEADER_DOWNLOAD_COUNT, HEADER_EXPIRES_AT,
};

pub async fn handle_get(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let file_id = get_file_id_param(&ctx)?;

    let file_metadata = match DatabaseService::get_file(&ctx.env, file_id).await? {
        Some(metadata) => metadata,
        None => return file_not_found(),
    };

    let _ = DatabaseService::increment_download_count(&ctx.env, file_id).await;

    if let Some(max_downloads) = file_metadata.max_downloads {
        let new_download_count = file_metadata.download_count + 1;
        if new_download_count >= max_downloads {
            let bucket = StorageService::get_bucket(&ctx.env)?;
            if let Err(e) = StorageService::delete_file(&bucket, file_id).await {
                console_log!("Failed to delete file from R2 after max downloads: {:?}", e);
            }

            if let Err(e) = DatabaseService::delete_file(&ctx.env, file_id, &file_metadata.delete_token).await {
                console_log!("Failed to delete file from database after max downloads: {:?}", e);
            }
        }
    }

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
        &get_content_disposition(
            &file_metadata.original_filename,
            DispositionType::Attachment,
            &file_metadata.mime_type,
        ),
    )?;
    headers.set(HEADER_CONTENT_LENGTH, &file_metadata.size.to_string())?;
    headers.set(HEADER_DOWNLOAD_COUNT, &file_metadata.download_count.to_string())?;
    headers.set(HEADER_EXPIRES_AT, &file_metadata.expires_at.to_string())?;

    Ok(Response::from_bytes(file_data)?.with_headers(headers))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_disposition_always_attachment() {
        let disposition = get_content_disposition("image.png", DispositionType::Attachment, "image/png");
        assert_eq!(disposition, "attachment; filename=\"image.png\"");
    }

    #[test]
    fn test_content_disposition_attachment_for_pdf() {
        let disposition = get_content_disposition("document.pdf", DispositionType::Attachment, "application/pdf");
        assert_eq!(disposition, "attachment; filename=\"document.pdf\"");
    }

    #[test]
    fn test_content_disposition_attachment_for_zip() {
        let disposition = get_content_disposition("archive.zip", DispositionType::Attachment, "application/zip");
        assert_eq!(disposition, "attachment; filename=\"archive.zip\"");
    }
}
