use worker::*;
use serde::{Deserialize, Serialize};
use crate::utils::constants::{MILLIS_PER_HOUR, MILLIS_PER_DAY};

#[derive(Debug, Serialize, Deserialize, Default)]
struct RateLimitCounts {
    hourly: u32,
    daily: u32,
    hour_reset_at: u64,
    day_reset_at: u64,
}

#[durable_object]
pub struct RateLimiter {
    state: State,
    #[allow(dead_code)]
    env: Env,
}

impl DurableObject for RateLimiter {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let url = req.url()?;
        let query_pairs: std::collections::HashMap<String, String> = url
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();

        let hourly_limit: u32 = query_pairs
            .get("hourly")
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        let daily_limit: u32 = query_pairs
            .get("daily")
            .and_then(|v| v.parse().ok())
            .unwrap_or(50);

        match url.path() {
            "/check" => self.check_limit(hourly_limit, daily_limit).await,
            "/reset" => self.reset_counters().await,
            _ => Response::error("Not found", 404),
        }
    }
}

impl RateLimiter {
    async fn check_limit(&self, hourly_limit: u32, daily_limit: u32) -> Result<Response> {
        let mut counts: RateLimitCounts = self
            .state
            .storage()
            .get("counts")
            .await
            .unwrap_or_default();

        let now = Date::now().as_millis();

        if now > counts.hour_reset_at {
            counts.hourly = 0;
            counts.hour_reset_at = now + MILLIS_PER_HOUR;
        }

        if now > counts.day_reset_at {
            counts.daily = 0;
            counts.day_reset_at = now + MILLIS_PER_DAY;
        }

        if counts.hourly >= hourly_limit {
            let retry_after = ((counts.hour_reset_at - now) / 1000) as i64;
            let headers = Headers::new();
            headers.set("Retry-After", &retry_after.to_string())?;
            return Ok(Response::error("Hourly rate limit exceeded", 429)?
                .with_headers(headers));
        }

        if counts.daily >= daily_limit {
            let retry_after = ((counts.day_reset_at - now) / 1000) as i64;
            let headers = Headers::new();
            headers.set("Retry-After", &retry_after.to_string())?;
            return Ok(Response::error("Daily rate limit exceeded", 429)?
                .with_headers(headers));
        }

        counts.hourly += 1;
        counts.daily += 1;

        self.state.storage().put("counts", counts).await?;

        Response::ok("OK")
    }

    async fn reset_counters(&self) -> Result<Response> {
        self.state.storage().delete("counts").await?;
        Response::ok("Reset")
    }
}
