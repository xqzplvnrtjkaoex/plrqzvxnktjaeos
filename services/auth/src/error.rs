use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Auth service error variants mapped to HTTP status codes.
#[derive(Debug, thiserror::Error)]
pub enum AuthServiceError {
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("too many requests")]
    TooManyRequests,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AuthServiceError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}
