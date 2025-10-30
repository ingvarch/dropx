pub const BYTES_PER_KB: usize = 1024;
pub const BYTES_PER_MB: usize = 1024 * BYTES_PER_KB;

pub const STANDARD_MAX_FILE_SIZE: usize = 100 * BYTES_PER_MB;
pub const PREMIUM_MAX_FILE_SIZE: usize = 500 * BYTES_PER_MB;

pub const STANDARD_RETENTION_DAYS: i64 = 7;
pub const PREMIUM_RETENTION_DAYS: i64 = 30;

pub const MILLIS_PER_SECOND: u64 = 1000;
pub const MILLIS_PER_HOUR: u64 = 60 * 60 * MILLIS_PER_SECOND;
pub const MILLIS_PER_DAY: u64 = 24 * MILLIS_PER_HOUR;

pub const SECONDS_PER_DAY: i64 = 24 * 60 * 60;

pub const TRANSFER_BUCKET_NAME: &str = "TRANSFER_BUCKET";
pub const DB_NAME: &str = "DB";
pub const RATE_LIMITER_NAME: &str = "RATE_LIMITER";

pub const ANON_HOURLY_LIMIT: usize = 10;
pub const ANON_DAILY_LIMIT: usize = 50;
pub const STANDARD_HOURLY_LIMIT: usize = 100;
pub const STANDARD_DAILY_LIMIT: usize = 500;
pub const PREMIUM_HOURLY_LIMIT: usize = 1000;
pub const PREMIUM_DAILY_LIMIT: usize = 5000;

pub const HASH_TRUNCATE_BYTES: usize = 16;
pub const FILE_ID_UUID_BYTES: usize = 8;
pub const MIN_MAGIC_BYTES_LENGTH: usize = 4;

pub const HTTP_OK: u16 = 200;
pub const HTTP_CREATED: u16 = 201;
pub const HTTP_BAD_REQUEST: u16 = 400;
pub const HTTP_UNAUTHORIZED: u16 = 401;
pub const HTTP_NOT_FOUND: u16 = 404;
pub const HTTP_PAYLOAD_TOO_LARGE: u16 = 413;
pub const HTTP_TOO_MANY_REQUESTS: u16 = 429;

pub const HEADER_API_KEY: &str = "X-API-Key";
pub const HEADER_ADMIN_TOKEN: &str = "X-Admin-Token";
pub const HEADER_CF_CONNECTING_IP: &str = "CF-Connecting-IP";
pub const HEADER_CONTENT_TYPE: &str = "Content-Type";
pub const HEADER_CONTENT_DISPOSITION: &str = "Content-Disposition";
pub const HEADER_CONTENT_LENGTH: &str = "Content-Length";
pub const HEADER_RETRY_AFTER: &str = "Retry-After";
pub const HEADER_DOWNLOAD_COUNT: &str = "X-Download-Count";
pub const HEADER_EXPIRES_AT: &str = "X-Expires-At";
pub const HEADER_MAX_DOWNLOADS: &str = "Max-Downloads";

pub const DO_RATE_LIMITER_BASE_URL: &str = "http://rate-limiter";
pub const DO_RATE_LIMITER_PATH_CHECK: &str = "/check";
pub const DO_RATE_LIMITER_PATH_RESET: &str = "/reset";

pub const DB_BOOL_TRUE: i64 = 1;
pub const DB_BOOL_FALSE: i64 = 0;

pub const ERR_RATE_LIMIT_EXCEEDED: &str = "Rate limit exceeded";
pub const ERR_FILE_NOT_FOUND: &str = "File not found";
pub const ERR_INVALID_API_KEY: &str = "Invalid API key";
pub const ERR_FILE_TOO_LARGE: &str = "File too large";
pub const ERR_FILE_ID_REQUIRED: &str = "File ID is required";

