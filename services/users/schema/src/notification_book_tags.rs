use sea_orm::entity::prelude::*;

/// Tag associated with a book notification.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "notification_book_tags")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub notification_book_id: Uuid,
    pub tag_kind: String,
    pub tag_name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::notification_books::Entity",
        from = "Column::NotificationBookId",
        to = "super::notification_books::Column::Id"
    )]
    NotificationBook,
}

impl Related<super::notification_books::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NotificationBook.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
