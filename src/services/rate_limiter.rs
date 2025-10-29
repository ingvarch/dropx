use worker::{Env, Result};
use crate::models::api_key::{ApiKeyTier, ANON_HOURLY_LIMIT, ANON_DAILY_LIMIT};

pub struct RateLimiterService;

impl RateLimiterService {
    pub async fn check_limit(
        env: &Env,
        user_id: &str,
        tier: Option<ApiKeyTier>,
    ) -> Result<()> {
        let namespace = env.durable_object("RATE_LIMITER")?;
        let stub = namespace.id_from_name(user_id)?.get_stub()?;

        let (hourly_limit, daily_limit) = match tier {
            Some(ApiKeyTier::Standard) => (100, 500),
            Some(ApiKeyTier::Premium) => (1000, 5000),
            None => (ANON_HOURLY_LIMIT, ANON_DAILY_LIMIT),
        };

        let url = format!(
            "http://rate-limiter/check?hourly={}&daily={}",
            hourly_limit, daily_limit
        );

        let req = worker::Request::new(&url, worker::Method::Get)?;
        let response = stub.fetch_with_request(req).await?;

        if response.status_code() == 429 {
            return Err(worker::Error::RustError("Rate limit exceeded".to_string()));
        }

        Ok(())
    }
}