pub const UNKNOWN_IP_ADDRESS: &str = "unknown";
pub const DEFAULT_FILENAME: &str = "file";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_conversion_constants() {
        assert_eq!(BYTES_PER_KB, 1024);
        assert_eq!(BYTES_PER_MB, 1024 * 1024);
        assert_eq!(BYTES_PER_MB, 1_048_576);
    }

    #[test]
    fn test_file_size_limits_correct_values() {
        assert_eq!(STANDARD_MAX_FILE_SIZE, 100 * 1024 * 1024);
        assert_eq!(STANDARD_MAX_FILE_SIZE, 104_857_600);

        assert_eq!(PREMIUM_MAX_FILE_SIZE, 500 * 1024 * 1024);
        assert_eq!(PREMIUM_MAX_FILE_SIZE, 524_288_000);
    }

    #[test]
    fn test_file_size_limits_relationship() {
        assert!(PREMIUM_MAX_FILE_SIZE > STANDARD_MAX_FILE_SIZE);
        assert_eq!(PREMIUM_MAX_FILE_SIZE, STANDARD_MAX_FILE_SIZE * 5);
    }

    #[test]
    fn test_retention_days_constants() {
        assert_eq!(STANDARD_RETENTION_DAYS, 7);
        assert_eq!(PREMIUM_RETENTION_DAYS, 30);
    }

    #[test]
    fn test_retention_days_relationship() {
        assert!(PREMIUM_RETENTION_DAYS > STANDARD_RETENTION_DAYS);
    }

    #[test]
    fn test_time_constants_milliseconds() {
        assert_eq!(MILLIS_PER_SECOND, 1000);
        assert_eq!(MILLIS_PER_HOUR, 3_600_000);
        assert_eq!(MILLIS_PER_DAY, 86_400_000);
    }

    #[test]
    fn test_time_constants_seconds() {
        assert_eq!(SECONDS_PER_DAY, 86_400);
    }

    #[test]
    fn test_time_constants_relationships() {
        assert_eq!(MILLIS_PER_HOUR, 60 * 60 * MILLIS_PER_SECOND);
        assert_eq!(MILLIS_PER_DAY, 24 * MILLIS_PER_HOUR);
    }

    #[test]
    fn test_rate_limits_anonymous() {
        assert_eq!(ANON_HOURLY_LIMIT, 10);
        assert_eq!(ANON_DAILY_LIMIT, 50);
        assert!(ANON_DAILY_LIMIT > ANON_HOURLY_LIMIT);
    }

    #[test]
    fn test_rate_limits_standard() {
        assert_eq!(STANDARD_HOURLY_LIMIT, 100);
        assert_eq!(STANDARD_DAILY_LIMIT, 500);
        assert!(STANDARD_DAILY_LIMIT > STANDARD_HOURLY_LIMIT);
    }

    #[test]
    fn test_rate_limits_premium() {
        assert_eq!(PREMIUM_HOURLY_LIMIT, 1000);
        assert_eq!(PREMIUM_DAILY_LIMIT, 5000);
        assert!(PREMIUM_DAILY_LIMIT > PREMIUM_HOURLY_LIMIT);
    }

    #[test]
    fn test_rate_limits_tier_progression() {
        assert!(STANDARD_HOURLY_LIMIT > ANON_HOURLY_LIMIT);
        assert!(PREMIUM_HOURLY_LIMIT > STANDARD_HOURLY_LIMIT);

        assert!(STANDARD_DAILY_LIMIT > ANON_DAILY_LIMIT);
        assert!(PREMIUM_DAILY_LIMIT > STANDARD_DAILY_LIMIT);
    }

    #[test]
    fn test_rate_limits_tier_multipliers() {
        assert_eq!(STANDARD_HOURLY_LIMIT, ANON_HOURLY_LIMIT * 10);
        assert_eq!(PREMIUM_HOURLY_LIMIT, STANDARD_HOURLY_LIMIT * 10);

        assert_eq!(STANDARD_DAILY_LIMIT, ANON_DAILY_LIMIT * 10);
        assert_eq!(PREMIUM_DAILY_LIMIT, STANDARD_DAILY_LIMIT * 10);
    }

    #[test]
    fn test_http_status_codes() {
        assert_eq!(HTTP_OK, 200);
        assert_eq!(HTTP_CREATED, 201);
        assert_eq!(HTTP_BAD_REQUEST, 400);
        assert_eq!(HTTP_UNAUTHORIZED, 401);
        assert_eq!(HTTP_NOT_FOUND, 404);
        assert_eq!(HTTP_PAYLOAD_TOO_LARGE, 413);
        assert_eq!(HTTP_TOO_MANY_REQUESTS, 429);
    }

    #[test]
    fn test_hash_and_id_constants() {
        assert_eq!(HASH_TRUNCATE_BYTES, 16);
        assert_eq!(FILE_ID_UUID_BYTES, 8);
        assert_eq!(MIN_MAGIC_BYTES_LENGTH, 4);
    }

    #[test]
    fn test_db_bool_constants() {
        assert_eq!(DB_BOOL_TRUE, 1);
        assert_eq!(DB_BOOL_FALSE, 0);
        assert_ne!(DB_BOOL_TRUE, DB_BOOL_FALSE);
    }

    #[test]
    fn test_error_messages_not_empty() {
        assert!(!ERR_RATE_LIMIT_EXCEEDED.is_empty());
        assert!(!ERR_FILE_NOT_FOUND.is_empty());
        assert!(!ERR_INVALID_API_KEY.is_empty());
        assert!(!ERR_FILE_TOO_LARGE.is_empty());
        assert!(!ERR_FILE_ID_REQUIRED.is_empty());
    }

    #[test]
    fn test_header_names_not_empty() {
        assert!(!HEADER_API_KEY.is_empty());
        assert!(!HEADER_ADMIN_TOKEN.is_empty());
        assert!(!HEADER_CF_CONNECTING_IP.is_empty());
        assert!(!HEADER_CONTENT_TYPE.is_empty());
    }

    #[test]
    fn test_durable_object_urls() {
        assert!(DO_RATE_LIMITER_BASE_URL.starts_with("http"));
        assert!(DO_RATE_LIMITER_PATH_CHECK.starts_with('/'));
        assert!(DO_RATE_LIMITER_PATH_RESET.starts_with('/'));
    }
}
