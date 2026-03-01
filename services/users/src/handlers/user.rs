use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use madome_auth_types::identity::IdentityHeaders;

use crate::error::UsersServiceError;
use crate::state::AppState;
use crate::usecase::user::{
    CreateUserInput, CreateUserUseCase, GetUserUseCase, UpdateUserInput, UpdateUserUseCase,
};

// ── POST /users ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub handle: String,
    pub email: String,
    pub role: Option<u8>,
}

pub async fn create_user(
    identity: IdentityHeaders,
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> Result<StatusCode, UsersServiceError> {
    if identity.user_role < 2 {
        return Err(UsersServiceError::Forbidden);
    }
    let role = body.role.unwrap_or(0);
    if role > 1 {
        return Err(UsersServiceError::Forbidden);
    }
    let usecase = CreateUserUseCase {
        repo: state.user_repo(),
    };
    usecase
        .execute(CreateUserInput {
            name: body.name,
            handle: body.handle,
            email: body.email,
            role,
        })
        .await?;
    Ok(StatusCode::CREATED)
}

// ── GET /users/@me ───────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub handle: String,
    pub email: String,
    pub role: u8,
    #[serde(serialize_with = "madome_core::serde::to_rfc3339_ms")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(serialize_with = "madome_core::serde::to_rfc3339_ms")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn get_me(
    identity: IdentityHeaders,
    State(state): State<AppState>,
) -> Result<Json<UserResponse>, UsersServiceError> {
    let usecase = GetUserUseCase {
        repo: state.user_repo(),
    };
    let user = usecase.execute(identity.user_id).await?;
    Ok(Json(UserResponse {
        id: user.id.to_string(),
        name: user.name,
        handle: user.handle,
        email: user.email,
        role: user.role,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }))
}

// ── PATCH /users/@me ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct UpdateMeRequest {
    pub name: Option<String>,
    pub handle: Option<String>,
}

pub async fn update_me(
    identity: IdentityHeaders,
    State(state): State<AppState>,
    Json(body): Json<UpdateMeRequest>,
) -> Result<StatusCode, UsersServiceError> {
    let usecase = UpdateUserUseCase {
        repo: state.user_repo(),
    };
    usecase
        .execute(
            identity.user_id,
            UpdateUserInput {
                name: body.name,
                handle: body.handle,
            },
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
