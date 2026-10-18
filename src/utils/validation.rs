use crate::models::api_key::ApiKeyTier;
use crate::utils::constants::{
    STANDARD_MAX_FILE_SIZE, PREMIUM_MAX_FILE_SIZE,
};

#[allow(dead_code)]
pub fn is_file_size_valid(size: usize, tier: Option<&ApiKeyTier>) -> bool {
    let max_size = match tier {
        Some(ApiKeyTier::Standard) => STANDARD_MAX_FILE_SIZE,
        Some(ApiKeyTier::Premium) => PREMIUM_MAX_FILE_SIZE,
        None => STANDARD_MAX_FILE_SIZE,
    };
    size <= max_size
}

#[allow(dead_code)]
pub fn get_max_file_size(tier: Option<&ApiKeyTier>) -> usize {
    match tier {
        Some(ApiKeyTier::Standard) => STANDARD_MAX_FILE_SIZE,
        Some(ApiKeyTier::Premium) => PREMIUM_MAX_FILE_SIZE,
        None => STANDARD_MAX_FILE_SIZE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_size_valid_anonymous_within_limit() {
        let size = 50 * 1024 * 1024;
        assert!(is_file_size_valid(size, None));
    }

    #[test]
    fn test_file_size_valid_anonymous_at_limit() {
        let size = STANDARD_MAX_FILE_SIZE;
        assert!(is_file_size_valid(size, None));
    }

    #[test]
    fn test_file_size_invalid_anonymous_exceeds_limit() {
        let size = STANDARD_MAX_FILE_SIZE + 1;
        assert!(!is_file_size_valid(size, None));
    }

    #[test]
    fn test_file_size_valid_standard_within_limit() {
        let size = 50 * 1024 * 1024;
        assert!(is_file_size_valid(size, Some(&ApiKeyTier::Standard)));
    }

    #[test]
    fn test_file_size_valid_standard_at_limit() {
        let size = STANDARD_MAX_FILE_SIZE;
        assert!(is_file_size_valid(size, Some(&ApiKeyTier::Standard)));
    }

    #[test]
    fn test_file_size_invalid_standard_exceeds_limit() {
        let size = STANDARD_MAX_FILE_SIZE + 1;
        assert!(!is_file_size_valid(size, Some(&ApiKeyTier::Standard)));
    }

    #[test]
    fn test_file_size_valid_premium_within_limit() {
        let size = 200 * 1024 * 1024;
        assert!(is_file_size_valid(size, Some(&ApiKeyTier::Premium)));
    }

    #[test]
    fn test_file_size_valid_premium_at_limit() {
        let size = PREMIUM_MAX_FILE_SIZE;
        assert!(is_file_size_valid(size, Some(&ApiKeyTier::Premium)));
    }

    #[test]
    fn test_file_size_invalid_premium_exceeds_limit() {
        let size = PREMIUM_MAX_FILE_SIZE + 1;
        assert!(!is_file_size_valid(size, Some(&ApiKeyTier::Premium)));
    }

    #[test]
    fn test_file_size_zero_is_valid() {
        assert!(is_file_size_valid(0, None));
        assert!(is_file_size_valid(0, Some(&ApiKeyTier::Standard)));
        assert!(is_file_size_valid(0, Some(&ApiKeyTier::Premium)));
    }

    #[test]
    fn test_file_size_one_byte_is_valid() {
        assert!(is_file_size_valid(1, None));
        assert!(is_file_size_valid(1, Some(&ApiKeyTier::Standard)));
        assert!(is_file_size_valid(1, Some(&ApiKeyTier::Premium)));
    }

    #[test]
    fn test_get_max_file_size_anonymous() {
        assert_eq!(get_max_file_size(None), STANDARD_MAX_FILE_SIZE);
    }

    #[test]
    fn test_get_max_file_size_standard() {
        assert_eq!(
            get_max_file_size(Some(&ApiKeyTier::Standard)),
            STANDARD_MAX_FILE_SIZE
        );
    }

    #[test]
    fn test_get_max_file_size_premium() {
        assert_eq!(
            get_max_file_size(Some(&ApiKeyTier::Premium)),
            PREMIUM_MAX_FILE_SIZE
        );
    }

    #[test]
    fn test_standard_file_exceeds_limit_for_premium() {
        let size = STANDARD_MAX_FILE_SIZE + 1024;
        assert!(!is_file_size_valid(size, Some(&ApiKeyTier::Standard)));
        assert!(is_file_size_valid(size, Some(&ApiKeyTier::Premium)));
    }

    #[test]
    fn test_boundary_conditions_file_sizes() {
        let standard_max = STANDARD_MAX_FILE_SIZE;
        let premium_max = PREMIUM_MAX_FILE_SIZE;

        assert!(is_file_size_valid(standard_max - 1, Some(&ApiKeyTier::Standard)));
        assert!(is_file_size_valid(standard_max, Some(&ApiKeyTier::Standard)));
        assert!(!is_file_size_valid(standard_max + 1, Some(&ApiKeyTier::Standard)));

        assert!(is_file_size_valid(premium_max - 1, Some(&ApiKeyTier::Premium)));
        assert!(is_file_size_valid(premium_max, Some(&ApiKeyTier::Premium)));
        assert!(!is_file_size_valid(premium_max + 1, Some(&ApiKeyTier::Premium)));
    }

    #[test]
    fn test_tier_upgrade_allows_larger_files() {
        let size = 200 * 1024 * 1024;

        assert!(!is_file_size_valid(size, Some(&ApiKeyTier::Standard)));
        assert!(is_file_size_valid(size, Some(&ApiKeyTier::Premium)));
    }
}
