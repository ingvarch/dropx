use worker::*;
use serde::{Deserialize, Serialize};
use crate::utils::constants::{
    MILLIS_PER_HOUR, MILLIS_PER_DAY, MILLIS_PER_SECOND,
    DO_RATE_LIMITER_PATH_CHECK, DO_RATE_LIMITER_PATH_RESET,
    HTTP_BAD_REQUEST, HTTP_NOT_FOUND, HTTP_TOO_MANY_REQUESTS, HEADER_RETRY_AFTER,
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

fn parse_check_limits(url: &Url) -> Option<(u32, u32)> {
    let query_value = |name: &str| {
        url.query_pairs()
            .find(|(key, _)| key == name)
            .and_then(|(_, value)| value.parse().ok())
    };
    Some((query_value("hourly")?, query_value("daily")?))
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
        match url.path() {
            path if path == DO_RATE_LIMITER_PATH_CHECK => match parse_check_limits(&url) {
                Some((hourly_limit, daily_limit)) => self.check_limit(hourly_limit, daily_limit).await,
                None => Response::error("hourly and daily must be numbers", HTTP_BAD_REQUEST),
            },
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

    fn check_url(query: &str) -> Url {
        Url::parse(&format!("http://rate-limiter/check{}", query)).unwrap()
    }

    #[test]
    fn check_limits_are_read_from_the_query() {
        assert_eq!(parse_check_limits(&check_url("?hourly=7&daily=30")), Some((7, 30)));
    }

    #[test]
    fn check_limits_need_both_values() {
        assert_eq!(parse_check_limits(&check_url("?hourly=7")), None);
        assert_eq!(parse_check_limits(&check_url("?daily=30")), None);
        assert_eq!(parse_check_limits(&check_url("")), None);
    }

    #[test]
    fn check_limits_must_be_numbers() {
        assert_eq!(parse_check_limits(&check_url("?hourly=many&daily=30")), None);
        assert_eq!(parse_check_limits(&check_url("?hourly=7&daily=-1")), None);
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
