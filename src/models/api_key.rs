use serde::{Deserialize, Serialize};

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
            ApiKeyTier::Standard => 100,
            ApiKeyTier::Premium => 1000,
        }
    }

    pub fn daily_limit(&self) -> usize {
        match self {
            ApiKeyTier::Standard => 500,
            ApiKeyTier::Premium => 5000,
        }
    }

    pub fn max_file_size(&self) -> usize {
        match self {
            ApiKeyTier::Standard => 100 * 1024 * 1024,
            ApiKeyTier::Premium => 500 * 1024 * 1024,
        }
    }

    pub fn retention_days(&self) -> i64 {
        match self {
            ApiKeyTier::Standard => 7,
            ApiKeyTier::Premium => 30,
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

pub const ANON_HOURLY_LIMIT: usize = 10;
pub const ANON_DAILY_LIMIT: usize = 50;
