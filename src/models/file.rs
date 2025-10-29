use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    pub original_filename: String,
    pub size: i64,
    pub mime_type: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub download_count: i32,
    pub uploaded_by: String,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub url: String,
    pub id: String,
    pub filename: String,
    pub size: usize,
    pub mime_type: String,
    pub expires_in: String,
    pub expires_at: i64,
}

#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub id: String,
    pub filename: String,
    pub size: i64,
    pub mime_type: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub download_count: i32,
}

impl From<FileMetadata> for FileInfo {
    fn from(metadata: FileMetadata) -> Self {
        Self {
            id: metadata.id,
            filename: metadata.original_filename,
            size: metadata.size,
            mime_type: metadata.mime_type,
            created_at: metadata.created_at,
            expires_at: metadata.expires_at,
            download_count: metadata.download_count,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct FileSizeError {
    pub error: String,
    pub message: String,
    pub max_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_metadata_serialization() {
        let metadata = FileMetadata {
            id: "abc123".to_string(),
            original_filename: "test.txt".to_string(),
            size: 1024,
            mime_type: "text/plain".to_string(),
            created_at: 1000000,
            expires_at: 2000000,
            download_count: 5,
            uploaded_by: "user123".to_string(),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"id\":\"abc123\""));
        assert!(json.contains("\"original_filename\":\"test.txt\""));
        assert!(json.contains("\"size\":1024"));
    }

    #[test]
    fn test_file_metadata_deserialization() {
        let json = r#"{"id":"abc123","original_filename":"test.txt","size":1024,"mime_type":"text/plain","created_at":1000000,"expires_at":2000000,"download_count":5,"uploaded_by":"user123"}"#;
        let metadata: FileMetadata = serde_json::from_str(json).unwrap();

        assert_eq!(metadata.id, "abc123");
        assert_eq!(metadata.original_filename, "test.txt");
        assert_eq!(metadata.size, 1024);
    }

    #[test]
    fn test_file_metadata_to_file_info_conversion() {
        let metadata = FileMetadata {
            id: "abc123".to_string(),
            original_filename: "document.pdf".to_string(),
            size: 2048,
            mime_type: "application/pdf".to_string(),
            created_at: 1000000,
            expires_at: 2000000,
            download_count: 10,
            uploaded_by: "user123".to_string(),
        };

        let file_info: FileInfo = metadata.into();

        assert_eq!(file_info.id, "abc123");
        assert_eq!(file_info.filename, "document.pdf");
        assert_eq!(file_info.size, 2048);
        assert_eq!(file_info.mime_type, "application/pdf");
        assert_eq!(file_info.created_at, 1000000);
        assert_eq!(file_info.expires_at, 2000000);
        assert_eq!(file_info.download_count, 10);
    }

    #[test]
    fn test_file_info_excludes_uploaded_by() {
        let metadata = FileMetadata {
            id: "test".to_string(),
            original_filename: "file.txt".to_string(),
            size: 100,
            mime_type: "text/plain".to_string(),
            created_at: 1000,
            expires_at: 2000,
            download_count: 0,
            uploaded_by: "sensitive_user".to_string(),
        };

        let file_info: FileInfo = metadata.into();
        let json = serde_json::to_string(&file_info).unwrap();

        assert!(!json.contains("uploaded_by"));
        assert!(!json.contains("sensitive_user"));
    }

    #[test]
    fn test_upload_response_serialization() {
        let response = UploadResponse {
            url: "https://example.com/abc123".to_string(),
            id: "abc123".to_string(),
            filename: "test.txt".to_string(),
            size: 1024,
            mime_type: "text/plain".to_string(),
            expires_in: "7d".to_string(),
            expires_at: 2000000,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"url\":\"https://example.com/abc123\""));
        assert!(json.contains("\"expires_in\":\"7d\""));
    }

    #[test]
    fn test_error_response_serialization() {
        let error = ErrorResponse {
            error: "NotFound".to_string(),
            message: "File not found".to_string(),
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"error\":\"NotFound\""));
        assert!(json.contains("\"message\":\"File not found\""));
    }

    #[test]
    fn test_file_size_error_serialization() {
        let error = FileSizeError {
            error: "FileTooLarge".to_string(),
            message: "File exceeds maximum size".to_string(),
            max_size: 104857600,
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"error\":\"FileTooLarge\""));
        assert!(json.contains("\"max_size\":104857600"));
    }
}
