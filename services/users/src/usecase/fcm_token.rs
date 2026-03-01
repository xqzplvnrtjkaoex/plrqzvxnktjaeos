use chrono::Utc;
use uuid::Uuid;

use crate::domain::repository::FcmTokenRepository;
use crate::domain::types::FcmToken;
use crate::error::UsersServiceError;

// ── CreateOrUpdateFcmToken ───────────────────────────────────────────────────

pub struct CreateFcmTokenInput {
    pub id: Uuid,
    pub token: String,
}

pub struct CreateFcmTokenUseCase<R: FcmTokenRepository> {
    pub repo: R,
}

impl<R: FcmTokenRepository> CreateFcmTokenUseCase<R> {
    pub async fn execute(
        &self,
        user_id: Uuid,
        input: CreateFcmTokenInput,
    ) -> Result<(), UsersServiceError> {
        let token = FcmToken {
            id: input.id,
            user_id,
            token: input.token,
            updated_at: Utc::now(),
        };
        self.repo.upsert(&token, user_id).await
    }
}
