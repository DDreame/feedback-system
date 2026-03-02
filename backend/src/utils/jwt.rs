use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

const ALGORITHM: Algorithm = Algorithm::HS256;

/// Token type discriminator stored inside the JWT claims.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenKind {
    Access,
    Refresh,
}

/// Claims encoded into every JWT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — the developer's UUID.
    pub sub: String,
    /// Expiry (UNIX timestamp, seconds).
    pub exp: i64,
    /// Issued-at (UNIX timestamp, seconds).
    pub iat: i64,
    /// Whether this is an access or refresh token.
    pub kind: TokenKind,
}

/// Generate a signed JWT.
///
/// * `developer_id` — the developer UUID to embed as `sub`
/// * `secret` — the HMAC-SHA256 signing secret
/// * `kind` — access or refresh token
/// * `expiry_secs` — how many seconds until the token expires
pub fn generate_token(
    developer_id: Uuid,
    secret: &str,
    kind: TokenKind,
    expiry_secs: u64,
) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = (now + Duration::seconds(expiry_secs as i64)).timestamp();

    let claims = Claims {
        sub: developer_id.to_string(),
        exp,
        iat: now.timestamp(),
        kind,
    };

    encode(
        &Header::new(ALGORITHM),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to generate token: {e}")))
}

/// Validate a JWT and return its claims.
///
/// Returns `AppError::Unauthorized` for any validation failure (expired, bad signature, …).
/// Optionally asserts that the token kind matches `expected_kind`.
pub fn validate_token(
    token: &str,
    secret: &str,
    expected_kind: Option<TokenKind>,
) -> Result<Claims, AppError> {
    let mut validation = Validation::new(ALGORITHM);
    validation.leeway = 0; // No clock skew allowed in tests

    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation)
        .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;

    if let Some(expected) = expected_kind {
        if data.claims.kind != expected {
            return Err(AppError::Unauthorized("Wrong token kind".to_string()));
        }
    }

    Ok(data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret-at-least-32-chars-long!!";

    fn dev_id() -> Uuid {
        Uuid::now_v7()
    }

    #[test]
    fn access_token_contains_correct_sub() {
        let id = dev_id();
        let token = generate_token(id, SECRET, TokenKind::Access, 3600).unwrap();
        let claims = validate_token(&token, SECRET, None).unwrap();
        assert_eq!(claims.sub, id.to_string());
    }

    #[test]
    fn access_token_kind_is_access() {
        let token = generate_token(dev_id(), SECRET, TokenKind::Access, 3600).unwrap();
        let claims = validate_token(&token, SECRET, None).unwrap();
        assert_eq!(claims.kind, TokenKind::Access);
    }

    #[test]
    fn refresh_token_kind_is_refresh() {
        let token = generate_token(dev_id(), SECRET, TokenKind::Refresh, 604800).unwrap();
        let claims = validate_token(&token, SECRET, None).unwrap();
        assert_eq!(claims.kind, TokenKind::Refresh);
    }

    #[test]
    fn wrong_secret_fails_validation() {
        let token = generate_token(dev_id(), SECRET, TokenKind::Access, 3600).unwrap();
        let result = validate_token(&token, "wrong-secret", None);
        assert!(matches!(result, Err(AppError::Unauthorized(_))));
    }

    #[test]
    fn wrong_kind_returns_unauthorized() {
        let token = generate_token(dev_id(), SECRET, TokenKind::Refresh, 3600).unwrap();
        let result = validate_token(&token, SECRET, Some(TokenKind::Access));
        assert!(matches!(result, Err(AppError::Unauthorized(_))));
    }

    #[test]
    fn expired_token_fails_validation() {
        // Issue a token that expired 1 second in the past
        let id = dev_id();
        let now = Utc::now();
        let claims = Claims {
            sub: id.to_string(),
            exp: (now - Duration::seconds(1)).timestamp(),
            iat: (now - Duration::seconds(2)).timestamp(),
            kind: TokenKind::Access,
        };
        let token = jsonwebtoken::encode(
            &Header::new(ALGORITHM),
            &claims,
            &EncodingKey::from_secret(SECRET.as_bytes()),
        )
        .unwrap();

        let result = validate_token(&token, SECRET, None);
        assert!(matches!(result, Err(AppError::Unauthorized(_))), "expired token must be rejected");
    }

    #[test]
    fn malformed_token_fails_validation() {
        let result = validate_token("this.is.not.a.jwt", SECRET, None);
        assert!(matches!(result, Err(AppError::Unauthorized(_))));
    }

    #[test]
    fn token_exp_is_in_the_future() {
        let token = generate_token(dev_id(), SECRET, TokenKind::Access, 3600).unwrap();
        let claims = validate_token(&token, SECRET, None).unwrap();
        assert!(claims.exp > Utc::now().timestamp());
    }
}
