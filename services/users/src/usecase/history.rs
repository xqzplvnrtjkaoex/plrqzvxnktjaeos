use chrono::Utc;
use uuid::Uuid;

use madome_domain::pagination::PageRequest;

use crate::domain::repository::HistoryRepository;
use crate::domain::types::{HistoryBook, HistorySortBy};
use crate::error::UsersServiceError;

// ── GetHistory ───────────────────────────────────────────────────────────────

pub struct GetHistoryUseCase<R: HistoryRepository> {
    pub repo: R,
}

impl<R: HistoryRepository> GetHistoryUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        book_id: i32,
    ) -> Result<HistoryBook, UsersServiceError> {
        self.repo
            .get(user_id, book_id)
            .await?
            .ok_or(UsersServiceError::HistoryNotFound)
    }
}

// ── GetHistories ─────────────────────────────────────────────────────────────

pub struct GetHistoriesUseCase<R: HistoryRepository> {
    pub repo: R,
}

impl<R: HistoryRepository> GetHistoriesUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        sort_by: HistorySortBy,
        page: PageRequest,
    ) -> Result<Vec<HistoryBook>, UsersServiceError> {
        self.repo.list(user_id, sort_by, page).await
    }
}

// ── CreateOrUpdateHistory ────────────────────────────────────────────────────

pub struct CreateHistoryInput {
    pub book_id: i32,
    pub page: i32,
}

pub struct CreateHistoryUseCase<R: HistoryRepository> {
    pub repo: R,
}

impl<R: HistoryRepository> CreateHistoryUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        input: CreateHistoryInput,
    ) -> Result<(), UsersServiceError> {
        let now = Utc::now();
        let history = HistoryBook {
            user_id,
            book_id: input.book_id,
            page: input.page,
            created_at: now,
            updated_at: now,
        };
        self.repo.upsert(&history).await
    }
}

// ── DeleteHistory ────────────────────────────────────────────────────────────

pub struct DeleteHistoryUseCase<R: HistoryRepository> {
    pub repo: R,
}

impl<R: HistoryRepository> DeleteHistoryUseCase<R> {
    pub async fn execute(&self, user_id: Uuid, book_id: i32) -> Result<(), UsersServiceError> {
        let deleted = self.repo.delete(user_id, book_id).await?;
        if !deleted {
            return Err(UsersServiceError::HistoryNotFound);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockHistoryRepo {
        history: Option<HistoryBook>,
        delete_returns: bool,
    }

    impl HistoryRepository for MockHistoryRepo {
        async fn list(
            &self,
            _user_id: Uuid,
            _sort_by: HistorySortBy,
            _page: PageRequest,
        ) -> Result<Vec<HistoryBook>, UsersServiceError> {
            Ok(vec![])
        }
        async fn get(
            &self,
            _user_id: Uuid,
            _book_id: i32,
        ) -> Result<Option<HistoryBook>, UsersServiceError> {
            Ok(self.history.clone())
        }
        async fn upsert(&self, _history: &HistoryBook) -> Result<(), UsersServiceError> {
            Ok(())
        }
        async fn delete(&self, _user_id: Uuid, _book_id: i32) -> Result<bool, UsersServiceError> {
            Ok(self.delete_returns)
        }
    }

    #[tokio::test]
    async fn should_create_history() {
        let usecase = CreateHistoryUseCase {
            repo: MockHistoryRepo {
                history: None,
                delete_returns: false,
            },
        };
        let result = usecase
            .execute(
                Uuid::now_v7(),
                CreateHistoryInput {
                    book_id: 1,
                    page: 5,
                },
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_return_history_not_found_on_get_missing() {
        let usecase = GetHistoryUseCase {
            repo: MockHistoryRepo {
                history: None,
                delete_returns: false,
            },
        };
        let result = usecase.execute(Uuid::now_v7(), 999).await;
        assert!(matches!(result, Err(UsersServiceError::HistoryNotFound)));
    }

    #[tokio::test]
    async fn should_return_history_not_found_on_delete_missing() {
        let usecase = DeleteHistoryUseCase {
            repo: MockHistoryRepo {
                history: None,
                delete_returns: false,
            },
        };
        let result = usecase.execute(Uuid::now_v7(), 999).await;
        assert!(matches!(result, Err(UsersServiceError::HistoryNotFound)));
    }
}
