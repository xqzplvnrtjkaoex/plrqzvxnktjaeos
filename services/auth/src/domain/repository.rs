#![allow(async_fn_in_trait)]

use uuid::Uuid;

use crate::domain::types::{AuthCode, AuthUser, OutboxEvent, PasskeyRecord};
use crate::error::AuthServiceError;

/// Port for looking up users via the users service.
pub trait UserPort: Send + Sync {
    async fn find_by_email(&self, email: &str) -> Result<Option<AuthUser>, AuthServiceError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuthUser>, AuthServiceError>;
}

/// Repository for one-time auth codes.
pub trait AuthCodeRepository: Send + Sync {
    /// Count active (unused and unexpired) codes for a user.
    async fn count_active(&self, user_id: Uuid) -> Result<u64, AuthServiceError>;

    /// Insert a new auth code and an outbox event atomically (same transaction).
    async fn create_with_outbox(
        &self,
        code: &AuthCode,
        event: &OutboxEvent,
    ) -> Result<(), AuthServiceError>;

    /// Find a valid (unused, unexpired) code by user + code string.
    async fn find_valid(
        &self,
        user_id: Uuid,
        code: &str,
    ) -> Result<Option<AuthCode>, AuthServiceError>;

    /// Mark a code as used (sets used_at = now).
    async fn mark_used(&self, id: Uuid) -> Result<(), AuthServiceError>;
}

/// Repository for WebAuthn passkey credentials.
pub trait PasskeyRepository: Send + Sync {
    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<PasskeyRecord>, AuthServiceError>;

    async fn find_by_id(
        &self,
        credential_id: &[u8],
    ) -> Result<Option<PasskeyRecord>, AuthServiceError>;

    async fn create(&self, record: &PasskeyRecord) -> Result<(), AuthServiceError>;

    /// Delete a passkey. Returns `true` if deleted, `false` if not found.
    async fn delete(&self, credential_id: &[u8], user_id: Uuid) -> Result<bool, AuthServiceError>;

    /// Replace an existing passkey credential (used to update counter after authentication).
    async fn update_credential(
        &self,
        credential_id: &[u8],
        credential: &[u8],
    ) -> Result<(), AuthServiceError>;
}

/// Cache for WebAuthn ceremony states (Redis, short TTL).
pub trait PasskeyCache: Send + Sync {
    async fn set_registration_state(
        &self,
        user_id: Uuid,
        reg_id: &str,
        state_json: &[u8],
    ) -> Result<(), AuthServiceError>;

    async fn take_registration_state(
        &self,
        user_id: Uuid,
        reg_id: &str,
    ) -> Result<Option<Vec<u8>>, AuthServiceError>;

    async fn set_authentication_state(
        &self,
        email: &str,
        auth_id: &str,
        state_json: &[u8],
    ) -> Result<(), AuthServiceError>;

    async fn take_authentication_state(
        &self,
        email: &str,
        auth_id: &str,
    ) -> Result<Option<Vec<u8>>, AuthServiceError>;
}
