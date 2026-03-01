use uuid::Uuid;

use madome_domain::pagination::PageRequest;

use crate::domain::repository::NotificationRepository;
use crate::domain::types::{NotificationBook, NotificationSortBy};
use crate::error::UsersServiceError;

// ── GetNotifications ─────────────────────────────────────────────────────────

pub struct GetNotificationsUseCase<R: NotificationRepository> {
    pub repo: R,
}

impl<R: NotificationRepository> GetNotificationsUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        sort_by: NotificationSortBy,
        page: PageRequest,
    ) -> Result<Vec<NotificationBook>, UsersServiceError> {
        self.repo.list(user_id, sort_by, page).await
    }
}

// ── CreateNotification (gRPC path) ───────────────────────────────────────────

pub struct CreateNotificationUseCase<R: NotificationRepository> {
    pub repo: R,
}

impl<R: NotificationRepository> CreateNotificationUseCase<R> {
    pub async fn execute(&self, notification: NotificationBook) -> Result<(), UsersServiceError> {
        self.repo.create(&notification).await
    }
}
