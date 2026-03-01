use chrono::Utc;
use uuid::Uuid;

use madome_domain::pagination::PageRequest;

use crate::domain::repository::{LibraryQueryPort, TasteBookRepository, TasteBookTagRepository};
use crate::domain::types::{TasteBook, TasteBookTag, TasteSortBy};
use crate::error::UsersServiceError;

// ── GetTaste (single book) ───────────────────────────────────────────────────

pub struct GetTasteBookUseCase<R: TasteBookRepository> {
    pub repo: R,
}

impl<R: TasteBookRepository> GetTasteBookUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        book_id: i32,
    ) -> Result<TasteBook, UsersServiceError> {
        self.repo
            .get(user_id, book_id)
            .await?
            .ok_or(UsersServiceError::TasteNotFound)
    }
}

// ── GetTaste (single book tag) ───────────────────────────────────────────────

pub struct GetTasteBookTagUseCase<R: TasteBookTagRepository> {
    pub repo: R,
}

impl<R: TasteBookTagRepository> GetTasteBookTagUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        tag_kind: &str,
        tag_name: &str,
    ) -> Result<TasteBookTag, UsersServiceError> {
        self.repo
            .get(user_id, tag_kind, tag_name)
            .await?
            .ok_or(UsersServiceError::TasteNotFound)
    }
}

// ── GetTastes (list books) ───────────────────────────────────────────────────

pub struct GetTasteBooksUseCase<R: TasteBookRepository> {
    pub repo: R,
}

impl<R: TasteBookRepository> GetTasteBooksUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        sort_by: TasteSortBy,
        is_dislike: Option<bool>,
        page: PageRequest,
    ) -> Result<Vec<TasteBook>, UsersServiceError> {
        self.repo.list(user_id, sort_by, is_dislike, page).await
    }
}

// ── GetTastes (list book tags) ───────────────────────────────────────────────

pub struct GetTasteBookTagsUseCase<R: TasteBookTagRepository> {
    pub repo: R,
}

impl<R: TasteBookTagRepository> GetTasteBookTagsUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        sort_by: TasteSortBy,
        is_dislike: Option<bool>,
        page: PageRequest,
    ) -> Result<Vec<TasteBookTag>, UsersServiceError> {
        self.repo.list(user_id, sort_by, is_dislike, page).await
    }
}

// ── GetTastesByBookIds ───────────────────────────────────────────────────────

pub struct GetTastesByBookIdsUseCase<R: TasteBookRepository> {
    pub repo: R,
}

impl<R: TasteBookRepository> GetTastesByBookIdsUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        book_ids: &[i32],
    ) -> Result<Vec<TasteBook>, UsersServiceError> {
        self.repo.list_by_book_ids(user_id, book_ids).await
    }
}

// ── CreateOrUpdateTaste (book) ───────────────────────────────────────────────

pub struct CreateTasteBookInput {
    pub book_id: i32,
    pub is_dislike: bool,
}

pub struct CreateTasteBookUseCase<R: TasteBookRepository, L: LibraryQueryPort> {
    pub repo: R,
    pub library: L,
}

impl<R: TasteBookRepository, L: LibraryQueryPort> CreateTasteBookUseCase<R, L> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        input: CreateTasteBookInput,
    ) -> Result<(), UsersServiceError> {
        if !self.library.has_book(input.book_id).await? {
            return Err(UsersServiceError::BookNotFound);
        }
        let taste = TasteBook {
            user_id,
            book_id: input.book_id,
            is_dislike: input.is_dislike,
            created_at: Utc::now(),
        };
        let changed = self.repo.upsert(&taste).await?;
        if !changed {
            return Err(UsersServiceError::TasteAlreadyExists);
        }
        Ok(())
    }
}

// ── CreateOrUpdateTaste (book tag) ───────────────────────────────────────────

pub struct CreateTasteBookTagInput {
    pub tag_kind: String,
    pub tag_name: String,
    pub is_dislike: bool,
}

pub struct CreateTasteBookTagUseCase<R: TasteBookTagRepository, L: LibraryQueryPort> {
    pub repo: R,
    pub library: L,
}

impl<R: TasteBookTagRepository, L: LibraryQueryPort> CreateTasteBookTagUseCase<R, L> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        input: CreateTasteBookTagInput,
    ) -> Result<(), UsersServiceError> {
        if !self
            .library
            .has_book_tag(&input.tag_kind, &input.tag_name)
            .await?
        {
            return Err(UsersServiceError::BookTagNotFound);
        }
        let taste = TasteBookTag {
            user_id,
            tag_kind: input.tag_kind,
            tag_name: input.tag_name,
            is_dislike: input.is_dislike,
            created_at: Utc::now(),
        };
        let changed = self.repo.upsert(&taste).await?;
        if !changed {
            return Err(UsersServiceError::TasteAlreadyExists);
        }
        Ok(())
    }
}

// ── DeleteTaste (book) ───────────────────────────────────────────────────────

pub struct DeleteTasteBookUseCase<R: TasteBookRepository> {
    pub repo: R,
}

