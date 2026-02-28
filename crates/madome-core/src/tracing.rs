use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize structured stdout tracing. Call once at service startup.
/// Uses JSON format with env-filter (`RUST_LOG` env var).
///
/// Safe to call multiple times — subsequent calls are silently ignored.
pub fn init_tracing() {
    let _ = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer().json())
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_tracing_twice_does_not_panic() {
        init_tracing();
        init_tracing();
    }
}
