//! User activity domain types: tastes, histories, notifications.

use serde::{Deserialize, Serialize};

/// Category of a user taste (like/dislike).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TasteKind {
    Book,
    BookTag,
}

/// Category of a user reading history entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoryKind {
    Book,
}

/// Category of a user notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationKind {
    Book,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_taste_kind_as_snake_case() {
        assert_eq!(serde_json::to_string(&TasteKind::Book).unwrap(), "\"book\"");
        assert_eq!(
            serde_json::to_string(&TasteKind::BookTag).unwrap(),
            "\"book_tag\""
        );
    }

    #[test]
    fn should_deserialize_taste_kind_from_snake_case() {
        assert_eq!(
            serde_json::from_str::<TasteKind>("\"book\"").unwrap(),
            TasteKind::Book
        );
        assert_eq!(
            serde_json::from_str::<TasteKind>("\"book_tag\"").unwrap(),
            TasteKind::BookTag
        );
    }

    #[test]
    fn should_serialize_history_kind_as_snake_case() {
        assert_eq!(
            serde_json::to_string(&HistoryKind::Book).unwrap(),
            "\"book\""
        );
    }

    #[test]
    fn should_serialize_notification_kind_as_snake_case() {
        assert_eq!(
            serde_json::to_string(&NotificationKind::Book).unwrap(),
            "\"book\""
        );
    }
}
