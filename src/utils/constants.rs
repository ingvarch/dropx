pub const BYTES_PER_KB: usize = 1024;
pub const BYTES_PER_MB: usize = 1024 * BYTES_PER_KB;

pub const ANON_MAX_FILE_SIZE: usize = 25 * BYTES_PER_MB;
// Standard and premium: the Pro plan's request body limit. Cloudflare rejects larger bodies before they reach the Worker.
pub const STANDARD_MAX_FILE_SIZE: usize = 100 * BYTES_PER_MB;
pub const PREMIUM_MAX_FILE_SIZE: usize = 100 * BYTES_PER_MB;
pub const MAX_BUFFERED_UPLOAD_SIZE: usize = 25 * BYTES_PER_MB;
pub const MIME_SNIFF_BYTES: u64 = 16;

pub const ANON_RETENTION_DAYS: i64 = 3;
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
pub const ANON_DAILY_LIMIT: usize = 25;
pub const STANDARD_HOURLY_LIMIT: usize = 100;
pub const STANDARD_DAILY_LIMIT: usize = 500;
pub const PREMIUM_HOURLY_LIMIT: usize = 1000;
pub const PREMIUM_DAILY_LIMIT: usize = 5000;

pub const HASH_TRUNCATE_BYTES: usize = 16;
pub const IPV6_HOST_BITS: u32 = 64;
pub const ENV_IP_HASH_KEY: &str = "IP_HASH_KEY";
pub const FILE_ID_UUID_BYTES: usize = 8;

pub const HTTP_OK: u16 = 200;
pub const HTTP_CREATED: u16 = 201;
pub const HTTP_BAD_REQUEST: u16 = 400;
pub const HTTP_UNAUTHORIZED: u16 = 401;
pub const HTTP_NOT_FOUND: u16 = 404;
pub const HTTP_LENGTH_REQUIRED: u16 = 411;
pub const HTTP_PAYLOAD_TOO_LARGE: u16 = 413;
pub const HTTP_TOO_MANY_REQUESTS: u16 = 429;

pub const HEADER_API_KEY: &str = "X-API-Key";
pub const HEADER_ADMIN_TOKEN: &str = "X-Admin-Token";
pub const HEADER_CF_CONNECTING_IP: &str = "CF-Connecting-IP";
pub const HEADER_CONTENT_TYPE: &str = "Content-Type";
pub const HEADER_CONTENT_DISPOSITION: &str = "Content-Disposition";
pub const HEADER_CONTENT_LENGTH: &str = "Content-Length";
pub const HEADER_RETRY_AFTER: &str = "Retry-After";
pub const HEADER_HOST: &str = "Host";
pub const HEADER_DOWNLOAD_COUNT: &str = "X-Download-Count";
pub const HEADER_EXPIRES_AT: &str = "X-Expires-At";
pub const HEADER_CONTENT_TYPE_OPTIONS: &str = "X-Content-Type-Options";
pub const HEADER_MAX_DOWNLOADS: &str = "Max-Downloads";
pub const HEADER_MAX_AGE_DAYS: &str = "Max-Age-Days";

pub const CONTENT_TYPE_OPTIONS_NOSNIFF: &str = "nosniff";
pub const INLINE_ACTIVE_CONTENT_TYPE: &str = "text/plain; charset=utf-8";

pub const DO_RATE_LIMITER_BASE_URL: &str = "http://rate-limiter";
pub const DO_RATE_LIMITER_PATH_CHECK: &str = "/check";
pub const DO_RATE_LIMITER_PATH_RESET: &str = "/reset";

pub const DB_BOOL_TRUE: i64 = 1;
pub const DB_BOOL_FALSE: i64 = 0;

pub const ERR_RATE_LIMIT_EXCEEDED: &str = "Rate limit exceeded";
pub const ERR_FILE_NOT_FOUND: &str = "File not found";
pub const ERR_INVALID_API_KEY: &str = "Invalid API key";
pub const ERR_FILE_TOO_LARGE: &str = "File too large";
pub const ERR_LENGTH_REQUIRED: &str = "Length required";
pub const ERR_FILE_ID_REQUIRED: &str = "File ID is required";
pub const ERR_INVALID_RETENTION: &str = "Invalid retention period";
pub const ERR_UNAUTHORIZED: &str = "Unauthorized";
pub const ERR_NOT_FOUND: &str = "NotFound";

pub const UNKNOWN_IP_ADDRESS: &str = "unknown";
pub const DEFAULT_FILENAME: &str = "file";
