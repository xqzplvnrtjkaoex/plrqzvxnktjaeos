use chrono::{Duration, Utc};
use rand::RngExt;
use serde_json::json;
use uuid::Uuid;

use crate::domain::repository::{AuthCodeRepository, UserPort};
use crate::domain::types::{
    AUTHCODE_LEN, AUTHCODE_TTL_SECS, AuthCode, MAX_ACTIVE_AUTHCODES, OutboxEvent,
};
use crate::error::AuthServiceError;

/// Charset for generating random auth codes (uppercase alphanumeric).
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

fn generate_code() -> String {
    let mut rng = rand::rng();
    (0..AUTHCODE_LEN)
        .map(|_| CHARSET[rng.random_range(0..CHARSET.len())] as char)
        .collect()
}

pub struct CreateAuthcodeInput {
    pub email: String,
}

pub struct CreateAuthcodeUseCase<U, A>
where
    U: UserPort,
    A: AuthCodeRepository,
{
    pub users: U,
    pub auth_codes: A,
}

impl<U, A> CreateAuthcodeUseCase<U, A>
where
    U: UserPort,
    A: AuthCodeRepository,
{
    pub async fn execute(&self, input: CreateAuthcodeInput) -> Result<(), AuthServiceError> {
        // 1. Find user by email → 404 if not found
        let user = self
            .users
            .find_by_email(&input.email)
            .await?
            .ok_or(AuthServiceError::UserNotFound)?;

        // 2. Check active code limit → 429 if at or over limit
        let active = self.auth_codes.count_active(user.id).await?;
        if active >= MAX_ACTIVE_AUTHCODES {
            return Err(AuthServiceError::TooManyAuthcodes);
        }

        // 3. Generate code + authcode record
        let code_str = generate_code();
        let now = Utc::now();
        let code = AuthCode {
            id: Uuid::new_v4(),
            user_id: user.id,
            code: code_str.clone(),
            expires_at: now + Duration::seconds(AUTHCODE_TTL_SECS),
            used_at: None,
            created_at: now,
        };

        // 4. Write authcode + outbox event in same transaction
        let event = OutboxEvent {
            id: Uuid::new_v4(),
            kind: "authcode_created".to_owned(),
            payload: json!({ "email": input.email, "code": code_str }),
            idempotency_key: format!("authcode_created:{}", code.id),
        };

        self.auth_codes.create_with_outbox(&code, &event).await?;
        Ok(())
    }
}
