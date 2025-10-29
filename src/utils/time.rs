use chrono::Utc;

pub fn current_timestamp() -> i64 {
    Utc::now().timestamp()
}

#[allow(dead_code)]
pub fn calculate_expiration(current: i64, retention_days: i64) -> i64 {
    current + (retention_days * crate::utils::constants::SECONDS_PER_DAY)
}

#[allow(dead_code)]
pub fn format_expires_in(retention_days: i64) -> String {
    if retention_days == 1 {
        "1d".to_string()
    } else {
        format!("{}d", retention_days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::constants::SECONDS_PER_DAY;

    #[test]
    fn test_current_timestamp_returns_positive() {
        let timestamp = current_timestamp();
        assert!(timestamp > 0, "Timestamp should be positive");
    }

    #[test]
    fn test_current_timestamp_is_reasonable() {
        let timestamp = current_timestamp();
        let year_2020 = 1577836800;
        let year_2030 = 1893456000;
        assert!(
            timestamp > year_2020 && timestamp < year_2030,
            "Timestamp should be between 2020 and 2030"
        );
    }

    #[test]
    fn test_current_timestamp_increases() {
        let t1 = current_timestamp();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let t2 = current_timestamp();
        assert!(t2 >= t1, "Timestamp should increase or stay the same");
    }

    #[test]
    fn test_calculate_expiration_7_days() {
        let current = 1000000;
        let retention_days = 7;
        let expected = 1000000 + (7 * SECONDS_PER_DAY);
        assert_eq!(calculate_expiration(current, retention_days), expected);
    }

    #[test]
    fn test_calculate_expiration_30_days() {
        let current = 1000000;
        let retention_days = 30;
        let expected = 1000000 + (30 * SECONDS_PER_DAY);
        assert_eq!(calculate_expiration(current, retention_days), expected);
    }

    #[test]
    fn test_calculate_expiration_1_day() {
        let current = 1000000;
        let retention_days = 1;
        let expected = 1000000 + SECONDS_PER_DAY;
        assert_eq!(calculate_expiration(current, retention_days), expected);
    }

    #[test]
    fn test_calculate_expiration_zero_current() {
        let current = 0;
        let retention_days = 7;
        let expected = 7 * SECONDS_PER_DAY;
        assert_eq!(calculate_expiration(current, retention_days), expected);
    }

    #[test]
    fn test_calculate_expiration_overflow_safety() {
        let current = i64::MAX - (10 * SECONDS_PER_DAY);
        let retention_days = 5;
        let result = calculate_expiration(current, retention_days);
        assert!(result > current, "Should not overflow for reasonable values");
    }

    #[test]
    fn test_format_expires_in_single_day() {
        assert_eq!(format_expires_in(1), "1d");
    }

    #[test]
    fn test_format_expires_in_7_days() {
        assert_eq!(format_expires_in(7), "7d");
    }

    #[test]
    fn test_format_expires_in_30_days() {
        assert_eq!(format_expires_in(30), "30d");
    }

    #[test]
    fn test_format_expires_in_100_days() {
        assert_eq!(format_expires_in(100), "100d");
    }

    #[test]
    fn test_expiration_boundary_exactly_at_expiry() {
        let current = 1000000;
        let retention_days = 7;
        let expires_at = calculate_expiration(current, retention_days);

        assert!(current < expires_at);
        assert_eq!(expires_at, current + (7 * SECONDS_PER_DAY));
    }

    #[test]
    fn test_expiration_before_and_after() {
        let current = 1000000;
        let retention_days = 7;
        let expires_at = calculate_expiration(current, retention_days);

        let before_expiry = expires_at - 1;
        let at_expiry = expires_at;
        let after_expiry = expires_at + 1;

        assert!(before_expiry < expires_at);
        assert_eq!(at_expiry, expires_at);
        assert!(after_expiry > expires_at);
    }
}
