use deadpool_redis::Pool;
use deadpool_redis::redis::AsyncCommands;
use uuid::Uuid;

use crate::domain::repository::PasskeyCache;
use crate::domain::types::PASSKEY_STATE_TTL_SECS;
use crate::error::AuthServiceError;

#[derive(Clone)]
pub struct RedisPasskeyCache {
    pub pool: Pool,
}

fn reg_state_key(user_id: Uuid, reg_id: &str) -> String {
    format!("passkey_reg:{}:{}", user_id, reg_id)
}

fn auth_state_key(email: &str, auth_id: &str) -> String {
    format!("passkey_auth:{}:{}", email, auth_id)
}

impl PasskeyCache for RedisPasskeyCache {
    async fn set_registration_state(
        &self,
        user_id: Uuid,
        reg_id: &str,
        state_json: &[u8],
    ) -> Result<(), AuthServiceError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AuthServiceError::Internal(e.into()))?;
        let key = reg_state_key(user_id, reg_id);
        let (): () = conn
            .set_ex(&key, state_json.to_vec(), PASSKEY_STATE_TTL_SECS as u64)
            .await
            .map_err(|e: deadpool_redis::redis::RedisError| AuthServiceError::Internal(e.into()))?;
        Ok(())
    }

    async fn take_registration_state(
        &self,
        user_id: Uuid,
        reg_id: &str,
    ) -> Result<Option<Vec<u8>>, AuthServiceError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AuthServiceError::Internal(e.into()))?;
        let key = reg_state_key(user_id, reg_id);
        let value: Option<Vec<u8>> = conn
            .get_del(&key)
            .await
            .map_err(|e| AuthServiceError::Internal(e.into()))?;
        Ok(value)
    }

    async fn set_authentication_state(
        &self,
        email: &str,
        auth_id: &str,
        state_json: &[u8],
    ) -> Result<(), AuthServiceError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AuthServiceError::Internal(e.into()))?;
        let key = auth_state_key(email, auth_id);
        let (): () = conn
            .set_ex(&key, state_json.to_vec(), PASSKEY_STATE_TTL_SECS as u64)
            .await
            .map_err(|e: deadpool_redis::redis::RedisError| AuthServiceError::Internal(e.into()))?;
        Ok(())
    }

    async fn take_authentication_state(
        &self,
        email: &str,
        auth_id: &str,
    ) -> Result<Option<Vec<u8>>, AuthServiceError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AuthServiceError::Internal(e.into()))?;
        let key = auth_state_key(email, auth_id);
        let value: Option<Vec<u8>> = conn
            .get_del(&key)
            .await
            .map_err(|e| AuthServiceError::Internal(e.into()))?;
        Ok(value)
    }
}
