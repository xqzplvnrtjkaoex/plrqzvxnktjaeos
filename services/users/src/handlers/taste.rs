use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use madome_auth_types::identity::IdentityHeaders;

use crate::domain::types::{Taste, TasteSortBy};
use crate::error::UsersServiceError;
use crate::state::AppState;
use crate::usecase::taste::{
    CreateTasteBookInput, CreateTasteBookTagInput, CreateTasteBookTagUseCase,
    CreateTasteBookUseCase, DeleteTasteBookTagUseCase, DeleteTasteBookUseCase,
    GetTasteBookTagUseCase, GetTasteBookTagsUseCase, GetTasteBookUseCase, GetTasteBooksUseCase,
    GetTastesByBookIdsUseCase, GetTastesUseCase,
};

// ── Response types ───────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TasteResponse {
    Book {
        book_id: i32,
        is_dislike: bool,
        #[serde(serialize_with = "madome_core::serde::to_rfc3339_ms")]
        created_at: chrono::DateTime<chrono::Utc>,
    },
    BookTag {
        tag_kind: String,
        tag_name: String,
        is_dislike: bool,
        #[serde(serialize_with = "madome_core::serde::to_rfc3339_ms")]
        created_at: chrono::DateTime<chrono::Utc>,
    },
}

impl From<Taste> for TasteResponse {
    fn from(taste: Taste) -> Self {
        match taste {
            Taste::Book(b) => TasteResponse::Book {
                book_id: b.book_id,
                is_dislike: b.is_dislike,
                created_at: b.created_at,
            },
            Taste::BookTag(t) => TasteResponse::BookTag {
                tag_kind: t.tag_kind,
                tag_name: t.tag_name,
                is_dislike: t.is_dislike,
                created_at: t.created_at,
            },
        }
    }
}

// ── Query params ─────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct TasteListQuery {
    pub per_page: Option<u32>,
    pub page: Option<u32>,
    pub kind: Option<String>,
    pub sort_by: Option<String>,
    pub is_dislike: Option<bool>,
    #[serde(default)]
    pub book_ids: Vec<i32>,
    pub books_per_page: Option<u32>,
    pub books_page: Option<u32>,
    pub books_sort_by: Option<String>,
}

// ── GET /users/@me/tastes ────────────────────────────────────────────────────

pub async fn get_tastes(
    identity: IdentityHeaders,
    State(state): State<AppState>,
    axum::extract::RawQuery(raw_query): axum::extract::RawQuery,
) -> Result<Json<Vec<TasteResponse>>, UsersServiceError> {
    let query: TasteListQuery = raw_query
        .as_deref()
        .map(serde_qs::from_str)
        .transpose()
        .map_err(|_| UsersServiceError::MissingData)?
        .unwrap_or_default();

    // If book-ids[] present, dispatch to GetTastesByBookIds
    if !query.book_ids.is_empty() {
        let uc = GetTastesByBookIdsUseCase {
            repo: state.taste_repo(),
        };
        let tastes = uc.execute(identity.user_id, &query.book_ids).await?;
        let items = tastes
            .into_iter()
            .map(|t| TasteResponse::Book {
                book_id: t.book_id,
                is_dislike: t.is_dislike,
                created_at: t.created_at,
            })
            .collect();
        return Ok(Json(items));
    }

    let sort_by = query
        .sort_by
        .as_deref()
        .and_then(TasteSortBy::from_kebab)
        .unwrap_or_default();

    let page = madome_domain::pagination::PageRequest {
        per_page: query.per_page.unwrap_or(25),
        page: query.page.unwrap_or(1),
    };

    let kind = query.kind.as_deref();

    match kind {
        // No kind filter → combined UNION ALL query
        None => {
            let uc = GetTastesUseCase {
                repo: state.taste_repo(),
            };
            let tastes = uc
                .execute(identity.user_id, sort_by, query.is_dislike, page)
                .await?;
            let items = tastes.into_iter().map(TasteResponse::from).collect();
            Ok(Json(items))
        }
        Some("book") => {
            let uc = GetTasteBooksUseCase {
                repo: state.taste_repo(),
            };
            let tastes = uc
                .execute(identity.user_id, sort_by, query.is_dislike, page)
                .await?;
            let items = tastes
                .into_iter()
                .map(|t| TasteResponse::Book {
                    book_id: t.book_id,
                    is_dislike: t.is_dislike,
                    created_at: t.created_at,
                })
                .collect();
            Ok(Json(items))
        }
        Some("book_tag") => {
            let uc = GetTasteBookTagsUseCase {
                repo: state.taste_repo(),
            };
            let tastes = uc
                .execute(identity.user_id, sort_by, query.is_dislike, page)
                .await?;
            let items = tastes
                .into_iter()
                .map(|t| TasteResponse::BookTag {
                    tag_kind: t.tag_kind,
                    tag_name: t.tag_name,
                    is_dislike: t.is_dislike,
                    created_at: t.created_at,
                })
                .collect();
            Ok(Json(items))
        }
        Some(_) => Err(UsersServiceError::MissingData),
    }
}

