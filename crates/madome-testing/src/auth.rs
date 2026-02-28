//! Mock auth helpers for integration tests.
//!
//! Services behind the gateway receive `x-madome-user-id` + `x-madome-user-role` headers
//! injected by the gateway. In tests, `MockAuthServer` injects these headers directly
//! so no real gateway or JWT is needed.

use axum::http::{HeaderMap, HeaderName, HeaderValue};
use uuid::Uuid;

/// Configurable identity injected into test requests.
pub struct MockAuth {
    pub user_id: Uuid,
    pub user_role: u8,
}

impl MockAuth {
    pub fn new(user_id: Uuid, user_role: u8) -> Self {
        Self { user_id, user_role }
    }

    /// Return headers as if the gateway injected them.
    pub fn headers(&self) -> HeaderMap {
        let mut map = HeaderMap::new();
        map.insert(
            HeaderName::from_static("x-madome-user-id"),
            HeaderValue::from_str(&self.user_id.to_string()).unwrap(),
        );
        map.insert(
            HeaderName::from_static("x-madome-user-role"),
            HeaderValue::from_str(&self.user_role.to_string()).unwrap(),
        );
        map
    }
}
