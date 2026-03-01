#![allow(async_fn_in_trait)]

use uuid::Uuid;

use madome_domain::pagination::PageRequest;

use crate::domain::types::{
    FcmToken, HistoryBook, HistorySortBy, NotificationBook, NotificationSortBy, TasteBook,
    TasteBookTag, TasteSortBy, User,
};
use crate::error::UsersServiceError;

/// Repository for user profiles.
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, UsersServiceError>;
    async fn create(&self, user: &User) -> Result<(), UsersServiceError>;
    async fn update_name_handle(
        &self,
        id: Uuid,
        name: Option<&str>,
        handle: Option<&str>,
    ) -> Result<(), UsersServiceError>;
}

/// Repository for book tastes (likes/dislikes).
pub trait TasteBookRepository: Send + Sync {
    async fn list(
        &self,
        user_id: Uuid,
        sort_by: TasteSortBy,
        is_dislike: Option<bool>,
        page: PageRequest,
    ) -> Result<Vec<TasteBook>, UsersServiceError>;

    async fn list_by_book_ids(
        &self,
        user_id: Uuid,
        book_ids: &[i32],
    ) -> Result<Vec<TasteBook>, UsersServiceError>;

    async fn get(
        &self,
        user_id: Uuid,
        book_id: i32,
    ) -> Result<Option<TasteBook>, UsersServiceError>;

    /// Upsert a book taste. Returns `true` if the `is_dislike` value changed.
    async fn upsert(&self, taste: &TasteBook) -> Result<bool, UsersServiceError>;

    /// Delete a book taste. Returns `true` if a row was deleted.
    async fn delete(&self, user_id: Uuid, book_id: i32) -> Result<bool, UsersServiceError>;
}

/// Repository for book-tag tastes (likes/dislikes).
pub trait TasteBookTagRepository: Send + Sync {
    async fn list(
        &self,
        user_id: Uuid,
        sort_by: TasteSortBy,
        is_dislike: Option<bool>,
        page: PageRequest,
    ) -> Result<Vec<TasteBookTag>, UsersServiceError>;

    async fn get(
        &self,
        user_id: Uuid,
        tag_kind: &str,
        tag_name: &str,
    ) -> Result<Option<TasteBookTag>, UsersServiceError>;

    /// Upsert a book-tag taste. Returns `true` if the `is_dislike` value changed.
    async fn upsert(&self, taste: &TasteBookTag) -> Result<bool, UsersServiceError>;

    /// Delete a book-tag taste. Returns `true` if a row was deleted.
    async fn delete(
        &self,
        user_id: Uuid,
        tag_kind: &str,
        tag_name: &str,
    ) -> Result<bool, UsersServiceError>;
}

/// Repository for book reading history.
pub trait HistoryRepository: Send + Sync {
    async fn list(
        &self,
        user_id: Uuid,
        sort_by: HistorySortBy,
        page: PageRequest,
    ) -> Result<Vec<HistoryBook>, UsersServiceError>;

    async fn get(
        &self,
        user_id: Uuid,
        book_id: i32,
    ) -> Result<Option<HistoryBook>, UsersServiceError>;

    async fn upsert(&self, history: &HistoryBook) -> Result<(), UsersServiceError>;

    /// Delete a history entry. Returns `true` if a row was deleted.
    async fn delete(&self, user_id: Uuid, book_id: i32) -> Result<bool, UsersServiceError>;
}

/// Repository for book notifications.
pub trait NotificationRepository: Send + Sync {
    async fn list(
        &self,
        user_id: Uuid,
        sort_by: NotificationSortBy,
        page: PageRequest,
    ) -> Result<Vec<NotificationBook>, UsersServiceError>;

    async fn create(&self, notification: &NotificationBook) -> Result<(), UsersServiceError>;
}

/// Atomically rename old_book_id to new_book_id across taste, history, and notification tables.
pub trait RenewBookPort: Send + Sync {
    async fn renew_book_id(&self, old_id: i32, new_id: i32) -> Result<(), UsersServiceError>;
}

/// Repository for FCM push tokens.
pub trait FcmTokenRepository: Send + Sync {
    /// Upsert an FCM token. The `user_id` guard ensures only the token owner can update.
    async fn upsert(&self, token: &FcmToken, user_id: Uuid) -> Result<(), UsersServiceError>;

    /// Find tokens updated within the last 30 days for the given user IDs.
    async fn find_fresh_by_user_ids(
        &self,
        user_ids: &[Uuid],
    ) -> Result<Vec<FcmToken>, UsersServiceError>;
}

/// Port for querying the library service (book/tag existence).
pub trait LibraryQueryPort: Send + Sync {
    async fn has_book(&self, book_id: i32) -> Result<bool, UsersServiceError>;
    async fn has_book_tag(
        &self,
        tag_kind: &str,
        tag_name: &str,
    ) -> Result<bool, UsersServiceError>;
}
