use worker::{Env, Result};
use crate::models::api_key::ApiKeyTier;
use crate::utils::constants::{
    RATE_LIMITER_NAME, ANON_HOURLY_LIMIT, ANON_DAILY_LIMIT,
    DO_RATE_LIMITER_BASE_URL, DO_RATE_LIMITER_PATH_CHECK, DO_RATE_LIMITER_PATH_RESET,
    HTTP_TOO_MANY_REQUESTS, HTTP_OK, ERR_RATE_LIMIT_EXCEEDED,
};

pub struct RateLimiterService;

impl RateLimiterService {
    fn build_check_url(hourly_limit: usize, daily_limit: usize) -> String {
        format!(
            "{}{}?hourly={}&daily={}",
            DO_RATE_LIMITER_BASE_URL, DO_RATE_LIMITER_PATH_CHECK,
            hourly_limit, daily_limit
        )
    }

    fn build_reset_url() -> String {
        format!("{}{}", DO_RATE_LIMITER_BASE_URL, DO_RATE_LIMITER_PATH_RESET)
    }

    async fn fetch_from_do(
        env: &Env,
        user_id: &str,
        url: &str,
    ) -> Result<worker::Response> {
        let namespace = env.durable_object(RATE_LIMITER_NAME)?;
        let stub = namespace.id_from_name(user_id)?.get_stub()?;
        let req = worker::Request::new(url, worker::Method::Get)?;
        stub.fetch_with_request(req).await
    }

    pub async fn check_limit(
        env: &Env,
        user_id: &str,
        tier: Option<ApiKeyTier>,
    ) -> Result<()> {
        let (hourly_limit, daily_limit) = match tier {
            Some(t) => (t.hourly_limit(), t.daily_limit()),
            None => (ANON_HOURLY_LIMIT, ANON_DAILY_LIMIT),
        };

        let url = Self::build_check_url(hourly_limit, daily_limit);
        let response = Self::fetch_from_do(env, user_id, &url).await?;

        if response.status_code() == HTTP_TOO_MANY_REQUESTS {
            return Err(worker::Error::RustError(ERR_RATE_LIMIT_EXCEEDED.to_string()));
        }

        Ok(())
    }

    pub async fn reset_limit(env: &Env, user_id: &str) -> Result<()> {
        let url = Self::build_reset_url();
        let response = Self::fetch_from_do(env, user_id, &url).await?;

        if response.status_code() != HTTP_OK {
            return Err(worker::Error::RustError("Failed to reset rate limit".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::constants::{
        STANDARD_HOURLY_LIMIT, STANDARD_DAILY_LIMIT,
        PREMIUM_HOURLY_LIMIT, PREMIUM_DAILY_LIMIT,
    };

    #[test]
    fn test_build_check_url_anon() {
        let url = RateLimiterService::build_check_url(ANON_HOURLY_LIMIT, ANON_DAILY_LIMIT);
        assert_eq!(url, "http://rate-limiter/check?hourly=10&daily=50");
    }

    #[test]
    fn test_build_check_url_standard() {
        let url = RateLimiterService::build_check_url(STANDARD_HOURLY_LIMIT, STANDARD_DAILY_LIMIT);
        assert_eq!(url, "http://rate-limiter/check?hourly=100&daily=500");
    }

    #[test]
    fn test_build_check_url_premium() {
        let url = RateLimiterService::build_check_url(PREMIUM_HOURLY_LIMIT, PREMIUM_DAILY_LIMIT);
        assert_eq!(url, "http://rate-limiter/check?hourly=1000&daily=5000");
    }

    #[test]
    fn test_build_check_url_custom() {
        let url = RateLimiterService::build_check_url(999, 4444);
        assert_eq!(url, "http://rate-limiter/check?hourly=999&daily=4444");
    }

    #[test]
    fn test_build_reset_url() {
        let url = RateLimiterService::build_reset_url();
        assert_eq!(url, "http://rate-limiter/reset");
    }

    #[test]
    fn test_tier_limits_standard() {
        let tier = ApiKeyTier::Standard;
        assert_eq!(tier.hourly_limit(), 100);
        assert_eq!(tier.daily_limit(), 500);
    }

    #[test]
    fn test_tier_limits_premium() {
        let tier = ApiKeyTier::Premium;
        assert_eq!(tier.hourly_limit(), 1000);
        assert_eq!(tier.daily_limit(), 5000);
    }

    #[test]
    fn test_tier_limits_none() {
        let (hourly, daily) = (ANON_HOURLY_LIMIT, ANON_DAILY_LIMIT);
        assert_eq!(hourly, 10);
        assert_eq!(daily, 50);
    }

    #[test]
    fn test_check_limit_url_construction_with_standard_tier() {
        let tier = Some(ApiKeyTier::Standard);
        let (hourly, daily) = match tier {
            Some(t) => (t.hourly_limit(), t.daily_limit()),
            None => (ANON_HOURLY_LIMIT, ANON_DAILY_LIMIT),
        };
        let url = RateLimiterService::build_check_url(hourly, daily);
        assert_eq!(url, "http://rate-limiter/check?hourly=100&daily=500");
    }

    #[test]
    fn test_check_limit_url_construction_with_premium_tier() {
        let tier = Some(ApiKeyTier::Premium);
        let (hourly, daily) = match tier {
            Some(t) => (t.hourly_limit(), t.daily_limit()),
            None => (ANON_HOURLY_LIMIT, ANON_DAILY_LIMIT),
        };
        let url = RateLimiterService::build_check_url(hourly, daily);
        assert_eq!(url, "http://rate-limiter/check?hourly=1000&daily=5000");
    }

    #[test]
    fn test_check_limit_url_construction_with_no_tier() {
        let tier: Option<ApiKeyTier> = None;
        let (hourly, daily) = match tier {
            Some(t) => (t.hourly_limit(), t.daily_limit()),
            None => (ANON_HOURLY_LIMIT, ANON_DAILY_LIMIT),
        };
        let url = RateLimiterService::build_check_url(hourly, daily);
        assert_eq!(url, "http://rate-limiter/check?hourly=10&daily=50");
    }
}
