use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use madome_auth_types::identity::IdentityHeaders;

use crate::domain::types::HistorySortBy;
use crate::error::UsersServiceError;
use crate::state::AppState;
use crate::usecase::history::{
    CreateHistoryInput, CreateHistoryUseCase, DeleteHistoryUseCase, GetHistoriesUseCase,
    GetHistoryUseCase,
};

// ── Response types ───────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HistoryResponse {
    Book {
        book_id: i32,
        page: i32,
        #[serde(serialize_with = "madome_core::serde::to_rfc3339_ms")]
        created_at: chrono::DateTime<chrono::Utc>,
        #[serde(serialize_with = "madome_core::serde::to_rfc3339_ms")]
        updated_at: chrono::DateTime<chrono::Utc>,
    },
}

// ── Query params ─────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct HistoryListQuery {
    pub per_page: Option<u32>,
    pub page: Option<u32>,
    pub kind: Option<String>,
    pub sort_by: Option<String>,
}

// ── GET /users/@me/histories ─────────────────────────────────────────────────

pub async fn get_histories(
    identity: IdentityHeaders,
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<HistoryListQuery>,
) -> Result<Json<Vec<HistoryResponse>>, UsersServiceError> {
    let sort_by = query
        .sort_by
        .as_deref()
        .map(HistorySortBy::from_kebab_case)
        .unwrap_or(Some(HistorySortBy::default()))
        .unwrap_or_default();

    let page = madome_domain::pagination::PageRequest {
        per_page: query.per_page.unwrap_or(25),
        page: query.page.unwrap_or(1),
    };

    let usecase = GetHistoriesUseCase {
        repo: state.history_repo(),
    };
    let histories = usecase.execute(identity.user_id, sort_by, page).await?;
    let items = histories
        .into_iter()
        .map(|history| HistoryResponse::Book {
            book_id: history.book_id,
            page: history.page,
            created_at: history.created_at,
            updated_at: history.updated_at,
        })
        .collect();
    Ok(Json(items))
}

// ── GET /users/@me/histories/{kind}/{value} ──────────────────────────────────

pub async fn get_history(
    identity: IdentityHeaders,
    State(state): State<AppState>,
    Path((kind, value)): Path<(String, String)>,
) -> Result<Json<HistoryResponse>, UsersServiceError> {
    match kind.as_str() {
        "book" => {
            let book_id: i32 = value.parse().map_err(|_| UsersServiceError::MissingData)?;
            let usecase = GetHistoryUseCase {
                repo: state.history_repo(),
            };
            let history = usecase.execute(identity.user_id, book_id).await?;
            Ok(Json(HistoryResponse::Book {
                book_id: history.book_id,
                page: history.page,
                created_at: history.created_at,
                updated_at: history.updated_at,
            }))
        }
        _ => Err(UsersServiceError::MissingData),
    }
}

// ── POST /users/@me/histories ────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CreateHistoryRequest {
    Book { book_id: i32, page: Option<i32> },
}

pub async fn create_history(
    identity: IdentityHeaders,
    State(state): State<AppState>,
    Json(body): Json<CreateHistoryRequest>,
) -> Result<StatusCode, UsersServiceError> {
    match body {
        CreateHistoryRequest::Book { book_id, page } => {
            let usecase = CreateHistoryUseCase {
                repo: state.history_repo(),
            };
            usecase
                .execute(
                    identity.user_id,
                    CreateHistoryInput {
                        book_id,
                        page: page.unwrap_or(1),
                    },
                )
                .await?;
        }
    }
    Ok(StatusCode::CREATED)
}

// ── DELETE /users/@me/histories ──────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DeleteHistoryRequest {
    Book { book_id: i32 },
}

pub async fn delete_history(
    identity: IdentityHeaders,
    State(state): State<AppState>,
    Json(body): Json<DeleteHistoryRequest>,
) -> Result<StatusCode, UsersServiceError> {
    match body {
        DeleteHistoryRequest::Book { book_id } => {
            let usecase = DeleteHistoryUseCase {
                repo: state.history_repo(),
            };
            usecase.execute(identity.user_id, book_id).await?;
        }
    }
    Ok(StatusCode::NO_CONTENT)
}
