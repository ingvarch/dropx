use sha2::{Digest, Sha256};
use hex;
use worker::{Request, Env, Result, Response};
use crate::utils::response::unauthorized;
use crate::utils::constants::{HASH_TRUNCATE_BYTES, HEADER_ADMIN_TOKEN};

pub fn hash_identifier(identifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(identifier.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..HASH_TRUNCATE_BYTES])
}

pub fn generate_api_key() -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("tfr_{}", hex::encode(uuid.as_bytes()))
}

pub fn verify_admin_token(req: &Request, env: &Env) -> Result<bool> {
    let token = req.headers().get(HEADER_ADMIN_TOKEN)?;
    let admin_token = env.var("ADMIN_TOKEN")?.to_string();

    Ok(token.as_deref() == Some(admin_token.as_str()))
}

pub fn require_admin_token(req: &Request, env: &Env) -> Option<Result<Response>> {
    match verify_admin_token(req, env) {
        Ok(true) => None,
        Ok(false) => Some(unauthorized("Unauthorized", "Invalid or missing admin token")),
        Err(e) => Some(Err(e)),
    }
}

#[macro_export]
macro_rules! require_admin {
    ($req:expr, $env:expr) => {
        if let Some(response) = $crate::utils::auth::require_admin_token($req, $env) {
            return response;
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_identifier_consistency() {
        let id = "192.168.1.1";
        let hash1 = hash_identifier(id);
        let hash2 = hash_identifier(id);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_identifier_uniqueness() {
        let hash1 = hash_identifier("192.168.1.1");
        let hash2 = hash_identifier("192.168.1.2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_generate_api_key() {
        let key = generate_api_key();
        assert!(key.starts_with("tfr_"));
        assert!(key.len() > 20);
    }

    #[test]
    fn test_generate_api_key_uniqueness() {
        let key1 = generate_api_key();
        let key2 = generate_api_key();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_hash_identifier_length() {
        let hash = hash_identifier("test");
        assert_eq!(hash.len(), HASH_TRUNCATE_BYTES * 2);
    }

    #[test]
    fn test_hash_identifier_hex_format() {
        let hash = hash_identifier("test");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_identifier_empty_string() {
        let hash1 = hash_identifier("");
        let hash2 = hash_identifier("");
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), HASH_TRUNCATE_BYTES * 2);
    }

    #[test]
    fn test_hash_identifier_different_inputs() {
        let hash1 = hash_identifier("user@example.com");
        let hash2 = hash_identifier("admin@example.com");
        let hash3 = hash_identifier("test@example.com");

        assert_ne!(hash1, hash2);
        assert_ne!(hash2, hash3);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_generate_api_key_format() {
        let key = generate_api_key();

        assert!(key.starts_with("tfr_"));

        let hex_part = &key[4..];
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));

        assert_eq!(hex_part.len(), 32);
    }

    #[test]
    fn test_generate_api_key_length() {
        let key = generate_api_key();
        assert_eq!(key.len(), 36);
    }

    #[test]
    fn test_hash_identifier_special_characters() {
        let hash1 = hash_identifier("test!@#$%");
        let hash2 = hash_identifier("test!@#$%");
        assert_eq!(hash1, hash2);

        let hash3 = hash_identifier("test");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_identifier_no_input_leakage() {
        let sensitive = "secret_password_123";
        let hash = hash_identifier(sensitive);

        assert!(!hash.contains("secret"));
        assert!(!hash.contains("password"));
        assert!(!hash.contains("123"));
    }
}
