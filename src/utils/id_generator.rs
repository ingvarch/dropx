use base58::ToBase58;
use uuid::Uuid;
use crate::utils::constants::FILE_ID_UUID_BYTES;

pub fn generate_file_id() -> String {
    let uuid = Uuid::new_v4();
    let bytes = uuid.as_bytes();
    bytes[..FILE_ID_UUID_BYTES].to_base58()
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
}
