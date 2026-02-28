//! Per-service contract runners.

/// Infrastructure URLs for test containers.
pub struct InfraUrls {
    pub database_url: String,
    pub redis_url: String,
}

#[cfg(feature = "auth")]
pub mod auth;
