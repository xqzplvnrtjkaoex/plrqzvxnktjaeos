use axum::{
    Router,
    routing::{delete, get, patch, post},
};

use madome_core::health::{healthz, readyz};

use crate::handlers::{
    auth_code::create_authcode,
    passkeys::{
        delete_passkey, finish_authentication, finish_registration, list_passkeys,
        start_authentication, start_registration,
    },
    token::{check_token, create_token, refresh_token, revoke_token},
};
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        // Health
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        // Auth code
        .route("/auth/code", post(create_authcode))
        // Token
        .route("/auth/token", get(check_token))
        .route("/auth/token", post(create_token))
        .route("/auth/token", patch(refresh_token))
        .route("/auth/token", delete(revoke_token))
        // Passkeys
        .route("/auth/passkeys", get(list_passkeys))
        .route("/auth/passkeys/{credential_id}", delete(delete_passkey))
        // WebAuthn registration
        .route("/auth/passkey/registration", post(start_registration))
        .route("/auth/passkey/registration", patch(finish_registration))
        // WebAuthn authentication
        .route("/auth/passkey/authentication", post(start_authentication))
        .route("/auth/passkey/authentication", patch(finish_authentication))
        .with_state(state)
}
