use sea_orm::entity::prelude::*;

/// User profile record owned by the users service.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub name: String,
    #[sea_orm(unique)]
    pub handle: String,
    #[sea_orm(unique)]
    pub email: String,
    pub role: i16,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::taste_books::Entity")]
    TasteBooks,
    #[sea_orm(has_many = "super::taste_book_tags::Entity")]
    TasteBookTags,
    #[sea_orm(has_many = "super::history_books::Entity")]
    HistoryBooks,
    #[sea_orm(has_many = "super::notification_books::Entity")]
    NotificationBooks,
    #[sea_orm(has_many = "super::fcm_tokens::Entity")]
    FcmTokens,
}

impl Related<super::taste_books::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TasteBooks.def()
    }
}

impl Related<super::taste_book_tags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TasteBookTags.def()
    }
}

impl Related<super::history_books::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::HistoryBooks.def()
    }
}

impl Related<super::notification_books::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NotificationBooks.def()
    }
}

impl Related<super::fcm_tokens::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FcmTokens.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
