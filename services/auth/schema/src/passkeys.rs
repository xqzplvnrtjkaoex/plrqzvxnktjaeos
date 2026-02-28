use sea_orm::entity::prelude::*;

/// WebAuthn passkey credential stored for a user.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "passkeys")]
pub struct Model {
    /// Raw credential ID bytes (primary key, as provided by the authenticator).
    #[sea_orm(primary_key, auto_increment = false)]
    pub credential_id: Vec<u8>,
    pub user_id: Uuid,
    /// AAGUID from the authenticator's attestation data.
    pub aaguid: Uuid,
    /// JSON-serialized `webauthn_rs::Passkey` (counter updates are persisted here).
    pub credential: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
