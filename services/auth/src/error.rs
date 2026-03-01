use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Auth service domain error variants.
#[derive(Debug, thiserror::Error)]
pub enum AuthServiceError {
    #[error("user not found")]
    UserNotFound,
    #[error("credential not found")]
    CredentialNotFound,
    #[error("invalid authcode")]
    InvalidAuthcode,
    #[error("invalid token")]
    InvalidToken,
    #[error("invalid refresh token")]
    InvalidRefreshToken,
    #[error("session expired")]
    InvalidSession,
    #[error("invalid credential")]
    InvalidCredential,
    #[error("too many authcodes")]
    TooManyAuthcodes,
    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}

impl AuthServiceError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::UserNotFound => "USER_NOT_FOUND",
            Self::CredentialNotFound => "CREDENTIAL_NOT_FOUND",
            Self::InvalidAuthcode => "INVALID_AUTHCODE",
            Self::InvalidToken => "INVALID_TOKEN",
            Self::InvalidRefreshToken => "INVALID_REFRESH_TOKEN",
            Self::InvalidSession => "INVALID_SESSION",
            Self::InvalidCredential => "INVALID_CREDENTIAL",
            Self::TooManyAuthcodes => "TOO_MANY_AUTHCODES",
            Self::Internal(_) => "INTERNAL",
        }
    }
}

impl IntoResponse for AuthServiceError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::UserNotFound | Self::CredentialNotFound => StatusCode::NOT_FOUND,
            Self::InvalidAuthcode
            | Self::InvalidToken
            | Self::InvalidRefreshToken
            | Self::InvalidSession => StatusCode::UNAUTHORIZED,
            Self::InvalidCredential => StatusCode::BAD_REQUEST,
            Self::TooManyAuthcodes => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        // Log 500s only — tower-http TraceLayer already records method/uri/status for all
        // requests. 4xx are expected client errors; logging them here would be noise.
        // Internal errors need the anyhow chain logged so the root cause is traceable.
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

    #[tokio::test]
    async fn should_return_user_not_found() {
        let resp = AuthServiceError::UserNotFound.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["kind"], "USER_NOT_FOUND");
        assert_eq!(json["message"], "user not found");
    }

    #[tokio::test]
    async fn should_return_credential_not_found() {
        let resp = AuthServiceError::CredentialNotFound.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["kind"], "CREDENTIAL_NOT_FOUND");
        assert_eq!(json["message"], "credential not found");
    }

    #[tokio::test]
    async fn should_return_invalid_authcode() {
        let resp = AuthServiceError::InvalidAuthcode.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["kind"], "INVALID_AUTHCODE");
        assert_eq!(json["message"], "invalid authcode");
    }

    #[tokio::test]
    async fn should_return_invalid_token() {
        let resp = AuthServiceError::InvalidToken.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["kind"], "INVALID_TOKEN");
        assert_eq!(json["message"], "invalid token");
    }

    #[tokio::test]
    async fn should_return_invalid_refresh_token() {
        let resp = AuthServiceError::InvalidRefreshToken.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["kind"], "INVALID_REFRESH_TOKEN");
        assert_eq!(json["message"], "invalid refresh token");
    }

    #[tokio::test]
    async fn should_return_invalid_session() {
        let resp = AuthServiceError::InvalidSession.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["kind"], "INVALID_SESSION");
        assert_eq!(json["message"], "session expired");
    }

    #[tokio::test]
    async fn should_return_invalid_credential() {
        let resp = AuthServiceError::InvalidCredential.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["kind"], "INVALID_CREDENTIAL");
        assert_eq!(json["message"], "invalid credential");
    }

    #[tokio::test]
    async fn should_return_too_many_authcodes() {
        let resp = AuthServiceError::TooManyAuthcodes.into_response();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["kind"], "TOO_MANY_AUTHCODES");
        assert_eq!(json["message"], "too many authcodes");
    }

    #[tokio::test]
    async fn should_return_internal() {
        let resp = AuthServiceError::Internal(anyhow::anyhow!("db error")).into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["kind"], "INTERNAL");
        assert_eq!(json["message"], "internal error");
    }
}
