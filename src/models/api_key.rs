use serde::{Deserialize, Serialize};
use crate::utils::constants::{
    ANON_MAX_FILE_SIZE, ANON_RETENTION_DAYS, ANON_HOURLY_LIMIT, ANON_DAILY_LIMIT,
    STANDARD_MAX_FILE_SIZE, PREMIUM_MAX_FILE_SIZE,
    STANDARD_RETENTION_DAYS, PREMIUM_RETENTION_DAYS,
    STANDARD_HOURLY_LIMIT, STANDARD_DAILY_LIMIT,
    PREMIUM_HOURLY_LIMIT, PREMIUM_DAILY_LIMIT,
};

#[derive(Debug)]
pub struct ApiKey {
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
    pub fn as_str(&self) -> &str {
        match self {
            ApiKeyTier::Standard => "standard",
            ApiKeyTier::Premium => "premium",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub email: String,
    pub tier: ApiKeyTier,
}

#[derive(Debug)]
pub struct Limits {
    pub hourly: usize,
    pub daily: usize,
    pub max_file_size: usize,
    pub retention_days: i64,
}

impl Limits {
    /// `None` is an anonymous upload.
    pub fn for_tier(tier: Option<ApiKeyTier>) -> Self {
        match tier {
            None => Self {
                hourly: ANON_HOURLY_LIMIT,
                daily: ANON_DAILY_LIMIT,
                max_file_size: ANON_MAX_FILE_SIZE,
                retention_days: ANON_RETENTION_DAYS,
            },
            Some(ApiKeyTier::Standard) => Self {
                hourly: STANDARD_HOURLY_LIMIT,
                daily: STANDARD_DAILY_LIMIT,
                max_file_size: STANDARD_MAX_FILE_SIZE,
                retention_days: STANDARD_RETENTION_DAYS,
            },
            Some(ApiKeyTier::Premium) => Self {
                hourly: PREMIUM_HOURLY_LIMIT,
                daily: PREMIUM_DAILY_LIMIT,
                max_file_size: PREMIUM_MAX_FILE_SIZE,
                retention_days: PREMIUM_RETENTION_DAYS,
            },
        }
    }
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
    fn as_str_matches_the_serde_name() {
        for tier in [ApiKeyTier::Standard, ApiKeyTier::Premium] {
            assert_eq!(serde_json::to_string(&tier).unwrap(), format!("\"{}\"", tier.as_str()));
        }
    }

    #[test]
    fn test_api_key_tier_deserialization() {
        let tier: ApiKeyTier = serde_json::from_str("\"standard\"").unwrap();
        assert_eq!(tier, ApiKeyTier::Standard);

        let tier: ApiKeyTier = serde_json::from_str("\"premium\"").unwrap();
        assert_eq!(tier, ApiKeyTier::Premium);
    }

    #[test]
    fn test_create_api_key_request_deserialization() {
        let json = r#"{"email":"test@example.com","tier":"standard"}"#;
        let request: CreateApiKeyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.email, "test@example.com");
        assert_eq!(request.tier, ApiKeyTier::Standard);
    }
}
