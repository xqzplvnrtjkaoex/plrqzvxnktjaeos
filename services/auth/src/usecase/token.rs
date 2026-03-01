use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use madome_auth_types::cookie::{ACCESS_TOKEN_EXP, REFRESH_TOKEN_EXP};

use crate::domain::repository::{AuthCodeRepository, UserRepository};
use crate::domain::types::AuthUser;
use crate::error::AuthServiceError;

/// JWT claims for both access and refresh tokens.
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub role: u8,
    pub exp: u64,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX epoch")
        .as_secs()
}

pub fn issue_access_token(
    user: &AuthUser,
    secret: &str,
) -> Result<(String, u64), AuthServiceError> {
    let exp = now_secs() + ACCESS_TOKEN_EXP;
    let claims = TokenClaims {
        sub: user.id.to_string(),
        role: user.role,
        exp,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AuthServiceError::Internal(e.into()))?;
    Ok((token, exp))
}

pub fn issue_refresh_token(user: &AuthUser, secret: &str) -> Result<String, AuthServiceError> {
    let exp = now_secs() + REFRESH_TOKEN_EXP;
    let claims = TokenClaims {
        sub: user.id.to_string(),
        role: user.role,
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AuthServiceError::Internal(e.into()))
}

/// Validate a token and return its claims. Used for the refresh flow.
pub fn validate_token(token: &str, secret: &str) -> Result<TokenClaims, AuthServiceError> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true;
    validation.required_spec_claims.clear();
    validation.set_required_spec_claims(&["exp", "sub"]);

    let data = decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|_| AuthServiceError::InvalidRefreshToken)?;

    Ok(data.claims)
}

// ── CreateToken (login) ───────────────────────────────────────────────────────

pub struct CreateTokenInput {
    pub email: String,
    pub code: String,
}

#[derive(Debug)]
pub struct CreateTokenOutput {
    pub user: AuthUser,
    pub access_token: String,
    pub access_token_exp: u64,
    pub refresh_token: String,
}

pub struct CreateTokenUseCase<U: UserRepository, A: AuthCodeRepository> {
    pub users: U,
    pub auth_codes: A,
    pub jwt_secret: String,
}

impl<U: UserRepository, A: AuthCodeRepository> CreateTokenUseCase<U, A> {
    pub async fn execute(
        &self,
        input: CreateTokenInput,
    ) -> Result<CreateTokenOutput, AuthServiceError> {
        let user = self
            .users
            .find_by_email(&input.email)
            .await?
            .ok_or(AuthServiceError::UserNotFound)?;

        let auth_code = self
            .auth_codes
            .find_valid(user.id, &input.code)
            .await?
            .ok_or(AuthServiceError::InvalidAuthcode)?;

        self.auth_codes.mark_used(auth_code.id).await?;

        let (access_token, access_token_exp) = issue_access_token(&user, &self.jwt_secret)?;
        let refresh_token = issue_refresh_token(&user, &self.jwt_secret)?;

        Ok(CreateTokenOutput {
            user,
            access_token,
            access_token_exp,
            refresh_token,
        })
    }
}

// ── RefreshToken ─────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct RefreshTokenOutput {
    pub user_id: Uuid,
    pub user_role: u8,
    pub access_token: String,
    pub access_token_exp: u64,
    pub refresh_token: String,
}

pub struct RefreshTokenUseCase<U: UserRepository> {
    pub users: U,
    pub jwt_secret: String,
}

impl<U: UserRepository> RefreshTokenUseCase<U> {
    pub async fn execute(
        &self,
        refresh_token_value: &str,
    ) -> Result<RefreshTokenOutput, AuthServiceError> {
        // Validate refresh token (sig + exp); expired access token is irrelevant here.
        let claims = validate_token(refresh_token_value, &self.jwt_secret)?;

        let user_id = claims
            .sub
            .parse::<Uuid>()
            .map_err(|_| AuthServiceError::InvalidRefreshToken)?;

        let user = self
            .users
            .find_by_id(user_id)
            .await?
            .ok_or(AuthServiceError::InvalidRefreshToken)?;

        let (access_token, access_token_exp) = issue_access_token(&user, &self.jwt_secret)?;
        let refresh_token = issue_refresh_token(&user, &self.jwt_secret)?;

        Ok(RefreshTokenOutput {
            user_id: user.id,
            user_role: user.role,
            access_token,
            access_token_exp,
            refresh_token,
        })
    }
}
