use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::{PublicKeyCredential, RegisterPublicKeyCredential};

use madome_auth_types::{
    cookie::{set_access_token_cookie, set_refresh_token_cookie},
    identity::IdentityHeaders,
};

use crate::error::AuthServiceError;
use crate::state::AppState;
use crate::usecase::passkey::{
    DeletePasskeyUseCase, FinishAuthenticationUseCase, FinishRegistrationUseCase,
    ListPasskeysUseCase, StartAuthenticationUseCase, StartRegistrationUseCase,
};

// ── GET /auth/passkeys ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct PasskeyResponse {
    pub credential_id: String,
    pub created_at: DateTime<Utc>,
}

pub async fn list_passkeys(
    State(state): State<AppState>,
    identity: IdentityHeaders,
) -> Result<Json<Vec<PasskeyResponse>>, AuthServiceError> {
    let usecase = ListPasskeysUseCase {
        passkeys: state.passkey_repo(),
    };
    let list = usecase.execute(identity.user_id).await?;
    let body: Vec<PasskeyResponse> = list
        .into_iter()
        .map(|passkey| PasskeyResponse {
            credential_id: URL_SAFE_NO_PAD.encode(&passkey.credential_id),
            created_at: passkey.created_at,
        })
        .collect();
    Ok(Json(body))
}

// ── DELETE /auth/passkeys/{credential_id} ─────────────────────────────────────

pub async fn delete_passkey(
    State(state): State<AppState>,
    identity: IdentityHeaders,
    Path(credential_id_b64): Path<String>,
) -> Result<StatusCode, AuthServiceError> {
    let credential_id = URL_SAFE_NO_PAD
        .decode(&credential_id_b64)
        .map_err(|_| AuthServiceError::InvalidCredential)?;

    let usecase = DeletePasskeyUseCase {
        passkeys: state.passkey_repo(),
    };
    usecase.execute(&credential_id, identity.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── POST /auth/passkey/registration ──────────────────────────────────────────

pub async fn start_registration(
    State(state): State<AppState>,
    identity: IdentityHeaders,
) -> Result<impl IntoResponse, AuthServiceError> {
    let usecase = StartRegistrationUseCase {
        users: state.user_repo(),
        passkeys: state.passkey_repo(),
        cache: state.passkey_cache(),
        webauthn: state.webauthn.clone(),
    };
    let out = usecase.execute(identity.user_id).await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("x-madome-passkey-registration-id"),
        HeaderValue::from_str(&out.registration_id).unwrap(),
    );

    Ok((StatusCode::OK, headers, Json(out.challenge)))
}

// ── PATCH /auth/passkey/registration?registration-id={id} ────────────────────

#[derive(Deserialize)]
pub struct RegistrationQuery {
    #[serde(rename = "registration-id")]
    pub registration_id: String,
}

pub async fn finish_registration(
    State(state): State<AppState>,
    identity: IdentityHeaders,
    Query(query): Query<RegistrationQuery>,
    Json(credential): Json<RegisterPublicKeyCredential>,
) -> Result<StatusCode, AuthServiceError> {
    let usecase = FinishRegistrationUseCase {
        passkeys: state.passkey_repo(),
        cache: state.passkey_cache(),
        webauthn: state.webauthn.clone(),
    };
    usecase
        .execute(identity.user_id, &query.registration_id, credential)
        .await?;
    Ok(StatusCode::CREATED)
}

// ── POST /auth/passkey/authentication?email={email} ───────────────────────────

#[derive(Deserialize)]
pub struct StartAuthQuery {
    pub email: String,
}

pub async fn start_authentication(
    State(state): State<AppState>,
    Query(query): Query<StartAuthQuery>,
) -> Result<impl IntoResponse, AuthServiceError> {
    let usecase = StartAuthenticationUseCase {
        users: state.user_repo(),
        passkeys: state.passkey_repo(),
        cache: state.passkey_cache(),
        webauthn: state.webauthn.clone(),
    };
    let out = usecase.execute(&query.email).await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("x-madome-passkey-authentication-id"),
        HeaderValue::from_str(&out.authentication_id).unwrap(),
    );

    Ok((StatusCode::OK, headers, Json(out.challenge)))
}

// ── PATCH /auth/passkey/authentication?authentication-id={id}&email={email} ───

#[derive(Deserialize)]
pub struct FinishAuthQuery {
    #[serde(rename = "authentication-id")]
    pub authentication_id: String,
    pub email: String,
}

pub async fn finish_authentication(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<FinishAuthQuery>,
    Json(credential): Json<PublicKeyCredential>,
) -> Result<impl IntoResponse, AuthServiceError> {
    let usecase = FinishAuthenticationUseCase {
        users: state.user_repo(),
        passkeys: state.passkey_repo(),
        cache: state.passkey_cache(),
        webauthn: state.webauthn.clone(),
        jwt_secret: state.jwt_secret.clone(),
    };
    let out = usecase
        .execute(&query.email, &query.authentication_id, credential)
        .await?;

    let jar = set_access_token_cookie(jar, out.access_token, state.cookie_domain.clone());
    let jar = set_refresh_token_cookie(jar, out.refresh_token, state.cookie_domain.clone());

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("x-madome-access-token-expires"),
        HeaderValue::from_str(&out.access_token_exp.to_string()).unwrap(),
    );

    Ok((StatusCode::CREATED, jar, headers))
}
