//! Cookie builders for access and refresh tokens.
//!
//! All cookie attributes match the legacy system exactly (Compat requirement).

use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use time::Duration;

/// Cookie name for the access token.
pub const MADOME_ACCESS_TOKEN: &str = "madome_access_token";

/// Cookie name for the refresh token.
pub const MADOME_REFRESH_TOKEN: &str = "madome_refresh_token";

/// Access-token JWT lifetime in seconds (4 hours).
pub const ACCESS_TOKEN_EXP: u64 = 14400;

/// Cookie Max-Age for both tokens in seconds (7 days).
pub const REFRESH_TOKEN_EXP: u64 = 604800;

/// Set the access-token cookie on the jar.
///
/// ```
/// use axum_extra::extract::cookie::CookieJar;
/// use madome_auth_types::cookie::{set_access_token_cookie, MADOME_ACCESS_TOKEN};
///
/// let jar = CookieJar::new();
/// let jar = set_access_token_cookie(jar, "token_value".to_string(), "example.com".to_string());
/// let cookie = jar.get(MADOME_ACCESS_TOKEN).unwrap();
/// assert_eq!(cookie.path(), Some("/"));
/// assert_eq!(cookie.domain(), Some("example.com"));
/// assert_eq!(cookie.max_age(), Some(time::Duration::seconds(604800)));
/// assert!(cookie.http_only().unwrap_or(false));
/// assert!(cookie.secure().unwrap_or(false));
/// ```
pub fn set_access_token_cookie(jar: CookieJar, value: String, domain: String) -> CookieJar {
    let cookie = Cookie::build((MADOME_ACCESS_TOKEN, value))
        .path("/")
        .domain(domain)
        .max_age(Duration::seconds(REFRESH_TOKEN_EXP as i64))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .build();
    jar.add(cookie)
}

/// Set the refresh-token cookie on the jar.
///
/// ```
/// use axum_extra::extract::cookie::CookieJar;
/// use madome_auth_types::cookie::{set_refresh_token_cookie, MADOME_REFRESH_TOKEN};
///
/// let jar = CookieJar::new();
/// let jar = set_refresh_token_cookie(jar, "refresh_value".to_string(), "example.com".to_string());
/// let cookie = jar.get(MADOME_REFRESH_TOKEN).unwrap();
/// assert_eq!(cookie.path(), Some("/auth/token"));
/// assert_eq!(cookie.domain(), Some("example.com"));
/// assert_eq!(cookie.max_age(), Some(time::Duration::seconds(604800)));
/// assert!(cookie.http_only().unwrap_or(false));
/// assert!(cookie.secure().unwrap_or(false));
/// ```
pub fn set_refresh_token_cookie(jar: CookieJar, value: String, domain: String) -> CookieJar {
    let cookie = Cookie::build((MADOME_REFRESH_TOKEN, value))
        .path("/auth/token")
        .domain(domain)
        .max_age(Duration::seconds(REFRESH_TOKEN_EXP as i64))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .build();
    jar.add(cookie)
}

/// Clear both token cookies by setting Max-Age to 0.
///
/// ```
/// use axum_extra::extract::cookie::CookieJar;
/// use madome_auth_types::cookie::{
///     clear_cookies, set_access_token_cookie, set_refresh_token_cookie,
///     MADOME_ACCESS_TOKEN, MADOME_REFRESH_TOKEN,
/// };
///
/// let jar = CookieJar::new();
/// let jar = set_access_token_cookie(jar, "a".to_string(), "example.com".to_string());
/// let jar = set_refresh_token_cookie(jar, "r".to_string(), "example.com".to_string());
/// let jar = clear_cookies(jar, "example.com".to_string());
/// let access = jar.get(MADOME_ACCESS_TOKEN).unwrap();
/// let refresh = jar.get(MADOME_REFRESH_TOKEN).unwrap();
/// assert_eq!(access.max_age(), Some(time::Duration::ZERO));
/// assert_eq!(refresh.max_age(), Some(time::Duration::ZERO));
/// ```
pub fn clear_cookies(jar: CookieJar, domain: String) -> CookieJar {
    let access = Cookie::build((MADOME_ACCESS_TOKEN, ""))
        .path("/")
        .domain(domain.clone())
        .max_age(Duration::ZERO)
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .build();
    let refresh = Cookie::build((MADOME_REFRESH_TOKEN, ""))
        .path("/auth/token")
        .domain(domain)
        .max_age(Duration::ZERO)
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .build();
    jar.add(access).add(refresh)
}
