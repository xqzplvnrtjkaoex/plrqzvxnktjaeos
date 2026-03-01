use anyhow::Context as _;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DatabaseTransaction,
    EntityTrait, QueryFilter, TransactionTrait,
};
use uuid::Uuid;

use madome_auth_schema::{auth_codes, outbox_events, passkeys};

use crate::domain::repository::{AuthCodeRepository, PasskeyRepository};
use crate::domain::types::{AuthCode, OutboxEvent, PasskeyRecord};
use crate::error::AuthServiceError;

// ── AuthCode repository ───────────────────────────────────────────────────────

#[derive(Clone)]
pub struct DbAuthCodeRepository {
    pub db: DatabaseConnection,
}

impl AuthCodeRepository for DbAuthCodeRepository {
    async fn count_active(&self, user_id: Uuid) -> Result<u64, AuthServiceError> {
        use sea_orm::PaginatorTrait;
        let now = Utc::now();
        let count = auth_codes::Entity::find()
            .filter(auth_codes::Column::UserId.eq(user_id))
            .filter(auth_codes::Column::UsedAt.is_null())
            .filter(auth_codes::Column::ExpiresAt.gt(now))
            .count(&self.db)
            .await
            .context("count active authcodes")?;
        Ok(count)
    }

    async fn create_with_outbox(
        &self,
        code: &AuthCode,
        event: &OutboxEvent,
    ) -> Result<(), AuthServiceError> {
        self.db
            .transaction::<_, (), sea_orm::DbErr>(|txn| {
                let code = code.clone();
                let event = event.clone();
                Box::pin(async move {
                    insert_auth_code(txn, &code).await?;
                    insert_outbox_event(txn, &event).await?;
                    Ok(())
                })
            })
            .await
            .context("create authcode with outbox")?;
        Ok(())
    }

    async fn find_valid(
        &self,
        user_id: Uuid,
        code: &str,
    ) -> Result<Option<AuthCode>, AuthServiceError> {
        let now = Utc::now();
        let model = auth_codes::Entity::find()
            .filter(auth_codes::Column::UserId.eq(user_id))
            .filter(auth_codes::Column::Code.eq(code))
            .filter(auth_codes::Column::UsedAt.is_null())
            .filter(auth_codes::Column::ExpiresAt.gt(now))
            .one(&self.db)
            .await
            .context("find valid authcode")?;
        Ok(model.map(authcode_from_model))
    }

    async fn mark_used(&self, id: Uuid) -> Result<(), AuthServiceError> {
        let now = Utc::now();
        auth_codes::ActiveModel {
            id: Set(id),
            used_at: Set(Some(now)),
            ..Default::default()
        }
        .update(&self.db)
        .await
        .context("mark authcode used")?;
        Ok(())
    }
}

async fn insert_auth_code(
    txn: &DatabaseTransaction,
    code: &AuthCode,
) -> Result<(), sea_orm::DbErr> {
    auth_codes::ActiveModel {
        id: Set(code.id),
        user_id: Set(code.user_id),
        code: Set(code.code.clone()),
        expires_at: Set(code.expires_at),
        used_at: Set(None),
        created_at: Set(code.created_at),
    }
    .insert(txn)
    .await?;
    Ok(())
}

async fn insert_outbox_event(
    txn: &DatabaseTransaction,
    event: &OutboxEvent,
) -> Result<(), sea_orm::DbErr> {
    let now = Utc::now();
    outbox_events::ActiveModel {
        id: Set(event.id),
        kind: Set(event.kind.clone()),
        payload: Set(event.payload.clone()),
        idempotency_key: Set(event.idempotency_key.clone()),
        attempts: Set(0),
        last_error: Set(None),
        created_at: Set(now),
        next_attempt_at: Set(now),
        processed_at: Set(None),
        failed_at: Set(None),
    }
    .insert(txn)
    .await?;
    Ok(())
}

fn authcode_from_model(model: auth_codes::Model) -> AuthCode {
    AuthCode {
        id: model.id,
        user_id: model.user_id,
        code: model.code,
        expires_at: model.expires_at,
        used_at: model.used_at,
        created_at: model.created_at,
    }
}

// ── Passkey repository ────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct DbPasskeyRepository {
    pub db: DatabaseConnection,
}

impl PasskeyRepository for DbPasskeyRepository {
    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<PasskeyRecord>, AuthServiceError> {
        let models = passkeys::Entity::find()
            .filter(passkeys::Column::UserId.eq(user_id))
            .all(&self.db)
            .await
            .context("list passkeys by user")?;
        Ok(models.into_iter().map(passkey_from_model).collect())
    }

    async fn find_by_id(
        &self,
        credential_id: &[u8],
    ) -> Result<Option<PasskeyRecord>, AuthServiceError> {
        let model = passkeys::Entity::find_by_id(credential_id.to_vec())
            .one(&self.db)
            .await
            .context("find passkey by id")?;
        Ok(model.map(passkey_from_model))
    }

    async fn create(&self, record: &PasskeyRecord) -> Result<(), AuthServiceError> {
        passkeys::ActiveModel {
            credential_id: Set(record.credential_id.clone()),
            user_id: Set(record.user_id),
            aaguid: Set(record.aaguid),
            credential: Set(record.credential.clone()),
            created_at: Set(record.created_at),
        }
        .insert(&self.db)
        .await
        .context("create passkey")?;
        Ok(())
    }

    async fn delete(&self, credential_id: &[u8], user_id: Uuid) -> Result<bool, AuthServiceError> {
        let result = passkeys::Entity::delete_many()
            .filter(passkeys::Column::CredentialId.eq(credential_id.to_vec()))
            .filter(passkeys::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .context("delete passkey")?;
        Ok(result.rows_affected > 0)
    }

    async fn update_credential(
        &self,
        credential_id: &[u8],
        credential: &[u8],
    ) -> Result<(), AuthServiceError> {
        passkeys::ActiveModel {
            credential_id: Set(credential_id.to_vec()),
            credential: Set(credential.to_vec()),
            ..Default::default()
        }
        .update(&self.db)
        .await
        .context("update passkey credential")?;
        Ok(())
    }
}

fn passkey_from_model(model: passkeys::Model) -> PasskeyRecord {
    PasskeyRecord {
        credential_id: model.credential_id,
        user_id: model.user_id,
        aaguid: model.aaguid,
        credential: model.credential,
        created_at: model.created_at,
    }
}
