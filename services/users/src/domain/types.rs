use chrono::{DateTime, Utc};
use uuid::Uuid;

use madome_domain::pagination::Sort;

/// User profile owned by the users service.
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub handle: String,
    pub email: String,
    pub role: u8,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A book taste (like or dislike).
#[derive(Debug, Clone)]
pub struct TasteBook {
    pub user_id: Uuid,
    pub book_id: i32,
    pub is_dislike: bool,
    pub created_at: DateTime<Utc>,
}

/// A book-tag taste (like or dislike for a tag).
#[derive(Debug, Clone)]
pub struct TasteBookTag {
    pub user_id: Uuid,
    pub tag_kind: String,
    pub tag_name: String,
    pub is_dislike: bool,
    pub created_at: DateTime<Utc>,
}

/// Unified taste enum — returned by combined UNION ALL queries.
#[derive(Debug, Clone)]
pub enum Taste {
    Book(TasteBook),
    BookTag(TasteBookTag),
}

/// A book reading history entry.
#[derive(Debug, Clone)]
pub struct HistoryBook {
    pub user_id: Uuid,
    pub book_id: i32,
    pub page: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A book notification with its associated tags.
#[derive(Debug, Clone)]
pub struct NotificationBook {
    pub id: Uuid,
    pub user_id: Uuid,
    pub book_id: i32,
    pub book_tags: Vec<(String, String)>,
    pub created_at: DateTime<Utc>,
}

/// A single tag on a notification.
#[derive(Debug, Clone)]
pub struct NotificationBookTag {
    pub id: Uuid,
    pub notification_book_id: Uuid,
    pub tag_kind: String,
    pub tag_name: String,
}

/// FCM push token for a device.
#[derive(Debug, Clone)]
pub struct FcmToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub updated_at: DateTime<Utc>,
}

/// Sort options for taste list queries.
#[derive(Debug, Clone, Copy)]
pub enum TasteSortBy {
    CreatedAt(Sort),
    Random,
}

impl Default for TasteSortBy {
    fn default() -> Self {
        Self::CreatedAt(Sort::Desc)
    }
}

/// Sort options for history list queries.
#[derive(Debug, Clone, Copy)]
pub enum HistorySortBy {
    CreatedAt(Sort),
    UpdatedAt(Sort),
    Random,
}

impl Default for HistorySortBy {
    fn default() -> Self {
        Self::UpdatedAt(Sort::Desc)
    }
}

/// Sort options for notification list queries.
#[derive(Debug, Clone, Copy)]
pub enum NotificationSortBy {
    CreatedAt(Sort),
}

impl Default for NotificationSortBy {
    fn default() -> Self {
        Self::CreatedAt(Sort::Desc)
    }
}

impl TasteSortBy {
    pub fn from_kebab_case(s: &str) -> Option<Self> {
        match s {
            "created-at-desc" => Some(Self::CreatedAt(Sort::Desc)),
            "created-at-asc" => Some(Self::CreatedAt(Sort::Asc)),
            "random" => Some(Self::Random),
            _ => None,
        }
    }
}

impl HistorySortBy {
    pub fn from_kebab_case(s: &str) -> Option<Self> {
        match s {
            "created-at-desc" => Some(Self::CreatedAt(Sort::Desc)),
            "created-at-asc" => Some(Self::CreatedAt(Sort::Asc)),
            "updated-at-desc" => Some(Self::UpdatedAt(Sort::Desc)),
            "updated-at-asc" => Some(Self::UpdatedAt(Sort::Asc)),
            "random" => Some(Self::Random),
            _ => None,
        }
    }
}

impl NotificationSortBy {
    pub fn from_kebab_case(s: &str) -> Option<Self> {
        match s {
            "created-at-desc" => Some(Self::CreatedAt(Sort::Desc)),
            "created-at-asc" => Some(Self::CreatedAt(Sort::Asc)),
            _ => None,
        }
    }
}

/// Validate a user handle: alphanumeric + hyphen + underscore, 1-20 chars.
/// Reserved: "me". Rejects handles starting with '@'.
pub fn validate_handle(handle: &str) -> bool {
    if handle.is_empty() || handle.len() > 20 {
        return false;
    }
    if handle == "me" {
        return false;
    }
    if handle.starts_with('@') {
        return false;
    }
    handle
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_accept_valid_handle() {
        assert!(validate_handle("alice"));
        assert!(validate_handle("bob-123"));
        assert!(validate_handle("user_name"));
        assert!(validate_handle("a"));
    }

    #[test]
    fn should_reject_empty_handle() {
        assert!(!validate_handle(""));
    }

    #[test]
    fn should_reject_too_long_handle() {
        assert!(!validate_handle("a]bcdefghijklmnopqrstu")); // 21 chars
    }

    #[test]
    fn should_reject_reserved_me() {
        assert!(!validate_handle("me"));
    }

    #[test]
    fn should_reject_at_prefix() {
        assert!(!validate_handle("@someone"));
    }

    #[test]
    fn should_reject_special_chars() {
        assert!(!validate_handle("user.name"));
        assert!(!validate_handle("user name"));
        assert!(!validate_handle("user@name"));
    }

    #[test]
    fn should_parse_taste_sort_from_kebab_case() {
        assert!(matches!(
            TasteSortBy::from_kebab_case("created-at-desc"),
            Some(TasteSortBy::CreatedAt(Sort::Desc))
        ));
        assert!(matches!(
            TasteSortBy::from_kebab_case("created-at-asc"),
            Some(TasteSortBy::CreatedAt(Sort::Asc))
        ));
        assert!(matches!(
            TasteSortBy::from_kebab_case("random"),
            Some(TasteSortBy::Random)
        ));
        assert!(TasteSortBy::from_kebab_case("invalid").is_none());
    }

    #[test]
    fn should_parse_history_sort_from_kebab_case() {
        assert!(matches!(
            HistorySortBy::from_kebab_case("updated-at-desc"),
            Some(HistorySortBy::UpdatedAt(Sort::Desc))
        ));
        assert!(matches!(
            HistorySortBy::from_kebab_case("created-at-asc"),
            Some(HistorySortBy::CreatedAt(Sort::Asc))
        ));
        assert!(matches!(
            HistorySortBy::from_kebab_case("random"),
            Some(HistorySortBy::Random)
        ));
        assert!(HistorySortBy::from_kebab_case("invalid").is_none());
    }

    #[test]
    fn should_parse_notification_sort_from_kebab_case() {
        assert!(matches!(
            NotificationSortBy::from_kebab_case("created-at-desc"),
            Some(NotificationSortBy::CreatedAt(Sort::Desc))
        ));
        assert!(matches!(
            NotificationSortBy::from_kebab_case("created-at-asc"),
            Some(NotificationSortBy::CreatedAt(Sort::Asc))
        ));
        assert!(NotificationSortBy::from_kebab_case("random").is_none());
    }
}
