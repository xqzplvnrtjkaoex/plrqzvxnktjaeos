use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Users service domain error variants.
#[derive(Debug, thiserror::Error)]
pub enum UsersServiceError {
    #[error("user not found")]
    UserNotFound,
    #[error("taste not found")]
    TasteNotFound,
    #[error("history not found")]
    HistoryNotFound,
    #[error("book not found")]
    BookNotFound,
    #[error("book tag not found")]
    BookTagNotFound,
    #[error("user already exists")]
    UserAlreadyExists,
    #[error("taste already exists")]
    TasteAlreadyExists,
    #[error("invalid handle")]
    InvalidHandle,
    #[error("missing data")]
    MissingData,
    #[error("forbidden")]
    Forbidden,
    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}

impl UsersServiceError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::UserNotFound => "USER_NOT_FOUND",
            Self::TasteNotFound => "TASTE_NOT_FOUND",
            Self::HistoryNotFound => "HISTORY_NOT_FOUND",
            Self::BookNotFound => "BOOK_NOT_FOUND",
            Self::BookTagNotFound => "BOOK_TAG_NOT_FOUND",
            Self::UserAlreadyExists => "USER_ALREADY_EXISTS",
            Self::TasteAlreadyExists => "TASTE_ALREADY_EXISTS",
            Self::InvalidHandle => "INVALID_HANDLE",
            Self::MissingData => "MISSING_DATA",
            Self::Forbidden => "FORBIDDEN",
            Self::Internal(_) => "INTERNAL",
        }
    }
}

impl IntoResponse for UsersServiceError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::UserNotFound
            | Self::TasteNotFound
            | Self::HistoryNotFound
            | Self::BookNotFound
            | Self::BookTagNotFound => StatusCode::NOT_FOUND,
            Self::UserAlreadyExists | Self::TasteAlreadyExists => StatusCode::CONFLICT,
            Self::InvalidHandle | Self::MissingData => StatusCode::BAD_REQUEST,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        if let Self::Internal(ref e) = self {
            tracing::error!(error = %e, kind = "INTERNAL", "internal error");
        }
        let body = serde_json::json!({
            "kind": self.kind(),
            "message": self.to_string(),
        });
        (status, axum::Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::response::IntoResponse;

    async fn assert_error(
        error: UsersServiceError,
        expected_status: StatusCode,
        expected_kind: &str,
        expected_message: &str,
    ) {
        let resp = error.into_response();
        assert_eq!(resp.status(), expected_status);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["kind"], expected_kind);
        assert_eq!(json["message"], expected_message);
    }

    #[tokio::test]
    async fn should_return_user_not_found() {
        assert_error(
            UsersServiceError::UserNotFound,
            StatusCode::NOT_FOUND,
            "USER_NOT_FOUND",
            "user not found",
        )
        .await;
    }

    #[tokio::test]
    async fn should_return_taste_not_found() {
        assert_error(
            UsersServiceError::TasteNotFound,
            StatusCode::NOT_FOUND,
            "TASTE_NOT_FOUND",
            "taste not found",
        )
        .await;
    }

    #[tokio::test]
    async fn should_return_history_not_found() {
        assert_error(
            UsersServiceError::HistoryNotFound,
            StatusCode::NOT_FOUND,
            "HISTORY_NOT_FOUND",
            "history not found",
        )
        .await;
    }

    #[tokio::test]
    async fn should_return_book_not_found() {
        assert_error(
            UsersServiceError::BookNotFound,
            StatusCode::NOT_FOUND,
            "BOOK_NOT_FOUND",
            "book not found",
        )
        .await;
    }

    #[tokio::test]
    async fn should_return_book_tag_not_found() {
        assert_error(
            UsersServiceError::BookTagNotFound,
            StatusCode::NOT_FOUND,
            "BOOK_TAG_NOT_FOUND",
            "book tag not found",
        )
        .await;
    }

    #[tokio::test]
    async fn should_return_user_already_exists() {
        assert_error(
            UsersServiceError::UserAlreadyExists,
            StatusCode::CONFLICT,
            "USER_ALREADY_EXISTS",
            "user already exists",
        )
        .await;
    }

    #[tokio::test]
    async fn should_return_taste_already_exists() {
        assert_error(
            UsersServiceError::TasteAlreadyExists,
            StatusCode::CONFLICT,
            "TASTE_ALREADY_EXISTS",
            "taste already exists",
        )
        .await;
    }

    #[tokio::test]
    async fn should_return_invalid_handle() {
        assert_error(
            UsersServiceError::InvalidHandle,
            StatusCode::BAD_REQUEST,
            "INVALID_HANDLE",
            "invalid handle",
        )
        .await;
    }

    #[tokio::test]
    async fn should_return_missing_data() {
        assert_error(
            UsersServiceError::MissingData,
            StatusCode::BAD_REQUEST,
            "MISSING_DATA",
            "missing data",
        )
        .await;
    }

    #[tokio::test]
    async fn should_return_forbidden() {
        assert_error(
            UsersServiceError::Forbidden,
            StatusCode::FORBIDDEN,
            "FORBIDDEN",
            "forbidden",
        )
        .await;
    }

    #[tokio::test]
    async fn should_return_internal() {
        assert_error(
            UsersServiceError::Internal(anyhow::anyhow!("db error")),
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL",
            "internal error",
        )
        .await;
    }
}
