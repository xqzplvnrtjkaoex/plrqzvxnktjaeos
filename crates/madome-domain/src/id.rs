//! Newtype wrappers for domain identifiers.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identifies a user account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for UserId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl From<Uuid> for UserId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

/// Identifies a book in the library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BookId(pub u32);

impl fmt::Display for BookId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for BookId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl From<u32> for BookId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

/// Identifies an email authentication code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuthcodeId(pub Uuid);

impl fmt::Display for AuthcodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for AuthcodeId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl From<Uuid> for AuthcodeId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_round_trip_user_id_via_display_and_from_str() {
        let id = UserId(Uuid::new_v4());
        let s = id.to_string();
        let parsed: UserId = s.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn should_round_trip_book_id_via_display_and_from_str() {
        let id = BookId(42);
        let s = id.to_string();
        let parsed: BookId = s.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn should_round_trip_authcode_id_via_display_and_from_str() {
        let id = AuthcodeId(Uuid::new_v4());
        let s = id.to_string();
        let parsed: AuthcodeId = s.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn should_serialize_user_id_as_uuid_string() {
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let id = UserId(uuid);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"550e8400-e29b-41d4-a716-446655440000\"");
    }
}
