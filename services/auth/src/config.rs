use madome_core::config::Config;
use serde::Deserialize;

/// Auth service configuration loaded from environment variables.
#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    /// PostgreSQL connection URL.
    pub database_url: String,
    /// Redis connection URL.
    pub redis_url: String,
    /// HMAC secret for signing JWT access and refresh tokens.
    pub jwt_secret: String,
    /// WebAuthn relying-party ID (e.g. "example.com").
    pub webauthn_rp_id: String,
    /// WebAuthn relying-party origin URL (e.g. "https://example.com").
    pub webauthn_origin: String,
    /// Cookie domain attribute (root domain, e.g. "example.com").
    pub cookie_domain: String,
    /// TCP port to listen on (default 3112). Env var: `AUTH_PORT`.
    #[serde(default = "default_port")]
    pub auth_port: u16,
}

fn default_port() -> u16 {
    3112
}

impl Config for AuthConfig {}
