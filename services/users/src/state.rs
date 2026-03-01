use sea_orm::DatabaseConnection;

use crate::infra::db::{
    DbFcmTokenRepository, DbHistoryRepository, DbNotificationRepository, DbRenewBookPort,
    DbTasteRepository, DbUserRepository,
};
use crate::infra::grpc::GrpcLibraryClient;

/// Shared application state passed to every handler via axum `State`.
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub library_client: GrpcLibraryClient,
}

impl AppState {
    pub fn user_repo(&self) -> DbUserRepository {
        DbUserRepository {
            db: self.db.clone(),
        }
    }

    pub fn taste_repo(&self) -> DbTasteRepository {
        DbTasteRepository {
            db: self.db.clone(),
        }
    }

    pub fn history_repo(&self) -> DbHistoryRepository {
        DbHistoryRepository {
            db: self.db.clone(),
        }
    }

    pub fn notification_repo(&self) -> DbNotificationRepository {
        DbNotificationRepository {
            db: self.db.clone(),
        }
    }

    pub fn renew_book_port(&self) -> DbRenewBookPort {
        DbRenewBookPort {
            db: self.db.clone(),
        }
    }

    pub fn fcm_token_repo(&self) -> DbFcmTokenRepository {
        DbFcmTokenRepository {
            db: self.db.clone(),
        }
    }

    pub fn library_client(&self) -> GrpcLibraryClient {
        self.library_client.clone()
    }
}