impl<R: TasteBookRepository> DeleteTasteBookUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        book_id: i32,
    ) -> Result<(), UsersServiceError> {
        let deleted = self.repo.delete(user_id, book_id).await?;
        if !deleted {
            return Err(UsersServiceError::TasteNotFound);
        }
        Ok(())
    }
}

// ── DeleteTaste (book tag) ───────────────────────────────────────────────────

pub struct DeleteTasteBookTagUseCase<R: TasteBookTagRepository> {
    pub repo: R,
}

impl<R: TasteBookTagRepository> DeleteTasteBookTagUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        tag_kind: &str,
        tag_name: &str,
    ) -> Result<(), UsersServiceError> {
        let deleted = self.repo.delete(user_id, tag_kind, tag_name).await?;
        if !deleted {
            return Err(UsersServiceError::TasteNotFound);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTasteBookRepo {
        taste: Option<TasteBook>,
        upsert_returns: bool,
        delete_returns: bool,
    }

    impl TasteBookRepository for MockTasteBookRepo {
        async fn list(
            &self,
            _user_id: Uuid,
            _sort_by: TasteSortBy,
            _is_dislike: Option<bool>,
            _page: PageRequest,
        ) -> Result<Vec<TasteBook>, UsersServiceError> {
            Ok(vec![])
        }
        async fn list_by_book_ids(
            &self,
            _user_id: Uuid,
            _book_ids: &[i32],
        ) -> Result<Vec<TasteBook>, UsersServiceError> {
            Ok(vec![])
        }
        async fn get(
            &self,
            _user_id: Uuid,
            _book_id: i32,
        ) -> Result<Option<TasteBook>, UsersServiceError> {
            Ok(self.taste.clone())
        }
        async fn upsert(&self, _taste: &TasteBook) -> Result<bool, UsersServiceError> {
            Ok(self.upsert_returns)
        }
        async fn delete(
            &self,
            _user_id: Uuid,
            _book_id: i32,
        ) -> Result<bool, UsersServiceError> {
            Ok(self.delete_returns)
        }
    }

    struct MockLibrary {
        has_book: bool,
        has_tag: bool,
    }

    impl LibraryQueryPort for MockLibrary {
        async fn has_book(&self, _book_id: i32) -> Result<bool, UsersServiceError> {
            Ok(self.has_book)
        }
        async fn has_book_tag(
            &self,
            _tag_kind: &str,
            _tag_name: &str,
        ) -> Result<bool, UsersServiceError> {
            Ok(self.has_tag)
        }
    }

    #[tokio::test]
    async fn should_create_book_taste_when_book_exists() {
        let uc = CreateTasteBookUseCase {
            repo: MockTasteBookRepo {
                taste: None,
                upsert_returns: true,
                delete_returns: false,
            },
            library: MockLibrary {
                has_book: true,
                has_tag: false,
            },
        };
        let result = uc
            .execute(
                Uuid::now_v7(),
                CreateTasteBookInput {
                    book_id: 1,
                    is_dislike: false,
                },
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_return_book_not_found_when_book_missing() {
        let uc = CreateTasteBookUseCase {
            repo: MockTasteBookRepo {
                taste: None,
                upsert_returns: true,
                delete_returns: false,
            },
            library: MockLibrary {
                has_book: false,
                has_tag: false,
            },
        };
        let result = uc
            .execute(
                Uuid::now_v7(),
                CreateTasteBookInput {
                    book_id: 999,
                    is_dislike: false,
                },
            )
            .await;
        assert!(matches!(result, Err(UsersServiceError::BookNotFound)));
    }

    #[tokio::test]
    async fn should_return_taste_already_exists_when_same_is_dislike() {
        let uc = CreateTasteBookUseCase {
            repo: MockTasteBookRepo {
                taste: None,
                upsert_returns: false, // no change
                delete_returns: false,
            },
            library: MockLibrary {
                has_book: true,
                has_tag: false,
            },
        };
        let result = uc
            .execute(
                Uuid::now_v7(),
                CreateTasteBookInput {
                    book_id: 1,
                    is_dislike: false,
                },
            )
            .await;
        assert!(matches!(result, Err(UsersServiceError::TasteAlreadyExists)));
    }

    #[tokio::test]
    async fn should_update_taste_when_different_is_dislike() {
        let uc = CreateTasteBookUseCase {
            repo: MockTasteBookRepo {
                taste: None,
                upsert_returns: true, // value changed
                delete_returns: false,
            },
            library: MockLibrary {
                has_book: true,
                has_tag: false,
            },
        };
        let result = uc
            .execute(
                Uuid::now_v7(),
                CreateTasteBookInput {
                    book_id: 1,
                    is_dislike: true,
                },
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_delete_taste_when_exists() {
        let uc = DeleteTasteBookUseCase {
            repo: MockTasteBookRepo {
                taste: None,
                upsert_returns: false,
                delete_returns: true,
            },
        };
        let result = uc.execute(Uuid::now_v7(), 1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_return_taste_not_found_on_delete_missing() {
        let uc = DeleteTasteBookUseCase {
            repo: MockTasteBookRepo {
                taste: None,
                upsert_returns: false,
                delete_returns: false,
            },
        };
        let result = uc.execute(Uuid::now_v7(), 999).await;
        assert!(matches!(result, Err(UsersServiceError::TasteNotFound)));
    }
}
