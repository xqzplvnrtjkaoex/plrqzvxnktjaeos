use chrono::Utc;
use uuid::Uuid;

use madome_domain::pagination::PageRequest;

use crate::domain::repository::{LibraryQueryPort, TasteRepository};
use crate::domain::types::{Taste, TasteBook, TasteBookTag, TasteSortBy};
use crate::error::UsersServiceError;

// ── GetTaste (single book) ───────────────────────────────────────────────────

pub struct GetTasteBookUseCase<R: TasteRepository> {
    pub repo: R,
}

impl<R: TasteRepository> GetTasteBookUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        book_id: i32,
    ) -> Result<TasteBook, UsersServiceError> {
        self.repo
            .get_book(user_id, book_id)
            .await?
            .ok_or(UsersServiceError::TasteNotFound)
    }
}

// ── GetTaste (single book tag) ───────────────────────────────────────────────

pub struct GetTasteBookTagUseCase<R: TasteRepository> {
    pub repo: R,
}

impl<R: TasteRepository> GetTasteBookTagUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        tag_kind: &str,
        tag_name: &str,
    ) -> Result<TasteBookTag, UsersServiceError> {
        self.repo
            .get_book_tag(user_id, tag_kind, tag_name)
            .await?
            .ok_or(UsersServiceError::TasteNotFound)
    }
}

// ── GetTastes (combined list — UNION ALL) ───────────────────────────────────

pub struct GetTastesUseCase<R: TasteRepository> {
    pub repo: R,
}

impl<R: TasteRepository> GetTastesUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        sort_by: TasteSortBy,
        is_dislike: Option<bool>,
        page: PageRequest,
    ) -> Result<Vec<Taste>, UsersServiceError> {
        self.repo.list_all(user_id, sort_by, is_dislike, page).await
    }
}

// ── GetTastes (list books only) ─────────────────────────────────────────────

pub struct GetTasteBooksUseCase<R: TasteRepository> {
    pub repo: R,
}

impl<R: TasteRepository> GetTasteBooksUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        sort_by: TasteSortBy,
        is_dislike: Option<bool>,
        page: PageRequest,
    ) -> Result<Vec<TasteBook>, UsersServiceError> {
        self.repo
            .list_books(user_id, sort_by, is_dislike, page)
            .await
    }
}

// ── GetTastes (list book tags only) ─────────────────────────────────────────

pub struct GetTasteBookTagsUseCase<R: TasteRepository> {
    pub repo: R,
}

impl<R: TasteRepository> GetTasteBookTagsUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        sort_by: TasteSortBy,
        is_dislike: Option<bool>,
        page: PageRequest,
    ) -> Result<Vec<TasteBookTag>, UsersServiceError> {
        self.repo
            .list_book_tags(user_id, sort_by, is_dislike, page)
            .await
    }
}

// ── GetTastesByBookIds ───────────────────────────────────────────────────────

pub struct GetTastesByBookIdsUseCase<R: TasteRepository> {
    pub repo: R,
}

