use std::sync::{Arc, Mutex};

use chrono::Utc;
use uuid::Uuid;

use madome_auth::domain::repository::{AuthCodeRepository, PasskeyRepository, UserRepository};
use madome_auth::domain::types::{AuthCode, AuthUser, OutboxEvent, PasskeyRecord};
use madome_auth::error::AuthServiceError;

// ── MockUserRepo ─────────────────────────────────────────────────────────────

pub struct MockUserRepo {
    pub users: Vec<AuthUser>,
}

impl MockUserRepo {
    pub fn new(users: Vec<AuthUser>) -> Self {
        Self { users }
    }

    pub fn empty() -> Self {
        Self { users: vec![] }
    }
}

impl UserRepository for MockUserRepo {
    async fn find_by_email(&self, email: &str) -> Result<Option<AuthUser>, AuthServiceError> {
        Ok(self.users.iter().find(|u| u.email == email).cloned())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuthUser>, AuthServiceError> {
        Ok(self.users.iter().find(|u| u.id == id).cloned())
    }
}

// ── MockAuthCodeRepo ─────────────────────────────────────────────────────────

pub struct MockAuthCodeRepo {
    pub codes: Arc<Mutex<Vec<AuthCode>>>,
    pub active_count: u64,
}

impl MockAuthCodeRepo {
    pub fn new(codes: Vec<AuthCode>, active_count: u64) -> Self {
        Self {
            codes: Arc::new(Mutex::new(codes)),
            active_count,
        }
    }

    pub fn empty() -> Self {
        Self::new(vec![], 0)
    }

    /// Returns a shared handle to the internal code list for post-execution inspection.
    pub fn codes_handle(&self) -> Arc<Mutex<Vec<AuthCode>>> {
        Arc::clone(&self.codes)
    }
}

impl AuthCodeRepository for MockAuthCodeRepo {
    async fn count_active(&self, _user_id: Uuid) -> Result<u64, AuthServiceError> {
        Ok(self.active_count)
    }

    async fn create_with_outbox(
        &self,
        code: &AuthCode,
        _event: &OutboxEvent,
    ) -> Result<(), AuthServiceError> {
        self.codes.lock().unwrap().push(code.clone());
        Ok(())
    }

    async fn find_valid(
        &self,
        user_id: Uuid,
        code: &str,
    ) -> Result<Option<AuthCode>, AuthServiceError> {
        Ok(self
            .codes
            .lock()
            .unwrap()
            .iter()
            .find(|c| c.user_id == user_id && c.code == code && c.is_valid())
            .cloned())
    }

    async fn mark_used(&self, id: Uuid) -> Result<(), AuthServiceError> {
        let mut codes = self.codes.lock().unwrap();
        if let Some(c) = codes.iter_mut().find(|c| c.id == id) {
            c.used_at = Some(Utc::now());
        }
        Ok(())
    }
}

// ── MockPasskeyRepo ──────────────────────────────────────────────────────────

pub struct MockPasskeyRepo {
    pub records: Vec<PasskeyRecord>,
}

impl MockPasskeyRepo {
    pub fn new(records: Vec<PasskeyRecord>) -> Self {
        Self { records }
    }

    pub fn empty() -> Self {
        Self { records: vec![] }
    }
}

impl PasskeyRepository for MockPasskeyRepo {
    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<PasskeyRecord>, AuthServiceError> {
        Ok(self
            .records
            .iter()
            .filter(|r| r.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn find_by_id(
        &self,
        credential_id: &[u8],
    ) -> Result<Option<PasskeyRecord>, AuthServiceError> {
        Ok(self
            .records
            .iter()
            .find(|r| r.credential_id == credential_id)
            .cloned())
    }

    async fn create(&self, _record: &PasskeyRecord) -> Result<(), AuthServiceError> {
        Ok(())
    }

    async fn delete(&self, credential_id: &[u8], user_id: Uuid) -> Result<bool, AuthServiceError> {
        Ok(self
            .records
            .iter()
            .any(|r| r.credential_id == credential_id && r.user_id == user_id))
    }

    async fn update_credential(
        &self,
        _credential_id: &[u8],
        _credential: &[u8],
    ) -> Result<(), AuthServiceError> {
        Ok(())
    }
}

// ── Test fixture helpers ─────────────────────────────────────────────────────

pub fn test_user() -> AuthUser {
    AuthUser {
        id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
        email: "user@example.com".to_owned(),
        role: 0,
    }
}

pub fn test_auth_code(user_id: Uuid) -> AuthCode {
    AuthCode {
        id: Uuid::new_v4(),
        user_id,
        code: "ABCDEF123456".to_owned(),
        expires_at: Utc::now() + chrono::Duration::seconds(120),
        used_at: None,
        created_at: Utc::now(),
    }
}

pub fn test_passkey_record(user_id: Uuid) -> PasskeyRecord {
    PasskeyRecord {
        credential_id: vec![1, 2, 3, 4],
        user_id,
        aaguid: Uuid::nil(),
        credential: vec![],
        created_at: Utc::now(),
    }
}

pub const TEST_JWT_SECRET: &str = "test-jwt-secret-for-unit-tests-only";
