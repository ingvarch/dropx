use worker::{Env, Result};
use crate::models::Limits;
use crate::utils::constants::{
    RATE_LIMITER_NAME,
    DO_RATE_LIMITER_BASE_URL, DO_RATE_LIMITER_PATH_CHECK, DO_RATE_LIMITER_PATH_RESET,
    HTTP_TOO_MANY_REQUESTS, HTTP_OK, HEADER_RETRY_AFTER,
};

#[derive(Debug, PartialEq, Eq)]
pub enum RateLimitCheck {
    Allowed,
    Limited { retry_after: u64, message: String },
}

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
        limits: &Limits,
    ) -> Result<RateLimitCheck> {
        let url = Self::build_check_url(limits.hourly, limits.daily);
        let mut response = Self::fetch_from_do(env, user_id, &url).await?;

        let retry_after = response.headers().get(HEADER_RETRY_AFTER)?;
        let body = response.text().await?;
        Self::parse_check_response(response.status_code(), retry_after.as_deref(), &body)
    }

    fn parse_check_response(status: u16, retry_after: Option<&str>, body: &str) -> Result<RateLimitCheck> {
        match status {
            HTTP_OK => Ok(RateLimitCheck::Allowed),
            HTTP_TOO_MANY_REQUESTS => {
                let retry_after = retry_after
                    .and_then(|value| value.parse().ok())
                    .ok_or_else(|| worker::Error::RustError("Rate limiter sent no valid Retry-After".to_string()))?;
                Ok(RateLimitCheck::Limited { retry_after, message: body.to_string() })
            }
            status => Err(worker::Error::RustError(format!("Rate limiter returned status {}", status))),
        }
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
    fn parse_check_response_ok_is_allowed() {
        let check = RateLimiterService::parse_check_response(200, None, "OK").unwrap();
        assert_eq!(check, RateLimitCheck::Allowed);
    }

    #[test]
    fn parse_check_response_429_carries_retry_after_and_message() {
        let check = RateLimiterService::parse_check_response(429, Some("120"), "Hourly rate limit exceeded").unwrap();
        assert_eq!(
            check,
            RateLimitCheck::Limited { retry_after: 120, message: "Hourly rate limit exceeded".to_string() }
        );
    }

    #[test]
    fn parse_check_response_429_without_retry_after_is_error() {
        assert!(RateLimiterService::parse_check_response(429, None, "x").is_err());
    }

    #[test]
    fn parse_check_response_429_with_non_numeric_retry_after_is_error() {
        assert!(RateLimiterService::parse_check_response(429, Some("soon"), "x").is_err());
    }

    #[test]
    fn parse_check_response_other_status_is_error() {
        assert!(RateLimiterService::parse_check_response(500, Some("5"), "x").is_err());
        assert!(RateLimiterService::parse_check_response(400, None, "x").is_err());
    }
}
