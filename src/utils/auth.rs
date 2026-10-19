use hmac::{Hmac, KeyInit, Mac};
use hmac::digest::InvalidLength;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use worker::{Request, Env, Result, Response};
use crate::utils::response::unauthorized;
use crate::utils::constants::{ENV_IP_HASH_KEY, HASH_TRUNCATE_BYTES, HEADER_ADMIN_TOKEN, ERR_UNAUTHORIZED};

pub fn hash_secret(value: &str) -> String {
    hex::encode(Sha256::digest(value.as_bytes()))
}

pub fn hash_ip(ip: &str, key: &str) -> std::result::Result<String, InvalidLength> {
    let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(key.as_bytes())?;
    mac.update(ip.as_bytes());
    Ok(hex::encode(&mac.finalize().into_bytes()[..HASH_TRUNCATE_BYTES]))
}

pub fn client_id(env: &Env, ip: &str) -> Result<String> {
    let key = env.secret(ENV_IP_HASH_KEY)?.to_string();
    hash_ip(ip, &key).map_err(|e| worker::Error::RustError(e.to_string()))
}

pub fn generate_api_key() -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("tfr_{}", hex::encode(uuid.as_bytes()))
}

/// Hashing first makes the comparison independent of the secret's length.
pub fn secrets_match(provided: &str, expected: &str) -> bool {
    let provided_digest = Sha256::digest(provided.as_bytes());
    let expected_digest = Sha256::digest(expected.as_bytes());
    provided_digest.as_slice().ct_eq(expected_digest.as_slice()).into()
}

pub fn verify_admin_token(req: &Request, env: &Env) -> Result<bool> {
    let token = req.headers().get(HEADER_ADMIN_TOKEN)?;
    let admin_token = env.var("ADMIN_TOKEN")?.to_string();

    Ok(token.is_some_and(|provided| secrets_match(&provided, &admin_token)))
}

pub fn require_admin_token(req: &Request, env: &Env) -> Option<Result<Response>> {
    match verify_admin_token(req, env) {
        Ok(true) => None,
        Ok(false) => Some(unauthorized(ERR_UNAUTHORIZED, "Invalid or missing admin token")),
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
    fn test_secrets_match_equal() {
        assert!(secrets_match("s3cret-token", "s3cret-token"));
    }

    #[test]
    fn test_secrets_match_same_length_different() {
        assert!(!secrets_match("s3cret-tokeX", "s3cret-token"));
    }

    #[test]
    fn test_secrets_match_different_length() {
        assert!(!secrets_match("s3cret-token-longer", "s3cret-token"));
    }

    #[test]
    fn test_secrets_match_empty_provided() {
        assert!(!secrets_match("", "s3cret-token"));
    }

    #[test]
    fn test_secrets_match_prefix_of_secret() {
        assert!(!secrets_match("s3cret", "s3cret-token"));
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
    fn hash_secret_is_sha256_hex() {
        assert_eq!(
            hash_secret("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn hash_ip_is_truncated_hmac_sha256() {
        // RFC 4231 test case 2, first 16 bytes
        assert_eq!(
            hash_ip("what do ya want for nothing?", "Jefe").unwrap(),
            "5bdcc146bf60754e6a042426089575c7"
        );
    }

    #[test]
    fn hash_ip_depends_on_the_key() {
        assert_ne!(hash_ip("192.168.1.1", "key one").unwrap(), hash_ip("192.168.1.1", "key two").unwrap());
    }
}