// ── GET /users/@me/tastes/{kind}/{value} ─────────────────────────────────────

pub async fn get_taste(
    identity: IdentityHeaders,
    State(state): State<AppState>,
    Path((kind, value)): Path<(String, String)>,
) -> Result<Json<TasteResponse>, UsersServiceError> {
    match kind.as_str() {
        "book" => {
            let book_id: i32 = value.parse().map_err(|_| UsersServiceError::MissingData)?;
            let uc = GetTasteBookUseCase {
                repo: state.taste_repo(),
            };
            let taste = uc.execute(identity.user_id, book_id).await?;
            Ok(Json(TasteResponse::Book {
                book_id: taste.book_id,
                is_dislike: taste.is_dislike,
                created_at: taste.created_at,
            }))
        }
        "book-tag" => {
            let (tag_kind, tag_name) = parse_book_tag_value(&value)?;
            let uc = GetTasteBookTagUseCase {
                repo: state.taste_repo(),
            };
            let taste = uc.execute(identity.user_id, &tag_kind, &tag_name).await?;
            Ok(Json(TasteResponse::BookTag {
                tag_kind: taste.tag_kind,
                tag_name: taste.tag_name,
                is_dislike: taste.is_dislike,
                created_at: taste.created_at,
            }))
        }
        _ => Err(UsersServiceError::MissingData),
    }
}

/// Parse `{tag_kind}-{tag_name}` by splitting on first hyphen.
fn parse_book_tag_value(value: &str) -> Result<(String, String), UsersServiceError> {
    if let Some(idx) = value.find('-') {
        let kind = &value[..idx];
        let name = &value[idx + 1..];
        if !kind.is_empty() && !name.is_empty() {
            return Ok((kind.to_owned(), name.to_owned()));
        }
    }
    Err(UsersServiceError::MissingData)
}

// ── POST /users/@me/tastes ───────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CreateTasteRequest {
    Book {
        book_id: i32,
        #[serde(default)]
        is_dislike: bool,
    },
    BookTag {
        tag_kind: String,
        tag_name: String,
        #[serde(default)]
        is_dislike: bool,
    },
}

pub async fn create_taste(
    identity: IdentityHeaders,
    State(state): State<AppState>,
    Json(body): Json<CreateTasteRequest>,
) -> Result<StatusCode, UsersServiceError> {
    match body {
        CreateTasteRequest::Book {
            book_id,
            is_dislike,
        } => {
            let uc = CreateTasteBookUseCase {
                repo: state.taste_repo(),
                library: state.library_client(),
            };
            uc.execute(
                identity.user_id,
                CreateTasteBookInput {
                    book_id,
                    is_dislike,
                },
            )
            .await?;
        }
        CreateTasteRequest::BookTag {
            tag_kind,
            tag_name,
            is_dislike,
        } => {
            let uc = CreateTasteBookTagUseCase {
                repo: state.taste_repo(),
                library: state.library_client(),
            };
            uc.execute(
                identity.user_id,
                CreateTasteBookTagInput {
                    tag_kind,
                    tag_name,
                    is_dislike,
                },
            )
            .await?;
        }
    }
    Ok(StatusCode::CREATED)
}

// ── DELETE /users/@me/tastes ─────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DeleteTasteRequest {
    Book { book_id: i32 },
    BookTag { tag_kind: String, tag_name: String },
}

pub async fn delete_taste(
    identity: IdentityHeaders,
    State(state): State<AppState>,
    Json(body): Json<DeleteTasteRequest>,
) -> Result<StatusCode, UsersServiceError> {
    match body {
        DeleteTasteRequest::Book { book_id } => {
            let uc = DeleteTasteBookUseCase {
                repo: state.taste_repo(),
            };
            uc.execute(identity.user_id, book_id).await?;
        }
        DeleteTasteRequest::BookTag { tag_kind, tag_name } => {
            let uc = DeleteTasteBookTagUseCase {
                repo: state.taste_repo(),
            };
            uc.execute(identity.user_id, &tag_kind, &tag_name).await?;
        }
    }
    Ok(StatusCode::NO_CONTENT)
}
