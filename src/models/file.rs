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
    pub delete_url: Option<String>,
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
