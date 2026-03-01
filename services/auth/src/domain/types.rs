use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Auth-relevant user data fetched from users service (email + role for auth decisions).
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub role: u8,
}

/// One-time auth code used for passwordless login.
#[derive(Debug, Clone)]
pub struct AuthCode {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl AuthCode {
    pub fn is_valid(&self) -> bool {
        self.used_at.is_none() && self.expires_at > Utc::now()
    }
}

/// Stored WebAuthn passkey credential.
#[derive(Debug, Clone)]
pub struct PasskeyRecord {
    pub credential_id: Vec<u8>,
    pub user_id: Uuid,
    pub aaguid: Uuid,
    /// JSON-serialized `webauthn_rs::Passkey` (with counter).
    pub credential: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

/// Outbox event for async delivery (e.g. authcode email).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEvent {
    pub id: Uuid,
    pub kind: String,
    pub payload: serde_json::Value,
    pub idempotency_key: String,
}

/// Maximum number of active (unused, unexpired) auth codes per user.
pub const MAX_ACTIVE_AUTHCODES: u64 = 5;

/// Auth code length in characters.
pub const AUTHCODE_LEN: usize = 12;

/// Auth code time-to-live in seconds.
pub const AUTHCODE_TTL_SECS: i64 = 120;

/// WebAuthn session state TTL in seconds (same as authcode TTL).
pub const PASSKEY_STATE_TTL_SECS: usize = 120;
