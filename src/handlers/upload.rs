use futures_util::StreamExt;
use worker::{console_log, Bucket, Data, Request, Response, RouteContext, Result};
use crate::models::{Limits, FileMetadata, UploadResponse, FileSizeError};
use crate::services::{StorageService, DatabaseService, RateLimiterService, RateLimitCheck};
use crate::utils::{generate_file_id, generate_delete_token, detect_mime_type, client_id, hash_secret, current_timestamp, origin_for_host};
use crate::utils::constants::{
    SECONDS_PER_DAY, BYTES_PER_MB,
    HEADER_API_KEY, HEADER_CONTENT_LENGTH, HEADER_HOST, HEADER_RETRY_AFTER, HEADER_CF_CONNECTING_IP, HEADER_MAX_DOWNLOADS, HEADER_MAX_AGE_DAYS,
    HTTP_TOO_MANY_REQUESTS, HTTP_PAYLOAD_TOO_LARGE, HTTP_LENGTH_REQUIRED,
    ERR_RATE_LIMIT_EXCEEDED, ERR_INVALID_API_KEY, ERR_FILE_TOO_LARGE, ERR_LENGTH_REQUIRED, ERR_INVALID_RETENTION, MIME_SNIFF_BYTES, MAX_BUFFERED_UPLOAD_SIZE,
    UNKNOWN_IP_ADDRESS, DEFAULT_FILENAME,
};
use crate::utils::response::{bad_request, error_response, unauthorized};
use crate::utils::upload_body::{body_plan, buffer_overflow, BodyPlan, BodyRejection};

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

fn filename_from_path(path: &str) -> String {
    match path.trim_start_matches('/') {
        "" => DEFAULT_FILENAME.to_string(),
        name => name.to_string(),
    }
}

fn too_large(max_size: usize) -> Result<Response> {
    Ok(Response::from_json(&FileSizeError {
        error: ERR_FILE_TOO_LARGE.to_string(),
        message: format!("Maximum file size is {} MB", max_size / BYTES_PER_MB),
        max_size,
    })?
    .with_status(HTTP_PAYLOAD_TOO_LARGE))
}

fn rejection_response(rejection: BodyRejection, max_size: usize) -> Result<Response> {
    match rejection {
        BodyRejection::TooLarge => too_large(max_size),
        BodyRejection::LengthRequired => error_response(
            ERR_LENGTH_REQUIRED,
            &format!(
                "Send a Content-Length header for files over {} MB",
                MAX_BUFFERED_UPLOAD_SIZE / BYTES_PER_MB
            ),
            HTTP_LENGTH_REQUIRED,
        ),
    }
}

async fn remove_orphan(bucket: &Bucket, file_id: &str) {
    if let Err(delete_error) = StorageService::delete_file(bucket, file_id).await {
        console_log!("Failed to delete orphaned object {}: {}", file_id, delete_error);
    }
}

enum Stored {
    Size(u64),
    Rejected(BodyRejection),
}

/// Writes the request body to R2. A known length goes straight from the request stream; otherwise the body is buffered up to the limit.
async fn store_body(req: &mut Request, plan: BodyPlan, max_size: usize, bucket: &Bucket, file_id: &str) -> Result<Stored> {
    let data: Data = match plan {
        BodyPlan::Stream => match req.inner().body() {
            Some(stream) => stream.into(),
            None => Vec::new().into(),
        },
        BodyPlan::Buffer => {
            let mut buffer = Vec::new();
            if req.inner().body().is_some() {
                let mut stream = req.stream()?;
                while let Some(chunk) = stream.next().await {
                    buffer.extend_from_slice(&chunk?);
                    if let Some(rejection) = buffer_overflow(buffer.len(), max_size) {
                        return Ok(Stored::Rejected(rejection));
                    }
                }
            }
            buffer.into()
        }
    };
    Ok(Stored::Size(StorageService::upload_file(bucket, file_id, data).await?))
}

pub async fn handle_upload(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let env = ctx.env;

    let api_key = req.headers().get(HEADER_API_KEY)?;

    let (user_id, tier) = if let Some(key) = &api_key {
        let key_hash = hash_secret(key);
        match DatabaseService::get_api_key(&env, &key_hash).await? {
            Some(api_key_data) if api_key_data.is_active => {
                let now = current_timestamp();
                let _ = DatabaseService::update_api_key_last_used(&env, &key_hash, now).await;
                (key_hash, Some(api_key_data.tier))
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
        (client_id(&env, &ip)?, None)
    };

    let limits = Limits::for_tier(tier);
    let UploadOptions { retention_days, max_downloads } = match UploadOptions::parse(
        req.headers().get(HEADER_MAX_AGE_DAYS)?.as_deref(),
        req.headers().get(HEADER_MAX_DOWNLOADS)?.as_deref(),
        limits.retention_days,
    ) {
        Ok(options) => options,
        Err(message) => return bad_request(ERR_INVALID_RETENTION, &message),
    };

    let max_size = limits.max_file_size;
    let plan = match body_plan(req.headers().get(HEADER_CONTENT_LENGTH)?.as_deref(), max_size) {
        Ok(plan) => plan,
        Err(rejection) => return rejection_response(rejection, max_size),
    };

    if let RateLimitCheck::Limited { retry_after, message } =
        RateLimiterService::check_limit(&env, &user_id, &limits).await?
    {
        let mut response = error_response(ERR_RATE_LIMIT_EXCEEDED, &message, HTTP_TOO_MANY_REQUESTS)?;
        response.headers_mut().set(HEADER_RETRY_AFTER, &retry_after.to_string())?;
        return Ok(response);
    }

    let file_id = generate_file_id();
    let delete_token = generate_delete_token();
    let filename = filename_from_path(&req.path());

    let bucket = StorageService::get_bucket(&env)?;
    let size = match store_body(&mut req, plan, max_size, &bucket, &file_id).await? {
        Stored::Size(size) => size as usize,
        Stored::Rejected(rejection) => return rejection_response(rejection, max_size),
    };

    let head = if size == 0 {
        Vec::new()
    } else {
        match StorageService::read_head(&bucket, &file_id, MIME_SNIFF_BYTES).await {
            Ok(head) => head,
            Err(error) => {
                remove_orphan(&bucket, &file_id).await;
                return Err(error);
            }
        }
    };
    let mime_type = detect_mime_type(&filename, &head);

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
        uploaded_by: user_id,
        delete_token_hash: hash_secret(&delete_token),
        max_downloads,
    };

    if let Err(insert_error) = DatabaseService::insert_file(&env, &file_metadata).await {
        remove_orphan(&bucket, &file_id).await;
        return Err(insert_error);
    }

    let origin = origin_for_host(req.headers().get(HEADER_HOST)?.as_deref());
    let url = format!("{}/{}", origin, file_id);
    let delete_url = format!("{}/{}/{}", origin, file_id, delete_token);

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

    #[test]
    fn root_path_gets_the_default_filename() {
        assert_eq!(filename_from_path("/"), DEFAULT_FILENAME);
        assert_eq!(filename_from_path(""), DEFAULT_FILENAME);
    }

    #[test]
    fn filename_is_the_path_without_the_leading_slash() {
        assert_eq!(filename_from_path("/document.pdf"), "document.pdf");
        assert_eq!(filename_from_path("/uploads/document.pdf"), "uploads/document.pdf");
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
