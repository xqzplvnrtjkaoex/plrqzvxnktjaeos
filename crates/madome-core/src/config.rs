/// Trait for loading service configuration from environment variables.
///
/// Implementors should derive `serde::Deserialize` and then call
/// `Config::from_env()` to load configuration at startup.
///
/// # Panics
///
/// Panics if any required env var is missing or cannot be deserialized.
pub trait Config: Sized + serde::de::DeserializeOwned {
    fn from_env() -> Self {
        envy::from_env().expect("failed to load config from environment")
    }
}