impl<R: TasteRepository> GetTastesByBookIdsUseCase<R> {
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

pub struct CreateTasteBookUseCase<R: TasteRepository, L: LibraryQueryPort> {
    pub repo: R,
    pub library: L,
}

impl<R: TasteRepository, L: LibraryQueryPort> CreateTasteBookUseCase<R, L> {
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
        let changed = self.repo.upsert_book(&taste).await?;
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

pub struct CreateTasteBookTagUseCase<R: TasteRepository, L: LibraryQueryPort> {
    pub repo: R,
    pub library: L,
}

impl<R: TasteRepository, L: LibraryQueryPort> CreateTasteBookTagUseCase<R, L> {
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
        let changed = self.repo.upsert_book_tag(&taste).await?;
        if !changed {
            return Err(UsersServiceError::TasteAlreadyExists);
        }
        Ok(())
    }
}

// ── DeleteTaste (book) ───────────────────────────────────────────────────────

pub struct DeleteTasteBookUseCase<R: TasteRepository> {
    pub repo: R,
}

impl<R: TasteRepository> DeleteTasteBookUseCase<R> {
    pub async fn execute(&self, user_id: Uuid, book_id: i32) -> Result<(), UsersServiceError> {
        let deleted = self.repo.delete_book(user_id, book_id).await?;
        if !deleted {
            return Err(UsersServiceError::TasteNotFound);
        }
        Ok(())
    }
}

// ── DeleteTaste (book tag) ───────────────────────────────────────────────────

pub struct DeleteTasteBookTagUseCase<R: TasteRepository> {
    pub repo: R,
}

impl<R: TasteRepository> DeleteTasteBookTagUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        tag_kind: &str,
        tag_name: &str,
    ) -> Result<(), UsersServiceError> {
        let deleted = self
            .repo
            .delete_book_tag(user_id, tag_kind, tag_name)
            .await?;
        if !deleted {
            return Err(UsersServiceError::TasteNotFound);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTasteRepo {
        taste: Option<TasteBook>,
        upsert_returns: bool,
        delete_returns: bool,
    }

    impl TasteRepository for MockTasteRepo {
        async fn list_all(
            &self,
            _user_id: Uuid,
            _sort_by: TasteSortBy,
            _is_dislike: Option<bool>,
            _page: PageRequest,
        ) -> Result<Vec<Taste>, UsersServiceError> {
            Ok(vec![])
        }
        async fn list_books(
            &self,
            _user_id: Uuid,
            _sort_by: TasteSortBy,
            _is_dislike: Option<bool>,
            _page: PageRequest,
        ) -> Result<Vec<TasteBook>, UsersServiceError> {
            Ok(vec![])
        }
        async fn list_book_tags(
            &self,
            _user_id: Uuid,
            _sort_by: TasteSortBy,
            _is_dislike: Option<bool>,
            _page: PageRequest,
        ) -> Result<Vec<TasteBookTag>, UsersServiceError> {
            Ok(vec![])
        }
        async fn list_by_book_ids(
            &self,
            _user_id: Uuid,
            _book_ids: &[i32],
        ) -> Result<Vec<TasteBook>, UsersServiceError> {
            Ok(vec![])
        }
        async fn get_book(
            &self,
            _user_id: Uuid,
            _book_id: i32,
        ) -> Result<Option<TasteBook>, UsersServiceError> {
            Ok(self.taste.clone())
        }
        async fn get_book_tag(
            &self,
            _user_id: Uuid,
            _tag_kind: &str,
            _tag_name: &str,
        ) -> Result<Option<TasteBookTag>, UsersServiceError> {
            Ok(None)
        }
        async fn upsert_book(&self, _taste: &TasteBook) -> Result<bool, UsersServiceError> {
            Ok(self.upsert_returns)
        }
        async fn upsert_book_tag(&self, _taste: &TasteBookTag) -> Result<bool, UsersServiceError> {
            Ok(self.upsert_returns)
        }
        async fn delete_book(
            &self,
            _user_id: Uuid,
            _book_id: i32,
        ) -> Result<bool, UsersServiceError> {
            Ok(self.delete_returns)
        }
        async fn delete_book_tag(
            &self,
            _user_id: Uuid,
            _tag_kind: &str,
            _tag_name: &str,
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
            repo: MockTasteRepo {
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
            repo: MockTasteRepo {
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
            repo: MockTasteRepo {
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
            repo: MockTasteRepo {
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
            repo: MockTasteRepo {
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
            repo: MockTasteRepo {
                taste: None,
                upsert_returns: false,
                delete_returns: false,
            },
        };
        let result = uc.execute(Uuid::now_v7(), 999).await;
        assert!(matches!(result, Err(UsersServiceError::TasteNotFound)));
    }
}
