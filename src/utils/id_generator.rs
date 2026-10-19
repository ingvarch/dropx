use base58::{FromBase58, ToBase58};
use uuid::Uuid;
use crate::utils::constants::FILE_ID_UUID_BYTES;

pub fn generate_file_id() -> String {
    let uuid = Uuid::new_v4();
    let bytes = uuid.as_bytes();
    bytes[..FILE_ID_UUID_BYTES].to_base58()
}

pub fn generate_delete_token() -> String {
    let uuid = Uuid::new_v4();
    uuid.as_bytes().to_base58()
}

/// True when `value` decodes from base58 to the byte length `generate_file_id` encodes.
pub fn is_file_id(value: &str) -> bool {
    value.from_base58().is_ok_and(|bytes| bytes.len() == FILE_ID_UUID_BYTES)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_file_id() {
        let id1 = generate_file_id();
        let id2 = generate_file_id();

        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
        assert_ne!(id1, id2);
        assert!(id1.len() >= 8 && id1.len() <= 12);
    }

    #[test]
    fn test_id_uniqueness() {
        let mut ids = std::collections::HashSet::new();
        for _ in 0..1000 {
            let id = generate_file_id();
            assert!(ids.insert(id), "Generated duplicate ID");
        }
    }

    #[test]
    fn test_generate_delete_token() {
        let token1 = generate_delete_token();
        let token2 = generate_delete_token();

        assert!(!token1.is_empty());
        assert!(!token2.is_empty());
        assert_ne!(token1, token2);
        assert!(token1.len() >= 16);
    }

    #[test]
    fn test_delete_token_uniqueness() {
        let mut tokens = std::collections::HashSet::new();
        for _ in 0..1000 {
            let token = generate_delete_token();
            assert!(tokens.insert(token), "Generated duplicate token");
        }
    }

    #[test]
    fn generated_file_ids_are_file_ids() {
        for _ in 0..10_000 {
            let id = generate_file_id();
            assert!(is_file_id(&id), "{id}");
        }
        assert!(is_file_id("11111111"), "all-zero bytes");
    }

    #[test]
    fn paths_that_are_not_file_ids_are_rejected() {
        let delete_token = generate_delete_token();
        for value in ["favicon.ico", "robots.txt", "", "admin", "0OIl0OIl0OI", "abc-def_ghi", delete_token.as_str()] {
            assert!(!is_file_id(value), "{value}");
        }
    }
}
