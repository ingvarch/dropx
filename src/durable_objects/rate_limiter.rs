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

fn reset_finished_windows(counts: &mut RateLimitCounts, now: u64) {
    if now >= counts.hour_reset_at {
        counts.hourly = 0;
        counts.hour_reset_at = now + MILLIS_PER_HOUR;
    }

    if now >= counts.day_reset_at {
        counts.daily = 0;
        counts.day_reset_at = now + MILLIS_PER_DAY;
    }
}

fn retry_after_seconds(reset_at: u64, now: u64) -> u64 {
    (reset_at - now).div_ceil(MILLIS_PER_SECOND).max(1)
}

#[derive(Debug, PartialEq, Eq)]
enum LimitDecision {
    Allowed,
    Limited { retry_after: u64, message: &'static str },
}

fn decide_limit(counts: &RateLimitCounts, hourly_limit: u32, daily_limit: u32, now: u64) -> LimitDecision {
    let windows = [
        (counts.hourly >= hourly_limit, counts.hour_reset_at, "Hourly rate limit exceeded"),
        (counts.daily >= daily_limit, counts.day_reset_at, "Daily rate limit exceeded"),
    ];
    windows
        .into_iter()
        .filter(|(full, _, _)| *full)
        .max_by_key(|(_, reset_at, _)| *reset_at)
        .map_or(LimitDecision::Allowed, |(_, reset_at, message)| LimitDecision::Limited {
            retry_after: retry_after_seconds(reset_at, now),
            message,
        })
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
    fn rate_limit_response(message: &str, retry_after: u64) -> Result<Response> {
        let headers = Headers::new();
        headers.set(HEADER_RETRY_AFTER, &retry_after.to_string())?;
        Ok(Response::error(message, HTTP_TOO_MANY_REQUESTS)?.with_headers(headers))
    }

    async fn check_limit(&self, hourly_limit: u32, daily_limit: u32) -> Result<Response> {
        let mut counts: RateLimitCounts = self
            .state
            .storage()
            .get("counts")
            .await?
            .unwrap_or_default();

        let now = Date::now().as_millis();
        reset_finished_windows(&mut counts, now);

        if let LimitDecision::Limited { retry_after, message } = decide_limit(&counts, hourly_limit, daily_limit, now) {
            return Self::rate_limit_response(message, retry_after);
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
    fn windows_reset_when_the_reset_time_is_reached() {
        let mut counts = RateLimitCounts { hourly: 10, daily: 25, hour_reset_at: 5_000, day_reset_at: 5_000 };

        reset_finished_windows(&mut counts, 5_000);

        assert_eq!((counts.hourly, counts.daily), (0, 0));
        assert_eq!(counts.hour_reset_at, 5_000 + MILLIS_PER_HOUR);
        assert_eq!(counts.day_reset_at, 5_000 + MILLIS_PER_DAY);
    }

    #[test]
    fn windows_before_the_reset_time_keep_their_counts() {
        let mut counts = RateLimitCounts { hourly: 10, daily: 25, hour_reset_at: 5_000, day_reset_at: 9_000 };

        reset_finished_windows(&mut counts, 4_999);

        assert_eq!((counts.hourly, counts.daily), (10, 25));
        assert_eq!((counts.hour_reset_at, counts.day_reset_at), (5_000, 9_000));
    }

    #[test]
    fn retry_after_is_whole_seconds_on_exact_boundary() {
        assert_eq!(retry_after_seconds(3_600_000, 1_000_000), 2600);
    }

    #[test]
    fn retry_after_rounds_sub_second_remainder_up() {
        assert_eq!(retry_after_seconds(1_003_600_000, 1_003_599_500), 1);
        assert_eq!(retry_after_seconds(3_600_001, 1_000_000), 2601);
    }

    #[test]
    fn retry_after_is_at_least_one_second() {
        assert_eq!(retry_after_seconds(1_000_000, 1_000_000), 1);
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
        let count = 24u32;
        assert!(count < daily_limit);

        let count = 25u32;
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

    const NOW: u64 = 1_000_000;

    fn counts(hourly: u32, daily: u32, hour_reset_at: u64, day_reset_at: u64) -> RateLimitCounts {
        RateLimitCounts { hourly, daily, hour_reset_at, day_reset_at }
    }

    fn limited(retry_after: u64, message: &'static str) -> LimitDecision {
        LimitDecision::Limited { retry_after, message }
    }

    #[test]
    fn decision_allows_when_no_window_is_full() {
        let decision = decide_limit(&counts(9, 24, NOW + 3_600_000, NOW + 86_400_000), 10, 25, NOW);
        assert_eq!(decision, LimitDecision::Allowed);
    }

    #[test]
    fn decision_names_hourly_when_only_hourly_is_full() {
        let decision = decide_limit(&counts(10, 5, NOW + 3_600_000, NOW + 86_400_000), 10, 25, NOW);
        assert_eq!(decision, limited(3600, "Hourly rate limit exceeded"));
    }

    #[test]
    fn decision_names_daily_when_only_daily_is_full() {
        let decision = decide_limit(&counts(3, 25, NOW + 3_600_000, NOW + 86_400_000), 10, 25, NOW);
        assert_eq!(decision, limited(86_400, "Daily rate limit exceeded"));
    }

    #[test]
    fn decision_waits_for_the_day_when_both_are_full_and_day_resets_later() {
        let decision = decide_limit(&counts(10, 25, NOW + 3_600_000, NOW + 86_400_000), 10, 25, NOW);
        assert_eq!(decision, limited(86_400, "Daily rate limit exceeded"));
    }

    #[test]
    fn decision_waits_for_the_hour_when_both_are_full_and_hour_resets_later() {
        let decision = decide_limit(&counts(10, 25, NOW + 7_200_000, NOW + 3_600_000), 10, 25, NOW);
        assert_eq!(decision, limited(7200, "Hourly rate limit exceeded"));
    }
}
