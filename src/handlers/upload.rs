use worker::{Request, Response, RouteContext, Result};
use crate::models::{FileMetadata, UploadResponse, FileSizeError};
use crate::services::{StorageService, DatabaseService, RateLimiterService};
use crate::utils::{generate_file_id, detect_mime_type, hash_identifier, current_timestamp};
use crate::utils::constants::{
    STANDARD_RETENTION_DAYS, SECONDS_PER_DAY, STANDARD_MAX_FILE_SIZE, BYTES_PER_MB,
    HEADER_API_KEY, HEADER_CF_CONNECTING_IP, HTTP_TOO_MANY_REQUESTS, HTTP_PAYLOAD_TOO_LARGE,
    ERR_RATE_LIMIT_EXCEEDED, ERR_INVALID_API_KEY, ERR_FILE_TOO_LARGE, UNKNOWN_IP_ADDRESS, DEFAULT_FILENAME,
};
use crate::utils::response::unauthorized;

pub async fn handle_upload(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let env = ctx.env;

    let api_key = req.headers().get(HEADER_API_KEY)?;

    let (user_id, tier) = if let Some(key) = &api_key {
        match DatabaseService::get_api_key(&env, key).await? {
            Some(api_key_data) if api_key_data.is_active => {
                let now = current_timestamp();
                let _ = DatabaseService::update_api_key_last_used(&env, key, now).await;
                (key.clone(), Some(api_key_data.tier))
            }
            _ => {
                return unauthorized(ERR_INVALID_API_KEY, "The provided API key is invalid or inactive");
            }
        }
    } else {
        let ip = req
            .headers()
            .get(HEADER_CF_CONNECTING_IP)?
            .unwrap_or_else(|| UNKNOWN_IP_ADDRESS.to_string());
        (hash_identifier(&ip), None)
    };

    if RateLimiterService::check_limit(&env, &user_id, tier).await.is_err() {
        return Response::error(ERR_RATE_LIMIT_EXCEEDED, HTTP_TOO_MANY_REQUESTS);
    }

    let body = req.bytes().await?;
    let size = body.len();

    let max_size = tier
        .map(|t| t.max_file_size())
        .unwrap_or(STANDARD_MAX_FILE_SIZE);

    if size > max_size {
        return Ok(Response::from_json(&FileSizeError {
            error: ERR_FILE_TOO_LARGE.to_string(),
            message: format!("Maximum file size is {} MB", max_size / BYTES_PER_MB),
            max_size,
        })?
        .with_status(HTTP_PAYLOAD_TOO_LARGE));
    }

    let file_id = generate_file_id();

    let path = req.path();
    let filename = if path == "/" || path.is_empty() {
        DEFAULT_FILENAME.to_string()
    } else {
        path.trim_start_matches('/').to_string()
    };

    let mime_type = detect_mime_type(&filename, &body);

    let bucket = StorageService::get_bucket(&env)?;
    StorageService::upload_file(&bucket, &file_id, body).await?;

    let now = current_timestamp();
    let retention_days = tier
        .map(|t| t.retention_days())
        .unwrap_or(STANDARD_RETENTION_DAYS);
    let expires_at = now + (retention_days * SECONDS_PER_DAY);

    let file_metadata = FileMetadata {
        id: file_id.clone(),
        original_filename: filename.clone(),
        size: size as i64,
        mime_type: mime_type.clone(),
        created_at: now,
        expires_at,
        download_count: 0,
        uploaded_by: user_id.to_string(),
    };

    DatabaseService::insert_file(&env, &file_metadata).await?;

    let url_obj = req.url()?;
    let host = url_obj.host_str().unwrap_or("localhost");
    let url = format!("https://{}/{}", host, file_id);

    let response = UploadResponse {
        url,
        id: file_id,
        filename,
        size,
        mime_type,
        expires_in: format!("{}d", retention_days),
        expires_at,
    };

    Response::from_json(&response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::api_key::ApiKeyTier;
    use crate::utils::constants::{
        PREMIUM_MAX_FILE_SIZE, PREMIUM_RETENTION_DAYS,
        STANDARD_MAX_FILE_SIZE, STANDARD_RETENTION_DAYS,
    };

    #[test]
    fn test_max_file_size_standard() {
        let tier = Some(ApiKeyTier::Standard);
        let max_size = tier.map(|t| t.max_file_size()).unwrap_or(STANDARD_MAX_FILE_SIZE);
        assert_eq!(max_size, 100 * 1024 * 1024);
    }

    #[test]
    fn test_max_file_size_premium() {
        let tier = Some(ApiKeyTier::Premium);
        let max_size = tier.map(|t| t.max_file_size()).unwrap_or(STANDARD_MAX_FILE_SIZE);
        assert_eq!(max_size, 500 * 1024 * 1024);
    }

    #[test]
    fn test_max_file_size_anon() {
        let tier: Option<ApiKeyTier> = None;
        let max_size = tier.map(|t| t.max_file_size()).unwrap_or(STANDARD_MAX_FILE_SIZE);
        assert_eq!(max_size, 100 * 1024 * 1024);
    }

    #[test]
    fn test_retention_days_standard() {
        let tier = Some(ApiKeyTier::Standard);
        let retention = tier.map(|t| t.retention_days()).unwrap_or(STANDARD_RETENTION_DAYS);
        assert_eq!(retention, 7);
    }

    #[test]
    fn test_retention_days_premium() {
        let tier = Some(ApiKeyTier::Premium);
        let retention = tier.map(|t| t.retention_days()).unwrap_or(STANDARD_RETENTION_DAYS);
        assert_eq!(retention, 30);
    }

    #[test]
    fn test_retention_days_anon() {
        let tier: Option<ApiKeyTier> = None;
        let retention = tier.map(|t| t.retention_days()).unwrap_or(STANDARD_RETENTION_DAYS);
        assert_eq!(retention, 7);
    }

    #[test]
    fn test_expires_at_calculation_standard() {
        let now: i64 = 1000000;
        let retention_days: i64 = 7;
        let expires_at = now + (retention_days * SECONDS_PER_DAY);
        assert_eq!(expires_at, 1000000 + 604800);
    }

    #[test]
    fn test_expires_at_calculation_premium() {
        let now: i64 = 1000000;
        let retention_days: i64 = 30;
        let expires_at = now + (retention_days * SECONDS_PER_DAY);
        assert_eq!(expires_at, 1000000 + 2592000);
    }

    #[test]
    fn test_filename_extraction_root() {
        let path = "/";
        let filename = if path == "/" || path.is_empty() {
            DEFAULT_FILENAME.to_string()
        } else {
            path.trim_start_matches('/').to_string()
        };
        assert_eq!(filename, "file");
    }

    #[test]
    fn test_filename_extraction_empty() {
        let path = "";
        let filename = if path == "/" || path.is_empty() {
            DEFAULT_FILENAME.to_string()
        } else {
            path.trim_start_matches('/').to_string()
        };
        assert_eq!(filename, "file");
    }

    #[test]
    fn test_filename_extraction_simple() {
        let path = "/document.pdf";
        let filename = if path == "/" || path.is_empty() {
            DEFAULT_FILENAME.to_string()
        } else {
            path.trim_start_matches('/').to_string()
        };
        assert_eq!(filename, "document.pdf");
    }

    #[test]
    fn test_filename_extraction_with_path() {
        let path = "/uploads/document.pdf";
        let filename = if path == "/" || path.is_empty() {
            DEFAULT_FILENAME.to_string()
        } else {
            path.trim_start_matches('/').to_string()
        };
        assert_eq!(filename, "uploads/document.pdf");
    }

    #[test]
    fn test_size_validation_under_limit() {
        let size = 50 * 1024 * 1024;
        let max_size = STANDARD_MAX_FILE_SIZE;
        assert!(size <= max_size);
    }

    #[test]
    fn test_size_validation_at_limit() {
        let size = 100 * 1024 * 1024;
        let max_size = STANDARD_MAX_FILE_SIZE;
        assert!(size <= max_size);
    }

    #[test]
    fn test_size_validation_over_limit() {
        let size = 150 * 1024 * 1024;
        let max_size = STANDARD_MAX_FILE_SIZE;
        assert!(size > max_size);
    }

    #[test]
    fn test_size_validation_premium_under_limit() {
        let size = 400 * 1024 * 1024;
        let max_size = PREMIUM_MAX_FILE_SIZE;
        assert!(size <= max_size);
    }

    #[test]
    fn test_size_validation_premium_at_limit() {
        let size = 500 * 1024 * 1024;
        let max_size = PREMIUM_MAX_FILE_SIZE;
        assert!(size <= max_size);
    }

    #[test]
    fn test_size_validation_premium_over_limit() {
        let size = 600 * 1024 * 1024;
        let max_size = PREMIUM_MAX_FILE_SIZE;
        assert!(size > max_size);
    }

    #[test]
    fn test_expires_in_format_standard() {
        let retention_days = STANDARD_RETENTION_DAYS;
        let expires_in = format!("{}d", retention_days);
        assert_eq!(expires_in, "7d");
    }

    #[test]
    fn test_expires_in_format_premium() {
        let retention_days = PREMIUM_RETENTION_DAYS;
        let expires_in = format!("{}d", retention_days);
        assert_eq!(expires_in, "30d");
    }

    #[test]
    fn test_url_format() {
        let host = "example.com";
        let file_id = "abc123";
        let url = format!("https://{}/{}", host, file_id);
        assert_eq!(url, "https://example.com/abc123");
    }
}
