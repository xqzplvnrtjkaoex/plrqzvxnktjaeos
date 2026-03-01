use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use madome_auth_types::identity::IdentityHeaders;

use crate::domain::types::NotificationSortBy;
use crate::error::UsersServiceError;
use crate::state::AppState;
use crate::usecase::notification::GetNotificationsUseCase;

// ── Response types ───────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NotificationResponse {
    Book {
        id: String,
        book_id: i32,
        book_tags: Vec<NotificationTagResponse>,
        #[serde(serialize_with = "madome_core::serde::to_rfc3339_ms")]
        created_at: chrono::DateTime<chrono::Utc>,
    },
}

#[derive(Serialize)]
pub struct NotificationTagResponse {
    pub kind: String,
    pub name: String,
}

// ── Query params ─────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct NotificationListQuery {
    pub per_page: Option<u32>,
    pub page: Option<u32>,
    pub kind: Option<String>,
    pub sort_by: Option<String>,
}

// ── GET /users/@me/notifications ─────────────────────────────────────────────

pub async fn get_notifications(
    identity: IdentityHeaders,
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<NotificationListQuery>,
) -> Result<Json<Vec<NotificationResponse>>, UsersServiceError> {
    let sort_by = query
        .sort_by
        .as_deref()
        .map(NotificationSortBy::from_kebab)
        .unwrap_or(Some(NotificationSortBy::default()))
        .unwrap_or_default();

    let page = madome_domain::pagination::PageRequest {
        per_page: query.per_page.unwrap_or(25),
        page: query.page.unwrap_or(1),
    };

    let uc = GetNotificationsUseCase {
        repo: state.notification_repo(),
    };
    let notifications = uc.execute(identity.user_id, sort_by, page).await?;
    let items = notifications
        .into_iter()
        .map(|n| NotificationResponse::Book {
            id: n.id.to_string(),
            book_id: n.book_id,
            book_tags: n
                .book_tags
                .into_iter()
                .map(|(kind, name)| NotificationTagResponse { kind, name })
                .collect(),
            created_at: n.created_at,
        })
        .collect();
    Ok(Json(items))
}
