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
        .map(NotificationSortBy::from_kebab_case)
        .unwrap_or(Some(NotificationSortBy::default()))
        .unwrap_or_default();

    let page = madome_domain::pagination::PageRequest {
        per_page: query.per_page.unwrap_or(25),
        page: query.page.unwrap_or(1),
    };

    let usecase = GetNotificationsUseCase {
        repo: state.notification_repo(),
    };
    let notifications = usecase.execute(identity.user_id, sort_by, page).await?;
    let items = notifications
        .into_iter()
        .map(|notification| NotificationResponse::Book {
            id: notification.id.to_string(),
            book_id: notification.book_id,
            book_tags: notification
                .book_tags
                .into_iter()
                .map(|(kind, name)| NotificationTagResponse { kind, name })
                .collect(),
            created_at: notification.created_at,
        })
        .collect();
    Ok(Json(items))
}
