use chrono::Utc;
use uuid::Uuid;

use crate::domain::repository::UserRepository;
use crate::domain::types::{User, validate_handle};
use crate::error::UsersServiceError;

// ── CreateUser ───────────────────────────────────────────────────────────────

pub struct CreateUserInput {
    pub name: String,
    pub handle: String,
    pub email: String,
    pub role: u8,
}

pub struct CreateUserUseCase<R: UserRepository> {
    pub repo: R,
}

impl<R: UserRepository> CreateUserUseCase<R> {
    pub async fn execute(&self, input: CreateUserInput) -> Result<(), UsersServiceError> {
        if !validate_handle(&input.handle) {
            return Err(UsersServiceError::InvalidHandle);
        }
        let now = Utc::now();
        let user = User {
            id: Uuid::now_v7(),
            name: input.name,
            handle: input.handle,
            email: input.email,
            role: input.role,
            created_at: now,
            updated_at: now,
        };
        self.repo.create(&user).await
    }
}

// ── GetUser ──────────────────────────────────────────────────────────────────

pub struct GetUserUseCase<R: UserRepository> {
    pub repo: R,
}

impl<R: UserRepository> GetUserUseCase<R> {
    pub async fn execute(&self, user_id: Uuid) -> Result<User, UsersServiceError> {
        self.repo
            .find_by_id(user_id)
            .await?
            .ok_or(UsersServiceError::UserNotFound)
    }
}

// ── UpdateUser ───────────────────────────────────────────────────────────────

pub struct UpdateUserInput {
    pub name: Option<String>,
    pub handle: Option<String>,
}

pub struct UpdateUserUseCase<R: UserRepository> {
    pub repo: R,
}

impl<R: UserRepository> UpdateUserUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        input: UpdateUserInput,
    ) -> Result<(), UsersServiceError> {
        if input.name.is_none() && input.handle.is_none() {
            return Err(UsersServiceError::MissingData);
        }
        if let Some(ref handle) = input.handle {
            if !validate_handle(handle) {
                return Err(UsersServiceError::InvalidHandle);
            }
        }
        self.repo
            .update_name_handle(user_id, input.name.as_deref(), input.handle.as_deref())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    struct MockUserRepo {
        user: Option<User>,
        create_called: std::sync::Mutex<bool>,
    }

    impl UserRepository for MockUserRepo {
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<User>, UsersServiceError> {
            Ok(self.user.clone())
        }
        async fn create(&self, _user: &User) -> Result<(), UsersServiceError> {
            *self.create_called.lock().unwrap() = true;
            Ok(())
        }
        async fn update_name_handle(
            &self,
            _id: Uuid,
            _name: Option<&str>,
            _handle: Option<&str>,
        ) -> Result<(), UsersServiceError> {
            Ok(())
        }
    }

    fn test_user() -> User {
        User {
            id: Uuid::now_v7(),
            name: "alice".into(),
            handle: "alice".into(),
            email: "alice@example.com".into(),
            role: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn should_return_invalid_handle_for_reserved_me() {
        let usecase = CreateUserUseCase {
            repo: MockUserRepo {
                user: None,
                create_called: std::sync::Mutex::new(false),
            },
        };
        let result = usecase
            .execute(CreateUserInput {
                name: "me".into(),
                handle: "me".into(),
                email: "me@example.com".into(),
                role: 0,
            })
            .await;
        assert!(matches!(result, Err(UsersServiceError::InvalidHandle)));
    }

    #[tokio::test]
    async fn should_create_user_with_valid_handle() {
        let repo = MockUserRepo {
            user: None,
            create_called: std::sync::Mutex::new(false),
        };
        let usecase = CreateUserUseCase { repo };
        let result = usecase
            .execute(CreateUserInput {
                name: "alice".into(),
                handle: "alice-123".into(),
                email: "alice@example.com".into(),
                role: 0,
            })
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_return_missing_data_when_both_fields_none() {
        let usecase = UpdateUserUseCase {
            repo: MockUserRepo {
                user: Some(test_user()),
                create_called: std::sync::Mutex::new(false),
            },
        };
        let result = usecase
            .execute(
                Uuid::now_v7(),
                UpdateUserInput {
                    name: None,
                    handle: None,
                },
            )
            .await;
        assert!(matches!(result, Err(UsersServiceError::MissingData)));
    }

    #[tokio::test]
    async fn should_return_user_not_found() {
        let usecase = GetUserUseCase {
            repo: MockUserRepo {
                user: None,
                create_called: std::sync::Mutex::new(false),
            },
        };
        let result = usecase.execute(Uuid::now_v7()).await;
        assert!(matches!(result, Err(UsersServiceError::UserNotFound)));
    }
}
