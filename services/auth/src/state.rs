use std::sync::Arc;

use deadpool_redis::Pool as RedisPool;
use sea_orm::DatabaseConnection;
use webauthn_rs::Webauthn;

use crate::infra::cache::RedisPasskeyCache;
use crate::infra::db::{DbAuthCodeRepository, DbPasskeyRepository, DbUserRepository};

/// Shared application state passed to every handler via axum `State`.
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis: RedisPool,
    pub webauthn: Arc<Webauthn>,
    pub jwt_secret: String,
    pub cookie_domain: String,
}

impl AppState {
    pub fn user_repo(&self) -> DbUserRepository {
        DbUserRepository {
            db: self.db.clone(),
        }
    }

    pub fn auth_code_repo(&self) -> DbAuthCodeRepository {
        DbAuthCodeRepository {
            db: self.db.clone(),
        }
    }

    pub fn passkey_repo(&self) -> DbPasskeyRepository {
        DbPasskeyRepository {
            db: self.db.clone(),
        }
    }

    pub fn passkey_cache(&self) -> RedisPasskeyCache {
        RedisPasskeyCache {
            pool: self.redis.clone(),
        }
    }
}
