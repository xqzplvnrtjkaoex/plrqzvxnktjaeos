use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use madome_auth_types::{
    cookie::{
        MADOME_ACCESS_TOKEN, MADOME_REFRESH_TOKEN, clear_cookies, set_access_token_cookie,
        set_refresh_token_cookie,
    },
    identity::IdentityHeaders,
    token::validate_access_token,
};

use crate::error::AuthServiceError;
use crate::state::AppState;
use crate::usecase::token::{CreateTokenInput, CreateTokenUseCase, RefreshTokenUseCase};

const X_MADOME_ACCESS_TOKEN_EXPIRES: &str = "x-madome-access-token-expires";

fn token_expires_header(exp: u64) -> (HeaderName, HeaderValue) {
    (
        HeaderName::from_static(X_MADOME_ACCESS_TOKEN_EXPIRES),
        HeaderValue::from_str(&exp.to_string()).unwrap(),
    )
}

// ── GET /auth/token ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CheckTokenQuery {
    pub role: Option<u8>,
}

#[derive(Serialize)]
pub struct CheckTokenResponse {
    pub user_id: uuid::Uuid,
    pub user_role: u8,
    pub access_token_exp: u64,
}

pub async fn check_token(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<CheckTokenQuery>,
) -> Result<impl IntoResponse, AuthServiceError> {
    let token_value = jar
        .get(MADOME_ACCESS_TOKEN)
        .map(|c| c.value().to_owned())
        .ok_or(AuthServiceError::InvalidToken)?;

    let info = validate_access_token(&token_value, &state.jwt_secret)
        .map_err(|_| AuthServiceError::InvalidToken)?;

    if let Some(min_role) = query.role {
        if info.user_role < min_role {
            return Err(AuthServiceError::InvalidToken);
        }
    }

    let body = CheckTokenResponse {
        user_id: info.user_id,
        user_role: info.user_role,
        access_token_exp: info.access_token_exp,
    };

    let mut headers = HeaderMap::new();
    let (name, value) = token_expires_header(info.access_token_exp);
    headers.insert(name, value);

    Ok((StatusCode::OK, headers, Json(body)))
}

// ── POST /auth/token ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateTokenRequest {
    pub email: String,
    pub code: String,
}

pub async fn create_token(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<CreateTokenRequest>,
) -> Result<impl IntoResponse, AuthServiceError> {
    let usecase = CreateTokenUseCase {
        users: state.user_repo(),
        auth_codes: state.auth_code_repo(),
        jwt_secret: state.jwt_secret.clone(),
    };

    let out = usecase
        .execute(CreateTokenInput {
            email: body.email,
            code: body.code,
        })
        .await?;

    let jar = set_access_token_cookie(jar, out.access_token, state.cookie_domain.clone());
    let jar = set_refresh_token_cookie(jar, out.refresh_token, state.cookie_domain.clone());

    let mut headers = HeaderMap::new();
    let (name, value) = token_expires_header(out.access_token_exp);
    headers.insert(name, value);

    Ok((StatusCode::CREATED, jar, headers))
}

// ── PATCH /auth/token ─────────────────────────────────────────────────────────

pub async fn refresh_token(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, AuthServiceError> {
    let refresh_value = jar
        .get(MADOME_REFRESH_TOKEN)
        .map(|c| c.value().to_owned())
        .ok_or(AuthServiceError::InvalidRefreshToken)?;

    let usecase = RefreshTokenUseCase {
        users: state.user_repo(),
        jwt_secret: state.jwt_secret.clone(),
    };

    let out = usecase.execute(&refresh_value).await?;

    let jar = set_access_token_cookie(jar, out.access_token, state.cookie_domain.clone());
    let jar = set_refresh_token_cookie(jar, out.refresh_token, state.cookie_domain.clone());

    let mut headers = HeaderMap::new();
    let (name, value) = token_expires_header(out.access_token_exp);
    headers.insert(name, value);

    Ok((StatusCode::CREATED, jar, headers))
}

// ── DELETE /auth/token ────────────────────────────────────────────────────────

pub async fn revoke_token(
    State(state): State<AppState>,
    _identity: IdentityHeaders,
    jar: CookieJar,
) -> Result<impl IntoResponse, AuthServiceError> {
    let jar = clear_cookies(jar, state.cookie_domain.clone());
    Ok((StatusCode::NO_CONTENT, jar))
}
