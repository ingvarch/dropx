use worker::{Request, Response, RouteContext, Result};
use crate::models::{FileMetadata, UploadResponse, FileSizeError};
use crate::services::{StorageService, DatabaseService, RateLimiterService, RateLimitCheck};
use crate::utils::{generate_file_id, generate_delete_token, detect_mime_type, hash_identifier, current_timestamp};
use crate::utils::constants::{
    ANON_RETENTION_DAYS, ANON_MAX_FILE_SIZE,
    SECONDS_PER_DAY, BYTES_PER_MB,
    HEADER_API_KEY, HEADER_RETRY_AFTER, HEADER_CF_CONNECTING_IP, HEADER_MAX_DOWNLOADS, HEADER_MAX_AGE_DAYS,
    HTTP_TOO_MANY_REQUESTS, HTTP_PAYLOAD_TOO_LARGE,
    ERR_RATE_LIMIT_EXCEEDED, ERR_INVALID_API_KEY, ERR_FILE_TOO_LARGE, ERR_INVALID_RETENTION,
    UNKNOWN_IP_ADDRESS, DEFAULT_FILENAME,
};
use crate::utils::response::{bad_request, error_response, unauthorized};

#[derive(Debug, PartialEq)]
struct UploadOptions {
    retention_days: i64,
    max_downloads: Option<i32>,
}

