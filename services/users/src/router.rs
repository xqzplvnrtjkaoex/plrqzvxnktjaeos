use axum::{
    Router,
    routing::{delete, get, patch, post},
};

use madome_core::health::{healthz, readyz};

use crate::handlers::{
    fcm_token::create_fcm_token,
    history::{create_history, delete_history, get_histories, get_history},
    notification::get_notifications,
    taste::{create_taste, delete_taste, get_taste, get_tastes},
    user::{create_user, get_me, update_me},
};
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        // Health
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        // Users
        .route("/users", post(create_user))
        .route("/users/@me", get(get_me))
        .route("/users/@me", patch(update_me))
        // Tastes
        .route("/users/@me/tastes", get(get_tastes))
        .route("/users/@me/tastes/{kind}/{value}", get(get_taste))
        .route("/users/@me/tastes", post(create_taste))
        .route("/users/@me/tastes", delete(delete_taste))
        // Histories
        .route("/users/@me/histories", get(get_histories))
        .route("/users/@me/histories/{kind}/{value}", get(get_history))
        .route("/users/@me/histories", post(create_history))
        .route("/users/@me/histories", delete(delete_history))
        // Notifications
        .route("/users/@me/notifications", get(get_notifications))
        // FCM token
        .route("/users/@me/fcm-token", post(create_fcm_token))
        .with_state(state)
}
