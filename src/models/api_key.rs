use serde::{Deserialize, Serialize};
use crate::utils::constants::{
    STANDARD_MAX_FILE_SIZE, PREMIUM_MAX_FILE_SIZE,
    STANDARD_RETENTION_DAYS, PREMIUM_RETENTION_DAYS,
    STANDARD_HOURLY_LIMIT, STANDARD_DAILY_LIMIT,
    PREMIUM_HOURLY_LIMIT, PREMIUM_DAILY_LIMIT,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key: String,
    pub user_email: Option<String>,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub is_active: bool,
    pub tier: ApiKeyTier,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyTier {
    Standard,
    Premium,
}

impl ApiKeyTier {
    pub fn hourly_limit(&self) -> usize {
        match self {
            ApiKeyTier::Standard => STANDARD_HOURLY_LIMIT,
            ApiKeyTier::Premium => PREMIUM_HOURLY_LIMIT,
        }
    }

    pub fn daily_limit(&self) -> usize {
        match self {
            ApiKeyTier::Standard => STANDARD_DAILY_LIMIT,
            ApiKeyTier::Premium => PREMIUM_DAILY_LIMIT,
        }
    }

    pub fn max_file_size(&self) -> usize {
        match self {
            ApiKeyTier::Standard => STANDARD_MAX_FILE_SIZE,
            ApiKeyTier::Premium => PREMIUM_MAX_FILE_SIZE,
        }
    }

    pub fn retention_days(&self) -> i64 {
        match self {
            ApiKeyTier::Standard => STANDARD_RETENTION_DAYS,
            ApiKeyTier::Premium => PREMIUM_RETENTION_DAYS,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ApiKeyTier::Standard => "standard",
            ApiKeyTier::Premium => "premium",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "premium" => ApiKeyTier::Premium,
            _ => ApiKeyTier::Standard,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub email: String,
    pub tier: ApiKeyTier,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub key: String,
    pub tier: ApiKeyTier,
    pub created_at: i64,
    pub limits: ApiKeyLimits,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyLimits {
    pub hourly: usize,
    pub daily: usize,
}

#[derive(Debug, Serialize)]
pub struct RevokeApiKeyResponse {
    pub message: String,
    pub key: String,
}

#[derive(Debug, Serialize)]
pub struct ResetRateLimitResponse {
    pub message: String,
    pub user_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_tier_serialization() {
        let tier = ApiKeyTier::Standard;
        let json = serde_json::to_string(&tier).unwrap();
        assert_eq!(json, "\"standard\"");

        let tier = ApiKeyTier::Premium;
        let json = serde_json::to_string(&tier).unwrap();
        assert_eq!(json, "\"premium\"");
    }

    #[test]
    fn test_api_key_tier_deserialization() {
        let tier: ApiKeyTier = serde_json::from_str("\"standard\"").unwrap();
        assert_eq!(tier, ApiKeyTier::Standard);

        let tier: ApiKeyTier = serde_json::from_str("\"premium\"").unwrap();
        assert_eq!(tier, ApiKeyTier::Premium);
    }

    #[test]
    fn test_api_key_tier_max_file_size() {
        assert_eq!(ApiKeyTier::Standard.max_file_size(), 100 * 1024 * 1024);
        assert_eq!(ApiKeyTier::Premium.max_file_size(), 500 * 1024 * 1024);
    }

    #[test]
    fn test_api_key_tier_retention_days() {
        assert_eq!(ApiKeyTier::Standard.retention_days(), 7);
        assert_eq!(ApiKeyTier::Premium.retention_days(), 30);
    }

    #[test]
    fn test_api_key_tier_hourly_limit() {
        assert_eq!(ApiKeyTier::Standard.hourly_limit(), 100);
        assert_eq!(ApiKeyTier::Premium.hourly_limit(), 1000);
    }

    #[test]
    fn test_api_key_tier_daily_limit() {
        assert_eq!(ApiKeyTier::Standard.daily_limit(), 500);
        assert_eq!(ApiKeyTier::Premium.daily_limit(), 5000);
    }

    #[test]
    fn test_api_key_serialization() {
        let api_key = ApiKey {
            key: "tfr_abc123".to_string(),
            user_email: Some("user@example.com".to_string()),
            created_at: 1000000,
            last_used_at: Some(1000100),
            is_active: true,
            tier: ApiKeyTier::Premium,
        };

        let json = serde_json::to_string(&api_key).unwrap();
        assert!(json.contains("\"key\":\"tfr_abc123\""));
        assert!(json.contains("\"user_email\":\"user@example.com\""));
        assert!(json.contains("\"tier\":\"premium\""));
    }

    #[test]
    fn test_create_api_key_request_deserialization() {
        let json = r#"{"email":"test@example.com","tier":"standard"}"#;
        let request: CreateApiKeyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.email, "test@example.com");
        assert_eq!(request.tier, ApiKeyTier::Standard);
    }

    #[test]
    fn test_api_key_limits_serialization() {
        let limits = ApiKeyLimits {
            hourly: 100,
            daily: 500,
        };

        let json = serde_json::to_string(&limits).unwrap();
        assert!(json.contains("\"hourly\":100"));
        assert!(json.contains("\"daily\":500"));
    }
}

