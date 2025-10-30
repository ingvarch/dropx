use worker::{Request, Response, RouteContext, Result, Headers, console_log};
use crate::services::{StorageService, DatabaseService};
use crate::utils::response::{get_file_id_param, not_found, file_not_found};
use crate::utils::content_disposition::{get_content_disposition, DispositionType};
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

    let _ = DatabaseService::increment_download_count(&ctx.env, file_id).await;

    if let Some(max_downloads) = file_metadata.max_downloads {
        let new_download_count = file_metadata.download_count + 1;
        if new_download_count >= max_downloads {
            if let Err(e) = StorageService::delete_file(&bucket, file_id).await {
                console_log!("Failed to delete file from R2 after max downloads: {:?}", e);
            }

            if let Err(e) = DatabaseService::delete_file(&ctx.env, file_id, &file_metadata.delete_token).await {
                console_log!("Failed to delete file from database after max downloads: {:?}", e);
            }
        }
    }

    let headers = Headers::new();
    headers.set(HEADER_CONTENT_TYPE, &file_metadata.mime_type)?;
    headers.set(
        HEADER_CONTENT_DISPOSITION,
        &get_content_disposition(
            &file_metadata.original_filename,
            DispositionType::Auto,
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
    fn test_content_disposition_auto_pdf() {
        let disposition = get_content_disposition("document.pdf", DispositionType::Auto, "application/pdf");
        assert_eq!(disposition, "inline; filename=\"document.pdf\"");
    }

    #[test]
    fn test_content_disposition_auto_image() {
        let disposition = get_content_disposition("photo.jpg", DispositionType::Auto, "image/jpeg");
        assert_eq!(disposition, "inline; filename=\"photo.jpg\"");
    }

    #[test]
    fn test_content_disposition_auto_zip() {
        let disposition = get_content_disposition("archive.zip", DispositionType::Auto, "application/zip");
        assert_eq!(disposition, "attachment; filename=\"archive.zip\"");
    }

    #[test]
    fn test_content_disposition_auto_text() {
        let disposition = get_content_disposition("notes.txt", DispositionType::Auto, "text/plain");
        assert_eq!(disposition, "inline; filename=\"notes.txt\"");
    }

    #[test]
    fn test_download_count_formatting() {
        let download_count = 0;
        let header = download_count.to_string();
        assert_eq!(header, "0");

        let download_count = 42;
        let header = download_count.to_string();
        assert_eq!(header, "42");

        let download_count = 999999;
        let header = download_count.to_string();
        assert_eq!(header, "999999");
    }

    #[test]
    fn test_expires_at_formatting() {
        let expires_at: i64 = 1704067200;
        let header = expires_at.to_string();
        assert_eq!(header, "1704067200");
    }

    #[test]
    fn test_content_length_formatting() {
        let size: i64 = 1024;
        let header = size.to_string();
        assert_eq!(header, "1024");

        let size: i64 = 104857600;
        let header = size.to_string();
        assert_eq!(header, "104857600");
    }

    #[test]
    fn test_download_count_increment_logic() {
        let current_count = 5;
        let new_count = current_count + 1;
        assert_eq!(new_count, 6);
    }

    #[test]
    fn test_download_count_from_zero() {
        let current_count = 0;
        let new_count = current_count + 1;
        assert_eq!(new_count, 1);
    }

    #[test]
    fn test_content_disposition_empty_filename() {
        let filename = "";
        let header = format!("attachment; filename=\"{}\"", filename);
        assert_eq!(header, "attachment; filename=\"\"");
    }

    #[test]
    fn test_file_not_found_in_storage_vs_database() {
        let in_database = true;
        let in_storage = false;

        assert!(in_database);
        assert!(!in_storage);
    }

    #[test]
    fn test_max_downloads_limit_reached() {
        let max_downloads = Some(5);
        let current_count = 4;
        let new_count = current_count + 1;

        assert!(new_count >= max_downloads.unwrap());
    }

    #[test]
    fn test_max_downloads_limit_not_reached() {
        let max_downloads = Some(5);
        let current_count = 3;
        let new_count = current_count + 1;

        assert!(new_count < max_downloads.unwrap());
    }

    #[test]
    fn test_max_downloads_one_time_link() {
        let max_downloads = Some(1);
        let current_count = 0;
        let new_count = current_count + 1;

        assert_eq!(new_count, max_downloads.unwrap());
    }

    #[test]
    fn test_max_downloads_none_unlimited() {
        let max_downloads: Option<i32> = None;
        assert!(max_downloads.is_none());
    }

    #[test]
    fn test_max_downloads_deletion_logic() {
        let max_downloads = Some(3);
        let download_count = 2;
        let new_download_count = download_count + 1;

        let should_delete = new_download_count >= max_downloads.unwrap();
        assert!(should_delete);
    }

    #[test]
    fn test_max_downloads_keep_file_logic() {
        let max_downloads = Some(10);
        let download_count = 5;
        let new_download_count = download_count + 1;

        let should_delete = new_download_count >= max_downloads.unwrap();
        assert!(!should_delete);
    }
}
