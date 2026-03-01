//! JWT access-token validation.

use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::Deserialize;
#[cfg(any(feature = "USE_ONLY_IN_AUTH_SERVICE", test))]
use serde::Serialize;
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

/// JWT claims payload shared by token creation (auth service) and validation (gateway).
///
/// # Fields
///
/// | Field | JWT claim | Rust type | Meaning |
/// |-------|-----------|-----------|---------|
/// | `sub` | `sub` | UUID string | user ID |
/// | `role` | custom | `u8` wire value | see [`madome_domain::user::UserRole`] |
/// | `exp` | `exp` | seconds since epoch | token expiration |
///
/// # Feature gate
///
/// [`Deserialize`] is always available — all consumers validate tokens.
/// [`Serialize`] requires the **`USE_ONLY_IN_AUTH_SERVICE`** cargo feature. Only the auth
/// service enables it because it is the sole token issuer.
#[derive(Debug, Deserialize)]
#[cfg_attr(any(feature = "USE_ONLY_IN_AUTH_SERVICE", test), derive(Serialize))]
pub struct JwtClaims {
    /// User ID (UUID string).
    pub sub: String,
    /// User role as `u8` wire value.
    pub role: u8,
    /// Expiration timestamp (seconds since UNIX epoch).
    pub exp: u64,
}

// ── Core decode (private) ────────────────────────────────────────────────

/// Decode and validate a JWT, returning raw claims.
///
/// Validation: HS256, exp checked, required claims: `exp` + `sub`.
/// Default leeway = 60s — tolerates clock skew between services.
/// Same library + version as legacy; matches legacy behavior.
fn decode_jwt(token: &str, secret: &str) -> Result<JwtClaims, AuthError> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true;
    validation.required_spec_claims.clear();
    validation.set_required_spec_claims(&["exp", "sub"]);

    let data = decode::<JwtClaims>(
        token,
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

    Ok(data.claims)
}

// ── Public: all consumers ────────────────────────────────────────────────

/// Validate an access-token cookie value, returning parsed identity.
///
/// This is the primary public API for token validation. Gateway calls this
/// on every request to extract user identity from the JWT cookie.
pub fn validate_access_token(cookie_value: &str, secret: &str) -> Result<TokenInfo, AuthError> {
    let claims = decode_jwt(cookie_value, secret)?;
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

// ── Feature-gated: auth service only ─────────────────────────────────────

/// Validate a token and return raw JWT claims.
///
/// Used by the auth service's refresh flow — validates the refresh token,
/// then looks up the user from the `sub` claim to issue new tokens.
///
/// Requires the `USE_ONLY_IN_AUTH_SERVICE` feature. Only the auth service
/// should call this directly; all other consumers use [`validate_access_token`].
#[cfg(any(feature = "USE_ONLY_IN_AUTH_SERVICE", test))]
pub fn validate_token(token: &str, secret: &str) -> Result<JwtClaims, AuthError> {
    decode_jwt(token, secret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{EncodingKey, Header, encode};

    const TEST_SECRET: &str = "test-secret-key-for-unit-tests";

    fn make_token(sub: &str, role: u8, exp: u64) -> String {
        let claims = JwtClaims {
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
