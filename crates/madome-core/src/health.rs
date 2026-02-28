use axum::http::StatusCode;

/// Handler for `GET /healthz` — liveness check.
pub async fn healthz() -> StatusCode {
    StatusCode::OK
}

/// Handler for `GET /readyz` — readiness check (override per service as needed).
pub async fn readyz() -> StatusCode {
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn healthz_returns_200() {
        assert_eq!(healthz().await, StatusCode::OK);
    }

    #[tokio::test]
    async fn readyz_returns_200() {
        assert_eq!(readyz().await, StatusCode::OK);
    }
}
