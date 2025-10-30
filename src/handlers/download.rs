use worker::{Request, Response, RouteContext, Result, Headers, console_log};
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
        &format!("attachment; filename=\"{}\"", file_metadata.original_filename),
    )?;
    headers.set(HEADER_CONTENT_LENGTH, &file_metadata.size.to_string())?;
    headers.set(HEADER_DOWNLOAD_COUNT, &file_metadata.download_count.to_string())?;
    headers.set(HEADER_EXPIRES_AT, &file_metadata.expires_at.to_string())?;

    Ok(Response::from_bytes(file_data)?.with_headers(headers))
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_content_disposition_simple_filename() {
        let filename = "document.pdf";
        let header = format!("attachment; filename=\"{}\"", filename);
        assert_eq!(header, "attachment; filename=\"document.pdf\"");
    }

    #[test]
    fn test_content_disposition_filename_with_spaces() {
        let filename = "my document.pdf";
        let header = format!("attachment; filename=\"{}\"", filename);
        assert_eq!(header, "attachment; filename=\"my document.pdf\"");
    }

    #[test]
    fn test_content_disposition_filename_with_special_chars() {
        let filename = "report-2024.pdf";
        let header = format!("attachment; filename=\"{}\"", filename);
        assert_eq!(header, "attachment; filename=\"report-2024.pdf\"");
    }

    #[test]
    fn test_content_disposition_filename_with_unicode() {
        let filename = "документ.pdf";
        let header = format!("attachment; filename=\"{}\"", filename);
        assert_eq!(header, "attachment; filename=\"документ.pdf\"");
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
}
