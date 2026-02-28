use sea_orm::entity::prelude::*;

/// Minimal user record owned by the auth service.
/// Stores only the fields needed for authentication (email lookup, role assertion).
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub email: String,
    pub role: i16,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::auth_codes::Entity")]
    AuthCodes,
    #[sea_orm(has_many = "super::passkeys::Entity")]
    Passkeys,
}

impl Related<super::auth_codes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AuthCodes.def()
    }
}

impl Related<super::passkeys::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Passkeys.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
