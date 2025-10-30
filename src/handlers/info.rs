use worker::{Request, Response, RouteContext, Result};
use crate::models::FileInfo;
use crate::services::DatabaseService;
use crate::utils::response::{get_file_id_param, file_not_found};

pub async fn handle_info(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let file_id = get_file_id_param(&ctx)?;

    let file_metadata = match DatabaseService::get_file(&ctx.env, file_id).await? {
        Some(metadata) => metadata,
        None => return file_not_found(),
    };

    let file_info: FileInfo = file_metadata.into();
    Response::from_json(&file_info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FileMetadata;

    #[test]
    fn test_file_info_conversion() {
        let metadata = FileMetadata {
            id: "test123".to_string(),
            original_filename: "document.pdf".to_string(),
            size: 1024,
            mime_type: "application/pdf".to_string(),
            created_at: 1000000,
            expires_at: 2000000,
            download_count: 5,
            uploaded_by: "user123".to_string(),
            delete_token: "secret123".to_string(),
            max_downloads: None,
        };

        let file_info: FileInfo = metadata.into();

        assert_eq!(file_info.id, "test123");
        assert_eq!(file_info.filename, "document.pdf");
        assert_eq!(file_info.size, 1024);
        assert_eq!(file_info.mime_type, "application/pdf");
        assert_eq!(file_info.created_at, 1000000);
        assert_eq!(file_info.expires_at, 2000000);
        assert_eq!(file_info.download_count, 5);
    }

    #[test]
    fn test_file_info_hides_uploaded_by() {
        let metadata = FileMetadata {
            id: "test".to_string(),
            original_filename: "file.txt".to_string(),
            size: 100,
            mime_type: "text/plain".to_string(),
            created_at: 1000,
            expires_at: 2000,
            download_count: 0,
            uploaded_by: "sensitive_user".to_string(),
            delete_token: "secret456".to_string(),
            max_downloads: None,
        };

        let file_info: FileInfo = metadata.into();
        let json = serde_json::to_string(&file_info).unwrap();

        assert!(!json.contains("uploaded_by"));
        assert!(!json.contains("sensitive_user"));
    }

    #[test]
    fn test_file_info_with_zero_downloads() {
        let metadata = FileMetadata {
            id: "new_file".to_string(),
            original_filename: "new.txt".to_string(),
            size: 512,
            mime_type: "text/plain".to_string(),
            created_at: 1000000,
            expires_at: 2000000,
            download_count: 0,
            uploaded_by: "user".to_string(),
            delete_token: "secret789".to_string(),
            max_downloads: None,
        };

        let file_info: FileInfo = metadata.into();
        assert_eq!(file_info.download_count, 0);
    }

    #[test]
    fn test_file_info_with_many_downloads() {
        let metadata = FileMetadata {
            id: "popular_file".to_string(),
            original_filename: "popular.pdf".to_string(),
            size: 2048,
            mime_type: "application/pdf".to_string(),
            created_at: 1000000,
            expires_at: 2000000,
            download_count: 9999,
            uploaded_by: "user".to_string(),
            delete_token: "secret111".to_string(),
            max_downloads: None,
        };

        let file_info: FileInfo = metadata.into();
        assert_eq!(file_info.download_count, 9999);
    }

    #[test]
    fn test_file_info_large_file_size() {
        let metadata = FileMetadata {
            id: "large".to_string(),
            original_filename: "large.zip".to_string(),
            size: 524_288_000,
            mime_type: "application/zip".to_string(),
            created_at: 1000000,
            expires_at: 2000000,
            download_count: 0,
            uploaded_by: "user".to_string(),
            delete_token: "secret222".to_string(),
            max_downloads: None,
        };

        let file_info: FileInfo = metadata.into();
        assert_eq!(file_info.size, 524_288_000);
    }

    #[test]
    fn test_file_info_timestamp_order() {
        let metadata = FileMetadata {
            id: "test".to_string(),
            original_filename: "test.txt".to_string(),
            size: 100,
            mime_type: "text/plain".to_string(),
            created_at: 1000000,
            expires_at: 2000000,
            download_count: 0,
            uploaded_by: "user".to_string(),
            delete_token: "secret333".to_string(),
            max_downloads: None,
        };

        let file_info: FileInfo = metadata.into();
        assert!(file_info.expires_at > file_info.created_at);
    }
}
