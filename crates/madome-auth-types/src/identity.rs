//! Gateway-injected identity headers extractor.

use axum::extract::FromRequestParts;
use http::StatusCode;
use http::request::Parts;
use uuid::Uuid;

/// User identity injected by the gateway via `x-madome-user-id` and `x-madome-user-role` headers.
///
/// Returns 401 if `x-madome-user-id` is absent or cannot be parsed as UUID.
/// Role enforcement (403) is done by handlers after extraction.
#[derive(Debug, Clone)]
pub struct IdentityHeaders {
    pub user_id: Uuid,
    pub user_role: u8,
}

impl<S> FromRequestParts<S> for IdentityHeaders
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    // axum-core 0.5 defines this as `fn -> impl Future + Send` (not `async fn`).
    // In Rust 1.82+ precise capturing, `async fn` captures lifetimes differently,
    // causing E0195. Fix: extract values synchronously, return a 'static async move block.
    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let user_id = parts
            .headers
            .get("x-madome-user-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<Uuid>().ok());

        let user_role = parts
            .headers
            .get("x-madome-user-role")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u8>().ok());

        async move {
            let user_id = user_id.ok_or(StatusCode::UNAUTHORIZED)?;
            let user_role = user_role.ok_or(StatusCode::UNAUTHORIZED)?;
            Ok(Self { user_id, user_role })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::FromRequestParts;
    use http::Request;

    async fn extract_identity(headers: Vec<(&str, &str)>) -> Result<IdentityHeaders, StatusCode> {
        let mut builder = Request::builder().method("GET").uri("/test");
        for (name, value) in headers {
            builder = builder.header(name, value);
        }
        let request = builder.body(()).unwrap();
        let (mut parts, _body) = request.into_parts();
        IdentityHeaders::from_request_parts(&mut parts, &()).await
    }

    #[tokio::test]
    async fn should_extract_valid_identity_headers() {
        let user_id = Uuid::new_v4();
        let result = extract_identity(vec![
            ("x-madome-user-id", &user_id.to_string()),
            ("x-madome-user-role", "1"),
        ])
        .await;

        let identity = result.unwrap();
        assert_eq!(identity.user_id, user_id);
        assert_eq!(identity.user_role, 1);
    }

    #[tokio::test]
    async fn should_reject_missing_user_id() {
        let result = extract_identity(vec![("x-madome-user-role", "0")]).await;
        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn should_reject_invalid_uuid() {
        let result = extract_identity(vec![
            ("x-madome-user-id", "not-a-uuid"),
            ("x-madome-user-role", "0"),
        ])
        .await;
        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn should_reject_missing_user_role() {
        let user_id = Uuid::new_v4();
        let result = extract_identity(vec![("x-madome-user-id", &user_id.to_string())]).await;
        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn should_reject_invalid_user_role() {
        let user_id = Uuid::new_v4();
        let result = extract_identity(vec![
            ("x-madome-user-id", &user_id.to_string()),
            ("x-madome-user-role", "abc"),
        ])
        .await;
        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }
}
