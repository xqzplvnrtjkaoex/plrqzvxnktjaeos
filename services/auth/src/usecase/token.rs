use jsonwebtoken::{EncodingKey, Header, encode};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use madome_auth_types::cookie::{ACCESS_TOKEN_EXP, REFRESH_TOKEN_EXP};
use madome_auth_types::token::{JwtClaims, validate_token};

use crate::domain::repository::{AuthCodeRepository, UserPort};
use crate::domain::types::AuthUser;
use crate::error::AuthServiceError;

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
    let claims = JwtClaims {
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
    let claims = JwtClaims {
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

pub struct CreateTokenUseCase<U, A>
where
    U: UserPort,
    A: AuthCodeRepository,
{
    pub users: U,
    pub auth_codes: A,
    pub jwt_secret: String,
}

impl<U, A> CreateTokenUseCase<U, A>
where
    U: UserPort,
    A: AuthCodeRepository,
{
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

pub struct RefreshTokenUseCase<U>
where
    U: UserPort,
{
    pub users: U,
    pub jwt_secret: String,
}

impl<U> RefreshTokenUseCase<U>
where
    U: UserPort,
{
    pub async fn execute(
        &self,
        refresh_token_value: &str,
    ) -> Result<RefreshTokenOutput, AuthServiceError> {
        // validate_token returns detailed AuthError (Expired, InvalidSignature, Malformed),
        // but for the refresh flow any failure = invalid refresh token from the client's
        // perspective. The detailed error is intentionally discarded here.
        let claims = validate_token(refresh_token_value, &self.jwt_secret)
            .map_err(|_| AuthServiceError::InvalidRefreshToken)?;

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
