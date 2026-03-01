use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Common application error variants.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("conflict")]
    Conflict,
    #[error("internal server error")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Forbidden => "FORBIDDEN",
            Self::NotFound => "NOT_FOUND",
            Self::Conflict => "CONFLICT",
            Self::Internal(_) => "INTERNAL",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Conflict => StatusCode::CONFLICT,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        // Log 500s only — tower-http TraceLayer already records method/uri/status for all
        // requests. 4xx are expected client errors; logging them here would be noise.
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

    #[test]
    fn unauthorized_returns_401() {
        let response = AppError::Unauthorized.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn forbidden_returns_403() {
        let response = AppError::Forbidden.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn not_found_returns_404() {
        let response = AppError::NotFound.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn conflict_returns_409() {
        let response = AppError::Conflict.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn internal_returns_500() {
        let err = AppError::Internal(anyhow::anyhow!("something went wrong"));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn unauthorized_json_body() {
        let resp = AppError::Unauthorized.into_response();
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["kind"], "UNAUTHORIZED");
        assert_eq!(json["message"], "unauthorized");
    }

    #[tokio::test]
    async fn forbidden_json_body() {
        let resp = AppError::Forbidden.into_response();
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["kind"], "FORBIDDEN");
        assert_eq!(json["message"], "forbidden");
    }

    #[tokio::test]
    async fn not_found_json_body() {
        let resp = AppError::NotFound.into_response();
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["kind"], "NOT_FOUND");
        assert_eq!(json["message"], "not found");
    }

    #[tokio::test]
    async fn conflict_json_body() {
        let resp = AppError::Conflict.into_response();
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["kind"], "CONFLICT");
        assert_eq!(json["message"], "conflict");
    }

    #[tokio::test]
    async fn internal_json_body() {
        let resp = AppError::Internal(anyhow::anyhow!("db error")).into_response();
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["kind"], "INTERNAL");
        assert_eq!(json["message"], "internal server error");
    }
}
