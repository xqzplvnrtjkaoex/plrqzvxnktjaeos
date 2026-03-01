/// Users service configuration loaded from environment variables.
#[derive(Debug)]
pub struct UsersConfig {
    /// PostgreSQL connection URL.
    pub database_url: String,
    /// TCP port for the HTTP server (default 3113). Env var: `USERS_PORT`.
    pub users_port: u16,
    /// TCP port for the gRPC server (default 50051). Env var: `USERS_GRPC_PORT`.
    pub users_grpc_port: u16,
    /// gRPC endpoint for the library service (e.g. "http://library:50051").
    pub library_grpc_url: String,
}

impl UsersConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL"),
            users_port: std::env::var("USERS_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3113),
            users_grpc_port: std::env::var("USERS_GRPC_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50051),
            library_grpc_url: std::env::var("LIBRARY_GRPC_URL").expect("LIBRARY_GRPC_URL"),
        }
    }
}
