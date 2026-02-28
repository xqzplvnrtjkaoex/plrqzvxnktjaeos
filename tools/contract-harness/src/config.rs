//! Contract harness configuration loaded from environment variables.

/// All configuration for the Docker-based contract harness.
///
/// Loaded from env vars after `dotenv::dotenv().ok()`; no CLI parsing.
/// All values have safe defaults suitable for local development.
#[derive(Debug)]
pub struct ContractHarnessConfig {
    /// Docker daemon URL (`DOCKER_HOST`).
    /// default: `"unix:///var/run/docker.sock"`
    pub docker_host: String,

    /// HMAC secret for signing JWTs (`JWT_SECRET`).
    /// default: `"test-contract-secret"`
    pub jwt_secret: String,

    /// WebAuthn relying-party ID (`WEBAUTHN_RP_ID`).
    /// default: `"localhost"`
    pub webauthn_rp_id: String,

    /// WebAuthn relying-party origin URL (`WEBAUTHN_ORIGIN`).
    /// default: `"http://localhost"`
    pub webauthn_origin: String,

    /// Cookie domain attribute (`COOKIE_DOMAIN`).
    /// default: `"localhost"`
    pub cookie_domain: String,
}

impl ContractHarnessConfig {
    pub fn from_env() -> Self {
        Self {
            docker_host: std::env::var("DOCKER_HOST")
                .unwrap_or_else(|_| "unix:///var/run/docker.sock".to_owned()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "test-contract-secret".to_owned()),
            webauthn_rp_id: std::env::var("WEBAUTHN_RP_ID")
                .unwrap_or_else(|_| "localhost".to_owned()),
            webauthn_origin: std::env::var("WEBAUTHN_ORIGIN")
                .unwrap_or_else(|_| "http://localhost".to_owned()),
            cookie_domain: std::env::var("COOKIE_DOMAIN")
                .unwrap_or_else(|_| "localhost".to_owned()),
        }
    }
}
