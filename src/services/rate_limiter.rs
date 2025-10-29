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
