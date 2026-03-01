use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use uuid::Uuid;

use madome_auth_types::identity::IdentityHeaders;

use crate::error::UsersServiceError;
use crate::state::AppState;
use crate::usecase::fcm_token::{CreateFcmTokenInput, CreateFcmTokenUseCase};

// ── POST /users/@me/fcm-token ────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateFcmTokenRequest {
    pub udid: Uuid,
    pub fcm_token: String,
}

pub async fn create_fcm_token(
    identity: IdentityHeaders,
    State(state): State<AppState>,
    Json(body): Json<CreateFcmTokenRequest>,
) -> Result<StatusCode, UsersServiceError> {
    let usecase = CreateFcmTokenUseCase {
        repo: state.fcm_token_repo(),
    };
    usecase
        .execute(
            identity.user_id,
            CreateFcmTokenInput {
                id: body.udid,
                token: body.fcm_token,
            },
        )
        .await?;
    Ok(StatusCode::CREATED)
}