impl UploadOptions {
    /// Values that are not integers are ignored. An out-of-range retention returns the error message.
    fn parse(
        max_age_days: Option<&str>,
        max_downloads: Option<&str>,
        default_retention_days: i64,
    ) -> std::result::Result<Self, String> {
        let retention_days = match max_age_days.and_then(|v| v.parse::<i64>().ok()) {
            Some(requested) if requested <= 0 => {
                return Err("Retention period must be at least 1 day".to_string());
            }
            Some(requested) if requested > default_retention_days => {
                return Err(format!(
                    "Retention period cannot exceed {} days for your tier",
                    default_retention_days
                ));
            }
            Some(requested) => requested,
            None => default_retention_days,
        };
        let max_downloads = max_downloads
            .and_then(|v| v.parse::<i32>().ok())
            .filter(|&v| v > 0);
        Ok(Self { retention_days, max_downloads })
    }
}

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

    let default_retention_days = tier
        .map(|t| t.retention_days())
        .unwrap_or(ANON_RETENTION_DAYS);
    let UploadOptions { retention_days, max_downloads } = match UploadOptions::parse(
        req.headers().get(HEADER_MAX_AGE_DAYS)?.as_deref(),
        req.headers().get(HEADER_MAX_DOWNLOADS)?.as_deref(),
        default_retention_days,
    ) {
        Ok(options) => options,
        Err(message) => return bad_request(ERR_INVALID_RETENTION, &message),
    };

    if let RateLimitCheck::Limited { retry_after, message } =
        RateLimiterService::check_limit(&env, &user_id, tier).await?
    {
        let mut response = error_response(ERR_RATE_LIMIT_EXCEEDED, &message, HTTP_TOO_MANY_REQUESTS)?;
        response.headers_mut().set(HEADER_RETRY_AFTER, &retry_after.to_string())?;
        return Ok(response);
    }

    let body = req.bytes().await?;
    let size = body.len();

    let max_size = tier
        .map(|t| t.max_file_size())
        .unwrap_or(ANON_MAX_FILE_SIZE);

    if size > max_size {
        return Ok(Response::from_json(&FileSizeError {
            error: ERR_FILE_TOO_LARGE.to_string(),
            message: format!("Maximum file size is {} MB", max_size / BYTES_PER_MB),
            max_size,
        })?
        .with_status(HTTP_PAYLOAD_TOO_LARGE));
    }

    let file_id = generate_file_id();
    let delete_token = generate_delete_token();

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
        delete_token: delete_token.clone(),
        max_downloads,
    };

    DatabaseService::insert_file(&env, &file_metadata).await?;

    let url_obj = req.url()?;
    let host = url_obj.host_str().unwrap_or("localhost");
    let url = format!("https://{}/{}", host, file_id);
    let delete_url = format!("https://{}/{}/{}", host, file_id, delete_token);

    let response = UploadResponse {
        url,
        id: file_id,
        filename,
        size,
        mime_type,
        expires_in: format!("{}d", retention_days),
        expires_at,
        delete_url: delete_url.clone(),
    };

    let mut resp = Response::from_json(&response)?;
    resp.headers_mut().set("X-Url-Delete", &delete_url)?;
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::api_key::ApiKeyTier;
    use crate::utils::constants::{
        ANON_MAX_FILE_SIZE, ANON_RETENTION_DAYS,
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
        let max_size = tier.map(|t| t.max_file_size()).unwrap_or(ANON_MAX_FILE_SIZE);
        assert_eq!(max_size, 25 * 1024 * 1024);
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
        let retention = tier.map(|t| t.retention_days()).unwrap_or(ANON_RETENTION_DAYS);
        assert_eq!(retention, 3);
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
    fn test_size_validation_anon_under_limit() {
        let size = 20 * 1024 * 1024;
        let max_size = ANON_MAX_FILE_SIZE;
        assert!(size <= max_size);
    }

    #[test]
    fn test_size_validation_anon_at_limit() {
        let size = 25 * 1024 * 1024;
        let max_size = ANON_MAX_FILE_SIZE;
        assert!(size <= max_size);
    }

    #[test]
    fn test_size_validation_anon_over_limit() {
        let size = 30 * 1024 * 1024;
        let max_size = ANON_MAX_FILE_SIZE;
        assert!(size > max_size);
    }

    #[test]
    fn test_size_validation_standard_under_limit() {
        let size = 50 * 1024 * 1024;
        let max_size = STANDARD_MAX_FILE_SIZE;
        assert!(size <= max_size);
    }

    #[test]
    fn test_size_validation_standard_at_limit() {
        let size = 100 * 1024 * 1024;
        let max_size = STANDARD_MAX_FILE_SIZE;
        assert!(size <= max_size);
    }

    #[test]
    fn test_size_validation_standard_over_limit() {
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
    fn test_expires_in_format_anon() {
        let retention_days = ANON_RETENTION_DAYS;
        let expires_in = format!("{}d", retention_days);
        assert_eq!(expires_in, "3d");
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

    #[test]
    fn test_max_downloads_one_time_link() {
        let max_downloads = Some(1);
        assert_eq!(max_downloads, Some(1));
    }

    #[test]
    fn test_max_downloads_unlimited() {
        let max_downloads: Option<i32> = None;
        assert_eq!(max_downloads, None);
    }

    #[test]
    fn test_max_downloads_large_value() {
        let max_downloads = Some(1000000);
        assert_eq!(max_downloads, Some(1000000));
    }

    #[test]
    fn test_max_age_days_anon_within_limit() {
        let requested = 1;
        let max_allowed = ANON_RETENTION_DAYS;
        assert!(requested > 0);
        assert!(requested <= max_allowed);
    }

    #[test]
    fn test_max_age_days_anon_at_limit() {
        let requested = 3;
        let max_allowed = ANON_RETENTION_DAYS;
        assert!(requested > 0);
        assert!(requested <= max_allowed);
    }

    #[test]
    fn test_max_age_days_anon_exceeds_limit() {
        let requested = 5;
        let max_allowed = ANON_RETENTION_DAYS;
        assert!(requested > max_allowed);
    }

    #[test]
    fn test_max_age_days_standard_within_limit() {
        let requested = 5;
        let max_allowed = STANDARD_RETENTION_DAYS;
        assert!(requested > 0);
        assert!(requested <= max_allowed);
    }

    #[test]
    fn test_max_age_days_standard_exceeds_limit() {
        let requested = 10;
        let max_allowed = STANDARD_RETENTION_DAYS;
        assert!(requested > max_allowed);
    }

    #[test]
    fn test_max_age_days_premium_within_limit() {
        let requested = 15;
        let max_allowed = PREMIUM_RETENTION_DAYS;
        assert!(requested > 0);
        assert!(requested <= max_allowed);
    }

    #[test]
    fn test_max_age_days_premium_exceeds_limit() {
        let requested = 50;
        let max_allowed = PREMIUM_RETENTION_DAYS;
        assert!(requested > max_allowed);
    }

    #[test]
    fn test_max_age_days_with_max_downloads() {
        let max_downloads = Some(5);
        let retention_days: i64 = 1;
        assert_eq!(max_downloads, Some(5));
        assert_eq!(retention_days, 1);
    }

    #[test]
    fn test_expires_at_with_custom_retention() {
        let now: i64 = 1000000;
        let retention_days: i64 = 1;
        let expires_at = now + (retention_days * SECONDS_PER_DAY);
        assert_eq!(expires_at, 1000000 + 86400);
    }

    fn options(max_age: Option<&str>, max_downloads: Option<&str>, default_days: i64) -> Result<UploadOptions, String> {
        UploadOptions::parse(max_age, max_downloads, default_days)
    }

    #[test]
    fn options_use_tier_default_when_headers_are_missing() {
        let parsed = options(None, None, 7).unwrap();
        assert_eq!(parsed, UploadOptions { retention_days: 7, max_downloads: None });
    }

    #[test]
    fn options_accept_retention_up_to_the_tier_default() {
        assert_eq!(options(Some("3"), None, 3).unwrap().retention_days, 3);
        assert_eq!(options(Some("1"), None, 3).unwrap().retention_days, 1);
    }

    #[test]
    fn options_ignore_non_integer_retention() {
        assert_eq!(options(Some("soon"), None, 7).unwrap().retention_days, 7);
    }

    #[test]
    fn options_reject_zero_and_negative_retention() {
        for value in ["0", "-1"] {
            assert_eq!(options(Some(value), None, 7), Err("Retention period must be at least 1 day".to_string()));
        }
    }

    #[test]
    fn options_reject_retention_above_the_tier_default() {
        assert_eq!(
            options(Some("8"), None, 7),
            Err("Retention period cannot exceed 7 days for your tier".to_string())
        );
    }

    #[test]
    fn options_parse_max_downloads() {
        assert_eq!(options(None, Some("5"), 7).unwrap().max_downloads, Some(5));
    }

    #[test]
    fn options_ignore_max_downloads_that_is_not_a_positive_integer() {
        for value in ["0", "-1", "many"] {
            assert_eq!(options(None, Some(value), 7).unwrap().max_downloads, None);
        }
    }
}
