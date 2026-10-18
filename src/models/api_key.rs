use serde::{Deserialize, Serialize};
use crate::utils::constants::{
    STANDARD_MAX_FILE_SIZE, PREMIUM_MAX_FILE_SIZE,
    STANDARD_RETENTION_DAYS, PREMIUM_RETENTION_DAYS,
    STANDARD_HOURLY_LIMIT, STANDARD_DAILY_LIMIT,
    PREMIUM_HOURLY_LIMIT, PREMIUM_DAILY_LIMIT,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key_hash: String,
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
    fn standard_tier_uses_the_standard_constants() {
        let tier = ApiKeyTier::Standard;
        assert_eq!(tier.max_file_size(), STANDARD_MAX_FILE_SIZE);
        assert_eq!(tier.retention_days(), STANDARD_RETENTION_DAYS);
        assert_eq!(tier.hourly_limit(), STANDARD_HOURLY_LIMIT);
        assert_eq!(tier.daily_limit(), STANDARD_DAILY_LIMIT);
    }

    #[test]
    fn premium_tier_uses_the_premium_constants() {
        let tier = ApiKeyTier::Premium;
        assert_eq!(tier.max_file_size(), PREMIUM_MAX_FILE_SIZE);
        assert_eq!(tier.retention_days(), PREMIUM_RETENTION_DAYS);
        assert_eq!(tier.hourly_limit(), PREMIUM_HOURLY_LIMIT);
        assert_eq!(tier.daily_limit(), PREMIUM_DAILY_LIMIT);
    }

    #[test]
    fn test_create_api_key_request_deserialization() {
        let json = r#"{"email":"test@example.com","tier":"standard"}"#;
        let request: CreateApiKeyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.email, "test@example.com");
        assert_eq!(request.tier, ApiKeyTier::Standard);
    }
}
