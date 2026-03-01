use std::sync::Arc;

use sea_orm::Database;
use tracing::info;
use url::Url;
use webauthn_rs::prelude::WebauthnBuilder;

use madome_auth::config::AuthConfig;
use madome_auth::infra::grpc::GrpcUserPort;
use madome_auth::router::build_router;
use madome_auth::state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = AuthConfig::from_env();

    let db = Database::connect(&config.database_url)
        .await
        .expect("failed to connect to database");

    let redis_cfg = deadpool_redis::Config::from_url(&config.redis_url);
    let redis = redis_cfg
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("failed to create Redis pool");

    let rp_origin = Url::parse(&config.webauthn_origin).expect("invalid WEBAUTHN_ORIGIN");
    let webauthn = WebauthnBuilder::new(&config.webauthn_rp_id, &rp_origin)
        .expect("invalid WebAuthn configuration")
        .rp_name("Madome")
        .build()
        .expect("failed to build Webauthn");

    let users_channel = tonic::transport::Channel::from_shared(config.users_grpc_url.clone())
        .expect("invalid USERS_GRPC_URL")
        .connect_lazy();

    let state = AppState {
        db,
        redis,
        webauthn: Arc::new(webauthn),
        jwt_secret: config.jwt_secret,
        cookie_domain: config.cookie_domain,
        user_port: GrpcUserPort::new(users_channel),
    };

    let router = build_router(state);
    let addr = format!("0.0.0.0:{}", config.auth_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind");

    info!("auth service listening on {addr}");
    axum::serve(listener, router).await.expect("server error");
}
