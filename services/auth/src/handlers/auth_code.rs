use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;

use crate::error::AuthServiceError;
use crate::state::AppState;
use crate::usecase::authcode::{CreateAuthcodeInput, CreateAuthcodeUseCase};

#[derive(Deserialize)]
pub struct CreateAuthcodeRequest {
    pub email: String,
}

pub async fn create_authcode(
    State(state): State<AppState>,
    Json(body): Json<CreateAuthcodeRequest>,
) -> Result<StatusCode, AuthServiceError> {
    let uc = CreateAuthcodeUseCase {
        users: state.user_repo(),
        auth_codes: state.auth_code_repo(),
    };
    uc.execute(CreateAuthcodeInput { email: body.email })
        .await?;
    Ok(StatusCode::CREATED)
}
