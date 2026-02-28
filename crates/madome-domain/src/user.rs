//! User domain types.

use serde::{Deserialize, Serialize};

/// User permission level.
///
/// Wire format: `u8` (0 = Normal, 1 = Developer, 2 = Bot).
/// Compat names preserved from legacy; rename planned for post-Stabilize phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Normal = 0,
    Developer = 1,
    Bot = 2,
}

impl UserRole {
    /// Convert from `u8` wire value. Returns `None` for unknown values.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Normal),
            1 => Some(Self::Developer),
            2 => Some(Self::Bot),
            _ => None,
        }
    }

    /// Convert to `u8` wire value.
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

impl PartialOrd for UserRole {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UserRole {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_u8().cmp(&other.as_u8())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_convert_u8_to_user_role() {
        assert_eq!(UserRole::from_u8(0), Some(UserRole::Normal));
        assert_eq!(UserRole::from_u8(1), Some(UserRole::Developer));
        assert_eq!(UserRole::from_u8(2), Some(UserRole::Bot));
        assert_eq!(UserRole::from_u8(3), None);
    }

    #[test]
    fn should_convert_user_role_to_u8() {
        assert_eq!(UserRole::Normal.as_u8(), 0);
        assert_eq!(UserRole::Developer.as_u8(), 1);
        assert_eq!(UserRole::Bot.as_u8(), 2);
    }

    #[test]
    fn should_order_roles_by_privilege_level() {
        assert!(UserRole::Normal < UserRole::Developer);
        assert!(UserRole::Developer < UserRole::Bot);
        assert!(UserRole::Normal < UserRole::Bot);
    }

    #[test]
    fn should_round_trip_user_role_via_serde() {
        for role in [UserRole::Normal, UserRole::Developer, UserRole::Bot] {
            let json = serde_json::to_string(&role).unwrap();
            let parsed: UserRole = serde_json::from_str(&json).unwrap();
            assert_eq!(role, parsed);
        }
    }
}
