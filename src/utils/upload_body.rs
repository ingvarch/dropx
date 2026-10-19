use crate::utils::constants::MAX_BUFFERED_UPLOAD_SIZE;

#[derive(Debug, PartialEq)]
pub enum BodyPlan {
    Stream,
    Buffer,
}

#[derive(Debug, PartialEq)]
pub enum BodyRejection {
    TooLarge,
    LengthRequired,
}

/// A valid `Content-Length` streams or is rejected up front. Anything else is buffered up to a fixed limit.
pub fn body_plan(content_length: Option<&str>, max_size: usize) -> Result<BodyPlan, BodyRejection> {
    // u64: usize is 32-bit on wasm32, and Enterprise zones allow bodies over 4 GB.
    match content_length.and_then(|v| v.parse::<u64>().ok()) {
        Some(length) if length > max_size as u64 => Err(BodyRejection::TooLarge),
        Some(_) => Ok(BodyPlan::Stream),
        None => Ok(BodyPlan::Buffer),
    }
}

/// Why a buffered body of `total` bytes is refused, if it is.
pub fn buffer_overflow(total: usize, max_size: usize) -> Option<BodyRejection> {
    if total <= max_size.min(MAX_BUFFERED_UPLOAD_SIZE) {
        None
    } else if total > max_size {
        Some(BodyRejection::TooLarge)
    } else {
        Some(BodyRejection::LengthRequired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::constants::{ANON_MAX_FILE_SIZE, STANDARD_MAX_FILE_SIZE};

    #[test]
    fn a_length_within_the_limit_streams() {
        assert_eq!(body_plan(Some("1000"), ANON_MAX_FILE_SIZE), Ok(BodyPlan::Stream));
        assert_eq!(body_plan(Some("0"), ANON_MAX_FILE_SIZE), Ok(BodyPlan::Stream));
    }

    #[test]
    fn a_length_equal_to_the_limit_streams() {
        let max = ANON_MAX_FILE_SIZE;
        assert_eq!(body_plan(Some(&max.to_string()), max), Ok(BodyPlan::Stream));
    }

    #[test]
    fn a_length_over_the_limit_is_too_large() {
        let over = (ANON_MAX_FILE_SIZE + 1).to_string();
        assert_eq!(body_plan(Some(&over), ANON_MAX_FILE_SIZE), Err(BodyRejection::TooLarge));
    }

    #[test]
    fn a_length_over_4_gb_is_too_large() {
        assert_eq!(body_plan(Some("5000000000"), ANON_MAX_FILE_SIZE), Err(BodyRejection::TooLarge));
    }

    #[test]
    fn a_missing_or_invalid_length_buffers() {
        for value in [None, Some("abc"), Some("-5"), Some("")] {
            assert_eq!(
                body_plan(value, ANON_MAX_FILE_SIZE),
                Ok(BodyPlan::Buffer),
                "{value:?}"
            );
        }
    }

    #[test]
    fn a_buffered_body_within_the_limit_is_accepted() {
        assert_eq!(buffer_overflow(MAX_BUFFERED_UPLOAD_SIZE, STANDARD_MAX_FILE_SIZE), None);
        assert_eq!(buffer_overflow(10, ANON_MAX_FILE_SIZE), None);
    }

    #[test]
    fn a_buffered_body_over_a_smaller_tier_limit_is_too_large() {
        let tier_max = MAX_BUFFERED_UPLOAD_SIZE / 2;
        assert_eq!(buffer_overflow(tier_max + 1, tier_max), Some(BodyRejection::TooLarge));
    }

    #[test]
    fn a_buffered_body_over_the_buffer_limit_needs_a_length() {
        assert_eq!(
            buffer_overflow(MAX_BUFFERED_UPLOAD_SIZE + 1, STANDARD_MAX_FILE_SIZE),
            Some(BodyRejection::LengthRequired)
        );
    }

    #[test]
    fn at_the_buffer_limit_the_tier_limit_decides() {
        assert_eq!(
            buffer_overflow(ANON_MAX_FILE_SIZE + 1, ANON_MAX_FILE_SIZE),
            Some(BodyRejection::TooLarge)
        );
    }
}
