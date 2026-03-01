/// Auth service configuration loaded from environment variables.
#[derive(Debug)]
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
    pub auth_port: u16,
    /// Users service gRPC URL (e.g. "http://users:50051"). Env var: `USERS_GRPC_URL`.
    pub users_grpc_url: String,
}

impl AuthConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL"),
            redis_url: std::env::var("REDIS_URL").expect("REDIS_URL"),
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET"),
            webauthn_rp_id: std::env::var("WEBAUTHN_RP_ID").expect("WEBAUTHN_RP_ID"),
            webauthn_origin: std::env::var("WEBAUTHN_ORIGIN").expect("WEBAUTHN_ORIGIN"),
            cookie_domain: std::env::var("COOKIE_DOMAIN").expect("COOKIE_DOMAIN"),
            auth_port: std::env::var("AUTH_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3112),
            users_grpc_url: std::env::var("USERS_GRPC_URL").expect("USERS_GRPC_URL"),
        }
    }
}
