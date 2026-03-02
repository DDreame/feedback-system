use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::error::AppError;

/// Hash a plaintext password using Argon2id.
///
/// Returns the PHC-format hash string suitable for storing in the database.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {e}")))
}

/// Verify a plaintext password against a stored Argon2 hash.
///
/// Returns `Ok(true)` when the password matches, `Ok(false)` when it does not.
/// Returns `Err` only for malformed hash strings.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Invalid password hash: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_password_produces_non_empty_string() {
        let hash = hash_password("hunter2").expect("should hash");
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2"), "should be PHC format");
    }

    #[test]
    fn correct_password_verifies_successfully() {
        let hash = hash_password("correct-horse-battery-staple").expect("should hash");
        let ok = verify_password("correct-horse-battery-staple", &hash).expect("verify should not error");
        assert!(ok, "correct password should verify");
    }

    #[test]
    fn wrong_password_fails_verification() {
        let hash = hash_password("secret").expect("should hash");
        let ok = verify_password("wrong", &hash).expect("verify should not error");
        assert!(!ok, "wrong password should not verify");
    }

    #[test]
    fn two_hashes_of_same_password_are_different() {
        let h1 = hash_password("same").expect("should hash");
        let h2 = hash_password("same").expect("should hash");
        assert_ne!(h1, h2, "salts must differ between calls");
    }

    #[test]
    fn malformed_hash_returns_error() {
        let result = verify_password("any", "not-a-valid-hash");
        assert!(result.is_err(), "malformed hash should return Err");
    }
}
