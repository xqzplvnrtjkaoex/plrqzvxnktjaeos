//! JWT access-token validation.

use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User identity extracted from a validated access token.
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub user_id: Uuid,
    pub user_role: u8,
    pub access_token_exp: u64,
}

/// Errors returned by [`validate_access_token`].
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid signature")]
    InvalidSignature,
    #[error("token expired")]
    Expired,
    #[error("malformed token")]
    Malformed,
}

/// JWT claims struct used for encoding/decoding.
#[derive(Debug, Serialize, Deserialize)]
struct AccessClaims {
    /// User ID as string.
    sub: String,
    /// User role as u8 wire value.
    role: u8,
    /// Expiration timestamp (seconds since epoch).
    exp: u64,
}

/// Validate an access-token cookie value. Pure function — no axum/tower dependency.
pub fn validate_access_token(cookie_value: &str, secret: &str) -> Result<TokenInfo, AuthError> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true;
    validation.required_spec_claims.clear();
    validation.set_required_spec_claims(&["exp", "sub"]);

    let token_data = decode::<AccessClaims>(
        cookie_value,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::Expired,
        jsonwebtoken::errors::ErrorKind::InvalidSignature
        | jsonwebtoken::errors::ErrorKind::InvalidEcdsaKey
        | jsonwebtoken::errors::ErrorKind::InvalidRsaKey(_) => AuthError::InvalidSignature,
        _ => AuthError::Malformed,
    })?;

    let claims = token_data.claims;

    let user_id = claims
        .sub
        .parse::<Uuid>()
        .map_err(|_| AuthError::Malformed)?;

    Ok(TokenInfo {
        user_id,
        user_role: claims.role,
        access_token_exp: claims.exp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{EncodingKey, Header, encode};

    const TEST_SECRET: &str = "test-secret-key-for-unit-tests";

    fn make_token(sub: &str, role: u8, exp: u64) -> String {
        let claims = AccessClaims {
            sub: sub.to_string(),
            role,
            exp,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(TEST_SECRET.as_bytes()),
        )
        .unwrap()
    }

    fn future_exp() -> u64 {
        // 1 hour from now
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600
    }

    #[test]
    fn should_validate_valid_token() {
        let user_id = Uuid::new_v4();
        let token = make_token(&user_id.to_string(), 1, future_exp());

        let info = validate_access_token(&token, TEST_SECRET).unwrap();
        assert_eq!(info.user_id, user_id);
        assert_eq!(info.user_role, 1);
    }

    #[test]
    fn should_reject_expired_token() {
        let user_id = Uuid::new_v4();
        // exp in the past
        let token = make_token(&user_id.to_string(), 0, 1_000_000);

        let err = validate_access_token(&token, TEST_SECRET).unwrap_err();
        assert!(matches!(err, AuthError::Expired));
    }

    #[test]
    fn should_reject_wrong_secret() {
        let user_id = Uuid::new_v4();
        let token = make_token(&user_id.to_string(), 0, future_exp());

        let err = validate_access_token(&token, "wrong-secret").unwrap_err();
        assert!(matches!(err, AuthError::InvalidSignature));
    }

    #[test]
    fn should_reject_malformed_token() {
        let err = validate_access_token("not-a-jwt", TEST_SECRET).unwrap_err();
        assert!(matches!(err, AuthError::Malformed));
    }
}
