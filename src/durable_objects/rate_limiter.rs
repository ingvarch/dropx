use worker::*;
use serde::{Deserialize, Serialize};
use crate::utils::constants::{
    MILLIS_PER_HOUR, MILLIS_PER_DAY, MILLIS_PER_SECOND,
    ANON_HOURLY_LIMIT, ANON_DAILY_LIMIT,
    DO_RATE_LIMITER_PATH_CHECK, DO_RATE_LIMITER_PATH_RESET,
    HTTP_NOT_FOUND, HTTP_TOO_MANY_REQUESTS, HEADER_RETRY_AFTER,
};

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
}

impl DurableObject for RateLimiter {
    fn new(state: State, _env: Env) -> Self {
        Self { state }
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
            .unwrap_or(ANON_HOURLY_LIMIT as u32);

        let daily_limit: u32 = query_pairs
            .get("daily")
            .and_then(|v| v.parse().ok())
            .unwrap_or(ANON_DAILY_LIMIT as u32);

        match url.path() {
            path if path == DO_RATE_LIMITER_PATH_CHECK => self.check_limit(hourly_limit, daily_limit).await,
            path if path == DO_RATE_LIMITER_PATH_RESET => self.reset_counters().await,
            _ => Response::error("Not found", HTTP_NOT_FOUND),
        }
    }
}

impl RateLimiter {
    fn rate_limit_response(message: &str, reset_at: u64, now: u64) -> Result<Response> {
        let retry_after = ((reset_at - now) / MILLIS_PER_SECOND) as i64;
        let headers = Headers::new();
        headers.set(HEADER_RETRY_AFTER, &retry_after.to_string())?;
        Ok(Response::error(message, HTTP_TOO_MANY_REQUESTS)?.with_headers(headers))
    }

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
            return Self::rate_limit_response("Hourly rate limit exceeded", counts.hour_reset_at, now);
        }

        if counts.daily >= daily_limit {
            return Self::rate_limit_response("Daily rate limit exceeded", counts.day_reset_at, now);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::constants::{
        STANDARD_HOURLY_LIMIT, STANDARD_DAILY_LIMIT,
        PREMIUM_HOURLY_LIMIT, PREMIUM_DAILY_LIMIT,
    };

    #[test]
    fn test_rate_limit_counts_default() {
        let counts = RateLimitCounts::default();
        assert_eq!(counts.hourly, 0);
        assert_eq!(counts.daily, 0);
        assert_eq!(counts.hour_reset_at, 0);
        assert_eq!(counts.day_reset_at, 0);
    }

    #[test]
    fn test_rate_limit_counts_serialization() {
        let counts = RateLimitCounts {
            hourly: 5,
            daily: 15,
            hour_reset_at: 1000000,
            day_reset_at: 2000000,
        };

        let serialized = serde_json::to_string(&counts).unwrap();
        let deserialized: RateLimitCounts = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.hourly, 5);
        assert_eq!(deserialized.daily, 15);
        assert_eq!(deserialized.hour_reset_at, 1000000);
        assert_eq!(deserialized.day_reset_at, 2000000);
    }

    #[test]
    fn test_hourly_reset_time_calculation() {
        let now: u64 = 1000000;
        let expected_reset = now + MILLIS_PER_HOUR;
        assert_eq!(expected_reset, 1000000 + 3600000);
    }

    #[test]
    fn test_daily_reset_time_calculation() {
        let now: u64 = 1000000;
        let expected_reset = now + MILLIS_PER_DAY;
        assert_eq!(expected_reset, 1000000 + 86400000);
    }

    #[test]
    fn test_retry_after_calculation() {
        let now: u64 = 1000000;
        let reset_at: u64 = 3600000;
        let retry_after = ((reset_at - now) / MILLIS_PER_SECOND) as i64;
        assert_eq!(retry_after, 2600);
    }

    #[test]
    fn test_retry_after_calculation_near_boundary() {
        let now: u64 = 1003599500;
        let reset_at: u64 = 1003600000;
        let retry_after = ((reset_at - now) / MILLIS_PER_SECOND) as i64;
        assert_eq!(retry_after, 0);
    }

    #[test]
    fn test_hourly_limit_check_anon() {
        let hourly_limit = ANON_HOURLY_LIMIT as u32;
        let count = 9u32;
        assert!(count < hourly_limit);

        let count = 10u32;
        assert!(count >= hourly_limit);
    }

    #[test]
    fn test_daily_limit_check_anon() {
        let daily_limit = ANON_DAILY_LIMIT as u32;
        let count = 49u32;
        assert!(count < daily_limit);

        let count = 50u32;
        assert!(count >= daily_limit);
    }

    #[test]
    fn test_hourly_limit_check_standard() {
        let hourly_limit = STANDARD_HOURLY_LIMIT as u32;
        let count = 99u32;
        assert!(count < hourly_limit);

        let count = 100u32;
        assert!(count >= hourly_limit);
    }

    #[test]
    fn test_daily_limit_check_standard() {
        let daily_limit = STANDARD_DAILY_LIMIT as u32;
        let count = 499u32;
        assert!(count < daily_limit);

        let count = 500u32;
        assert!(count >= daily_limit);
    }

    #[test]
    fn test_hourly_limit_check_premium() {
        let hourly_limit = PREMIUM_HOURLY_LIMIT as u32;
        let count = 999u32;
        assert!(count < hourly_limit);

        let count = 1000u32;
        assert!(count >= hourly_limit);
    }

    #[test]
    fn test_daily_limit_check_premium() {
        let daily_limit = PREMIUM_DAILY_LIMIT as u32;
        let count = 4999u32;
        assert!(count < daily_limit);

        let count = 5000u32;
        assert!(count >= daily_limit);
    }

    #[test]
    fn test_reset_logic_hourly() {
        let now: u64 = 1003600001;
        let counts = RateLimitCounts {
            hourly: 50,
            daily: 100,
            hour_reset_at: 1003600000,
            day_reset_at: 2000000000,
        };

        let should_reset_hourly = now > counts.hour_reset_at;
        let should_reset_daily = now > counts.day_reset_at;

        assert!(should_reset_hourly);
        assert!(!should_reset_daily);
    }

    #[test]
    fn test_reset_logic_daily() {
        let now: u64 = 2000000001;
        let counts = RateLimitCounts {
            hourly: 50,
            daily: 100,
            hour_reset_at: 1003600000,
            day_reset_at: 2000000000,
        };

        let should_reset_hourly = now > counts.hour_reset_at;
        let should_reset_daily = now > counts.day_reset_at;

        assert!(should_reset_hourly);
        assert!(should_reset_daily);
    }

    #[test]
    fn test_reset_logic_both_boundaries() {
        let now: u64 = 2000000000;
        let counts = RateLimitCounts {
            hourly: 50,
            daily: 100,
            hour_reset_at: 2000000000,
            day_reset_at: 2000000000,
        };

        let should_reset_hourly = now > counts.hour_reset_at;
        let should_reset_daily = now > counts.day_reset_at;

        assert!(!should_reset_hourly);
        assert!(!should_reset_daily);
    }
}
