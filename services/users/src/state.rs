use sea_orm::DatabaseConnection;

/// Shared application state passed to every handler via axum `State`.
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}
