use sea_orm::Database;
use tracing::info;

use madome_proto::notification::notification_service_server::NotificationServiceServer;
use madome_proto::user::user_service_server::UserServiceServer;

use madome_users::config::UsersConfig;
use madome_users::grpc_server::UsersGrpcServer;
use madome_users::infra::grpc::GrpcLibraryClient;
use madome_users::router::build_router;
use madome_users::state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = UsersConfig::from_env();

    let db = Database::connect(&config.database_url)
        .await
        .expect("failed to connect to database");

    let library_client = GrpcLibraryClient::connect(&config.library_grpc_url)
        .await
        .expect("failed to connect to library gRPC");

    let state = AppState {
        db,
        library_client,
    };

    // Spawn gRPC server
    let grpc_state = state.clone();
    let grpc_addr = format!("0.0.0.0:{}", config.users_grpc_port);
    tokio::spawn(async move {
        let server = UsersGrpcServer {
            state: grpc_state,
        };
        info!("users gRPC server listening on {grpc_addr}");
        tonic::transport::Server::builder()
            .add_service(UserServiceServer::new(server.clone()))
            .add_service(NotificationServiceServer::new(server))
            .serve(grpc_addr.parse().expect("invalid gRPC address"))
            .await
            .expect("gRPC server error");
    });

    // HTTP server
    let router = build_router(state);
    let http_addr = format!("0.0.0.0:{}", config.users_port);
    let listener = tokio::net::TcpListener::bind(&http_addr)
        .await
        .expect("failed to bind");

    info!("users service listening on {http_addr}");
    axum::serve(listener, router).await.expect("server error");
}
