use argon2::password_hash::rand_core::{OsRng, RngCore};

const PREFIX: &str = "proj_";
/// Number of random bytes → 32 bytes → 64 hex chars → key length = 5 + 64 = 69 chars
const RANDOM_BYTES: usize = 32;

/// Generate a cryptographically secure API key with the `proj_` prefix.
///
/// Format: `proj_<64 lowercase hex chars>`
pub fn generate_api_key() -> String {
    let mut bytes = [0u8; RANDOM_BYTES];
    OsRng.fill_bytes(&mut bytes);
    format!("{}{}", PREFIX, hex::encode(bytes))
}

/// Return true if the string looks like a valid API key (correct prefix + length).
pub fn is_valid_api_key_format(key: &str) -> bool {
    key.starts_with(PREFIX) && key.len() == PREFIX.len() + RANDOM_BYTES * 2
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn api_key_has_proj_prefix() {
        let key = generate_api_key();
        assert!(key.starts_with("proj_"), "key={key}");
    }

    #[test]
    fn api_key_has_correct_length() {
        let key = generate_api_key();
        // "proj_" (5) + 64 hex chars
        assert_eq!(key.len(), 5 + 64, "key={key}");
    }

    #[test]
    fn api_key_suffix_is_hex() {
        let key = generate_api_key();
        let suffix = &key["proj_".len()..];
        assert!(
            suffix.chars().all(|c| c.is_ascii_hexdigit()),
            "suffix not hex: {suffix}"
        );
    }

    #[test]
    fn api_key_is_unique_across_calls() {
        let keys: HashSet<String> = (0..20).map(|_| generate_api_key()).collect();
        assert_eq!(keys.len(), 20, "generated duplicate keys");
    }

    #[test]
    fn is_valid_api_key_format_accepts_valid_key() {
        let key = generate_api_key();
        assert!(is_valid_api_key_format(&key));
    }

    #[test]
    fn is_valid_api_key_format_rejects_missing_prefix() {
        assert!(!is_valid_api_key_format("abc123"));
    }

    #[test]
    fn is_valid_api_key_format_rejects_wrong_length() {
        assert!(!is_valid_api_key_format("proj_tooshort"));
    }
}
