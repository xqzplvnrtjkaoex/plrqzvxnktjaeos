use anyhow::Context as _;
use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel as _, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
    sea_query::OnConflict,
};
use uuid::Uuid;

use madome_core::sea_ext::OrderByRandom;
use madome_domain::pagination::{PageRequest, Sort};
use madome_users_schema::{
    fcm_tokens, history_books, notification_book_tags, notification_books, taste_book_tags,
    taste_books, users,
};

use crate::domain::repository::{
    FcmTokenRepository, HistoryRepository, NotificationRepository, RenewBookPort, TasteRepository,
    UserRepository,
};
use crate::domain::types::{
    FcmToken, HistoryBook, HistorySortBy, NotificationBook, NotificationSortBy, Taste, TasteBook,
    TasteBookTag, TasteSortBy, User,
};
use crate::error::UsersServiceError;

// ── User repository ──────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct DbUserRepository {
    pub db: DatabaseConnection,
}

impl UserRepository for DbUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, UsersServiceError> {
        let model = users::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .context("find user by id")?;
        Ok(model.map(user_from_model))
    }

    async fn create(&self, user: &User) -> Result<(), UsersServiceError> {
        users::ActiveModel {
            id: Set(user.id),
            name: Set(user.name.clone()),
            handle: Set(user.handle.clone()),
            email: Set(user.email.clone()),
            role: Set(user.role as i16),
            created_at: Set(user.created_at),
            updated_at: Set(user.updated_at),
        }
        .insert(&self.db)
        .await
        .context("create user")?;
        Ok(())
    }

    async fn update_name_handle(
        &self,
        id: Uuid,
        name: Option<&str>,
        handle: Option<&str>,
    ) -> Result<(), UsersServiceError> {
        let mut am = users::ActiveModel {
            id: Set(id),
            ..Default::default()
        };
        if let Some(new_name) = name {
            am.name = Set(new_name.to_owned());
        }
        if let Some(new_handle) = handle {
            am.handle = Set(new_handle.to_owned());
        }
        am.updated_at = Set(Utc::now());
        am.update(&self.db)
            .await
            .context("update user name/handle")?;
        Ok(())
    }
}

fn user_from_model(model: users::Model) -> User {
    User {
        id: model.id,
        name: model.name,
        handle: model.handle,
        email: model.email,
        role: model.role as u8,
        created_at: model.created_at,
        updated_at: model.updated_at,
    }
}

// ── Taste repository (unified) ──────────────────────────────────────────────

#[derive(Clone)]
pub struct DbTasteRepository {
    pub db: DatabaseConnection,
}

impl TasteRepository for DbTasteRepository {
    async fn list_all(
        &self,
        user_id: Uuid,
        sort_by: TasteSortBy,
        is_dislike: Option<bool>,
        page: PageRequest,
    ) -> Result<Vec<Taste>, UsersServiceError> {
        use sea_orm::{ConnectionTrait, FromQueryResult, Statement};

        let PageRequest { per_page, page } = page.clamped();
        let offset = ((page - 1) * per_page) as i64;
        let limit = per_page as i64;

        let sort_clause = match sort_by {
            TasteSortBy::CreatedAt(Sort::Desc) => "created_at DESC",
            TasteSortBy::CreatedAt(Sort::Asc) => "created_at ASC",
            TasteSortBy::Random => "RANDOM()",
        };
        let dislike_clause = match is_dislike {
            Some(v) => format!("AND is_dislike = {v}"),
            None => String::new(),
        };

        let sql = format!(
            r#"
            SELECT * FROM (
                SELECT user_id, book_id, NULL AS tag_kind, NULL AS tag_name, is_dislike, created_at
                    FROM taste_books
                    WHERE user_id = $1 {dislike_clause}
                UNION ALL
                SELECT user_id, NULL, tag_kind, tag_name, is_dislike, created_at
                    FROM taste_book_tags
                    WHERE user_id = $1 {dislike_clause}
            ) AS a
            ORDER BY {sort_clause}
            LIMIT $2 OFFSET $3
            "#,
        );

        #[derive(Debug, FromQueryResult)]
        struct TasteRow {
            book_id: Option<i32>,
            tag_kind: Option<String>,
            tag_name: Option<String>,
            is_dislike: bool,
            created_at: chrono::DateTime<chrono::Utc>,
        }

        let rows = TasteRow::find_by_statement(Statement::from_sql_and_values(
            self.db.get_database_backend(),
            &sql,
            [user_id.into(), limit.into(), offset.into()],
        ))
        .all(&self.db)
        .await
        .context("list all tastes (UNION ALL)")?;

        let tastes = rows
            .into_iter()
            .map(|row| {
                if let Some(book_id) = row.book_id {
                    Taste::Book(TasteBook {
                        user_id,
                        book_id,
                        is_dislike: row.is_dislike,
                        created_at: row.created_at,
                    })
                } else {
                    Taste::BookTag(TasteBookTag {
                        user_id,
                        tag_kind: row.tag_kind.unwrap_or_default(),
                        tag_name: row.tag_name.unwrap_or_default(),
                        is_dislike: row.is_dislike,
                        created_at: row.created_at,
                    })
                }
            })
            .collect();
        Ok(tastes)
    }

    async fn list_books(
        &self,
        user_id: Uuid,
        sort_by: TasteSortBy,
        is_dislike: Option<bool>,
        page: PageRequest,
    ) -> Result<Vec<TasteBook>, UsersServiceError> {
        let PageRequest { per_page, page } = page.clamped();
        let mut query = taste_books::Entity::find().filter(taste_books::Column::UserId.eq(user_id));
        if let Some(dislike) = is_dislike {
            query = query.filter(taste_books::Column::IsDislike.eq(dislike));
        }
        query = match sort_by {
            TasteSortBy::CreatedAt(Sort::Desc) => {
                query.order_by_desc(taste_books::Column::CreatedAt)
            }
            TasteSortBy::CreatedAt(Sort::Asc) => query.order_by_asc(taste_books::Column::CreatedAt),
            TasteSortBy::Random => query.order_by_random(),
        };
        let models = query
            .offset(((page - 1) * per_page) as u64)
            .limit(per_page as u64)
            .all(&self.db)
            .await
            .context("list taste books")?;
        Ok(models.into_iter().map(taste_book_from_model).collect())
    }

    async fn list_book_tags(
        &self,
        user_id: Uuid,
        sort_by: TasteSortBy,
        is_dislike: Option<bool>,
        page: PageRequest,
    ) -> Result<Vec<TasteBookTag>, UsersServiceError> {
        let PageRequest { per_page, page } = page.clamped();
        let mut query =
            taste_book_tags::Entity::find().filter(taste_book_tags::Column::UserId.eq(user_id));
        if let Some(dislike) = is_dislike {
            query = query.filter(taste_book_tags::Column::IsDislike.eq(dislike));
        }
        query = match sort_by {
            TasteSortBy::CreatedAt(Sort::Desc) => {
                query.order_by_desc(taste_book_tags::Column::CreatedAt)
            }
            TasteSortBy::CreatedAt(Sort::Asc) => {
                query.order_by_asc(taste_book_tags::Column::CreatedAt)
            }
            TasteSortBy::Random => query.order_by_random(),
        };
        let models = query
            .offset(((page - 1) * per_page) as u64)
            .limit(per_page as u64)
            .all(&self.db)
            .await
            .context("list taste book tags")?;
        Ok(models.into_iter().map(taste_book_tag_from_model).collect())
    }

    async fn list_by_book_ids(
        &self,
        user_id: Uuid,
        book_ids: &[i32],
    ) -> Result<Vec<TasteBook>, UsersServiceError> {
        let models = taste_books::Entity::find()
            .filter(taste_books::Column::UserId.eq(user_id))
            .filter(taste_books::Column::BookId.is_in(book_ids.iter().copied()))
            .all(&self.db)
            .await
            .context("list taste books by book ids")?;
        Ok(models.into_iter().map(taste_book_from_model).collect())
    }

    async fn get_book(
        &self,
        user_id: Uuid,
        book_id: i32,
    ) -> Result<Option<TasteBook>, UsersServiceError> {
        let model = taste_books::Entity::find_by_id((user_id, book_id))
            .one(&self.db)
            .await
            .context("get taste book")?;
        Ok(model.map(taste_book_from_model))
    }

    async fn get_book_tag(
        &self,
        user_id: Uuid,
        tag_kind: &str,
        tag_name: &str,
    ) -> Result<Option<TasteBookTag>, UsersServiceError> {
        let model = taste_book_tags::Entity::find_by_id((
            user_id,
            tag_kind.to_owned(),
            tag_name.to_owned(),
        ))
        .one(&self.db)
        .await
        .context("get taste book tag")?;
        Ok(model.map(taste_book_tag_from_model))
    }

    async fn upsert_book(&self, taste: &TasteBook) -> Result<bool, UsersServiceError> {
        let existing = taste_books::Entity::find_by_id((taste.user_id, taste.book_id))
            .one(&self.db)
            .await
            .context("find taste book for upsert")?;

        match existing {
            Some(row) if row.is_dislike == taste.is_dislike => Ok(false),
            Some(row) => {
                let mut taste_book = row.into_active_model();
                taste_book.is_dislike = Set(taste.is_dislike);
                taste_book
                    .update(&self.db)
                    .await
                    .context("update taste book")?;
                Ok(true)
            }
            None => {
                taste_books::ActiveModel {
                    user_id: Set(taste.user_id),
                    book_id: Set(taste.book_id),
                    is_dislike: Set(taste.is_dislike),
                    created_at: Set(taste.created_at),
                }
                .insert(&self.db)
                .await
                .context("insert taste book")?;
                Ok(true)
            }
        }
    }

    async fn upsert_book_tag(&self, taste: &TasteBookTag) -> Result<bool, UsersServiceError> {
        let existing = taste_book_tags::Entity::find_by_id((
            taste.user_id,
            taste.tag_kind.clone(),
            taste.tag_name.clone(),
        ))
        .one(&self.db)
        .await
        .context("find taste book tag for upsert")?;

        match existing {
            Some(row) if row.is_dislike == taste.is_dislike => Ok(false),
            Some(row) => {
                let mut taste_book_tag = row.into_active_model();
                taste_book_tag.is_dislike = Set(taste.is_dislike);
                taste_book_tag
                    .update(&self.db)
                    .await
                    .context("update taste book tag")?;
                Ok(true)
            }
            None => {
                taste_book_tags::ActiveModel {
                    user_id: Set(taste.user_id),
                    tag_kind: Set(taste.tag_kind.clone()),
                    tag_name: Set(taste.tag_name.clone()),
                    is_dislike: Set(taste.is_dislike),
                    created_at: Set(taste.created_at),
                }
                .insert(&self.db)
                .await
                .context("insert taste book tag")?;
                Ok(true)
            }
        }
    }

    async fn delete_book(&self, user_id: Uuid, book_id: i32) -> Result<bool, UsersServiceError> {
        let result = taste_books::Entity::delete_many()
            .filter(taste_books::Column::UserId.eq(user_id))
            .filter(taste_books::Column::BookId.eq(book_id))
            .exec(&self.db)
            .await
            .context("delete taste book")?;
        Ok(result.rows_affected > 0)
    }

    async fn delete_book_tag(
        &self,
        user_id: Uuid,
        tag_kind: &str,
        tag_name: &str,
    ) -> Result<bool, UsersServiceError> {
        let result = taste_book_tags::Entity::delete_many()
            .filter(taste_book_tags::Column::UserId.eq(user_id))
            .filter(taste_book_tags::Column::TagKind.eq(tag_kind))
            .filter(taste_book_tags::Column::TagName.eq(tag_name))
            .exec(&self.db)
            .await
            .context("delete taste book tag")?;
        Ok(result.rows_affected > 0)
    }
}

fn taste_book_from_model(model: taste_books::Model) -> TasteBook {
    TasteBook {
        user_id: model.user_id,
        book_id: model.book_id,
        is_dislike: model.is_dislike,
        created_at: model.created_at,
    }
}

fn taste_book_tag_from_model(model: taste_book_tags::Model) -> TasteBookTag {
    TasteBookTag {
        user_id: model.user_id,
        tag_kind: model.tag_kind,
        tag_name: model.tag_name,
        is_dislike: model.is_dislike,
        created_at: model.created_at,
    }
}

// ── History repository ───────────────────────────────────────────────────────

#[derive(Clone)]
pub struct DbHistoryRepository {
    pub db: DatabaseConnection,
}

impl HistoryRepository for DbHistoryRepository {
    async fn list(
        &self,
        user_id: Uuid,
        sort_by: HistorySortBy,
        page: PageRequest,
    ) -> Result<Vec<HistoryBook>, UsersServiceError> {
        let PageRequest { per_page, page } = page.clamped();
        let mut query =
            history_books::Entity::find().filter(history_books::Column::UserId.eq(user_id));
        query = match sort_by {
            HistorySortBy::CreatedAt(Sort::Desc) => {
                query.order_by_desc(history_books::Column::CreatedAt)
            }
            HistorySortBy::CreatedAt(Sort::Asc) => {
                query.order_by_asc(history_books::Column::CreatedAt)
            }
            HistorySortBy::UpdatedAt(Sort::Desc) => {
                query.order_by_desc(history_books::Column::UpdatedAt)
            }
            HistorySortBy::UpdatedAt(Sort::Asc) => {
                query.order_by_asc(history_books::Column::UpdatedAt)
            }
            HistorySortBy::Random => query.order_by_random(),
        };
        let models = query
            .offset(((page - 1) * per_page) as u64)
            .limit(per_page as u64)
            .all(&self.db)
            .await
            .context("list history books")?;
        Ok(models.into_iter().map(history_book_from_model).collect())
    }

    async fn get(
        &self,
        user_id: Uuid,
        book_id: i32,
    ) -> Result<Option<HistoryBook>, UsersServiceError> {
        let model = history_books::Entity::find_by_id((user_id, book_id))
            .one(&self.db)
            .await
            .context("get history book")?;
        Ok(model.map(history_book_from_model))
    }

    async fn upsert(&self, history: &HistoryBook) -> Result<(), UsersServiceError> {
        let history_book = history_books::ActiveModel {
            user_id: Set(history.user_id),
            book_id: Set(history.book_id),
            page: Set(history.page),
            created_at: Set(history.created_at),
            updated_at: Set(history.updated_at),
        };
        history_books::Entity::insert(history_book)
            .on_conflict(
                OnConflict::columns([history_books::Column::UserId, history_books::Column::BookId])
                    .update_columns([
                        history_books::Column::Page,
                        history_books::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec_without_returning(&self.db)
            .await
            .context("upsert history book")?;
        Ok(())
    }

    async fn delete(&self, user_id: Uuid, book_id: i32) -> Result<bool, UsersServiceError> {
        let result = history_books::Entity::delete_many()
            .filter(history_books::Column::UserId.eq(user_id))
            .filter(history_books::Column::BookId.eq(book_id))
            .exec(&self.db)
            .await
            .context("delete history book")?;
        Ok(result.rows_affected > 0)
    }
}

fn history_book_from_model(model: history_books::Model) -> HistoryBook {
    HistoryBook {
        user_id: model.user_id,
        book_id: model.book_id,
        page: model.page,
        created_at: model.created_at,
        updated_at: model.updated_at,
    }
}

// ── Notification repository ──────────────────────────────────────────────────

#[derive(Clone)]
pub struct DbNotificationRepository {
    pub db: DatabaseConnection,
}

impl NotificationRepository for DbNotificationRepository {
    async fn list(
        &self,
        user_id: Uuid,
        sort_by: NotificationSortBy,
        page: PageRequest,
    ) -> Result<Vec<NotificationBook>, UsersServiceError> {
        let PageRequest { per_page, page } = page.clamped();
        let mut query = notification_books::Entity::find()
            .filter(notification_books::Column::UserId.eq(user_id));
        query = match sort_by {
            NotificationSortBy::CreatedAt(Sort::Desc) => {
                query.order_by_desc(notification_books::Column::CreatedAt)
            }
            NotificationSortBy::CreatedAt(Sort::Asc) => {
                query.order_by_asc(notification_books::Column::CreatedAt)
            }
        };
        let models = query
            .offset(((page - 1) * per_page) as u64)
            .limit(per_page as u64)
            .all(&self.db)
            .await
            .context("list notification books")?;

        let mut results = Vec::with_capacity(models.len());
        for model in models {
            let tags = notification_book_tags::Entity::find()
                .filter(notification_book_tags::Column::NotificationBookId.eq(model.id))
                .all(&self.db)
                .await
                .context("list notification book tags")?;
            let book_tags = tags
                .into_iter()
                .map(|tag| (tag.tag_kind, tag.tag_name))
                .collect();
            results.push(NotificationBook {
                id: model.id,
                user_id: model.user_id,
                book_id: model.book_id,
                book_tags,
                created_at: model.created_at,
            });
        }
        Ok(results)
    }

    async fn create(&self, notification: &NotificationBook) -> Result<(), UsersServiceError> {
        self.db
            .transaction::<_, (), sea_orm::DbErr>(|txn| {
                let notification = notification.clone();
                Box::pin(async move {
                    notification_books::ActiveModel {
                        id: Set(notification.id),
                        user_id: Set(notification.user_id),
                        book_id: Set(notification.book_id),
                        created_at: Set(notification.created_at),
                    }
                    .insert(txn)
                    .await?;

                    for (tag_kind, tag_name) in &notification.book_tags {
                        notification_book_tags::ActiveModel {
                            id: Set(Uuid::now_v7()),
                            notification_book_id: Set(notification.id),
                            tag_kind: Set(tag_kind.clone()),
                            tag_name: Set(tag_name.clone()),
                        }
                        .insert(txn)
                        .await?;
                    }
                    Ok(())
                })
            })
            .await
            .context("create notification book")?;
        Ok(())
    }
}

// ── RenewBook port ───────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct DbRenewBookPort {
    pub db: DatabaseConnection,
}

impl RenewBookPort for DbRenewBookPort {
    async fn renew_book_id(&self, old_id: i32, new_id: i32) -> Result<(), UsersServiceError> {
        self.db
            .transaction::<_, (), sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    use sea_orm::sea_query::Expr;

                    // taste_books: delete old rows where new_id already exists for same user
                    let _ = taste_books::Entity::delete_many()
                        .filter(taste_books::Column::BookId.eq(old_id))
                        .filter(
                            taste_books::Column::UserId.in_subquery(
                                sea_orm::sea_query::Query::select()
                                    .column(taste_books::Column::UserId)
                                    .from(taste_books::Entity)
                                    .and_where(Expr::col(taste_books::Column::BookId).eq(new_id))
                                    .to_owned(),
                            ),
                        )
                        .exec(txn)
                        .await?;
                    let _ = taste_books::Entity::update_many()
                        .filter(taste_books::Column::BookId.eq(old_id))
                        .col_expr(taste_books::Column::BookId, Expr::value(new_id))
                        .exec(txn)
                        .await?;

                    // history_books: same pattern
                    let _ = history_books::Entity::delete_many()
                        .filter(history_books::Column::BookId.eq(old_id))
                        .filter(
                            history_books::Column::UserId.in_subquery(
                                sea_orm::sea_query::Query::select()
                                    .column(history_books::Column::UserId)
                                    .from(history_books::Entity)
                                    .and_where(Expr::col(history_books::Column::BookId).eq(new_id))
                                    .to_owned(),
                            ),
                        )
                        .exec(txn)
                        .await?;
                    let _ = history_books::Entity::update_many()
                        .filter(history_books::Column::BookId.eq(old_id))
                        .col_expr(history_books::Column::BookId, Expr::value(new_id))
                        .exec(txn)
                        .await?;

                    // notification_books: no composite PK conflict, just rename
                    let _ = notification_books::Entity::update_many()
                        .filter(notification_books::Column::BookId.eq(old_id))
                        .col_expr(notification_books::Column::BookId, Expr::value(new_id))
                        .exec(txn)
                        .await?;

                    Ok(())
                })
            })
            .await
            .context("renew book id")?;
        Ok(())
    }
}

// ── FCM token repository ─────────────────────────────────────────────────────

#[derive(Clone)]
pub struct DbFcmTokenRepository {
    pub db: DatabaseConnection,
}

impl FcmTokenRepository for DbFcmTokenRepository {
    async fn upsert(&self, token: &FcmToken, user_id: Uuid) -> Result<(), UsersServiceError> {
        let existing = fcm_tokens::Entity::find_by_id(token.id)
            .one(&self.db)
            .await
            .context("find fcm token for upsert")?;

        match existing {
            Some(row) if row.user_id == user_id => {
                let mut fcm_token = row.into_active_model();
                fcm_token.token = Set(token.token.clone());
                fcm_token.updated_at = Set(Utc::now());
                fcm_token
                    .update(&self.db)
                    .await
                    .context("update fcm token")?;
            }
            Some(_) => {
                // user_id mismatch — ignore silently (guard)
            }
            None => {
                fcm_tokens::ActiveModel {
                    id: Set(token.id),
                    user_id: Set(user_id),
                    token: Set(token.token.clone()),
                    updated_at: Set(token.updated_at),
                }
                .insert(&self.db)
                .await
                .context("insert fcm token")?;
            }
        }
        Ok(())
    }

    async fn find_fresh_by_user_ids(
        &self,
        user_ids: &[Uuid],
    ) -> Result<Vec<FcmToken>, UsersServiceError> {
        let cutoff = Utc::now() - Duration::days(30);
        let models = fcm_tokens::Entity::find()
            .filter(fcm_tokens::Column::UserId.is_in(user_ids.iter().copied()))
            .filter(fcm_tokens::Column::UpdatedAt.gt(cutoff))
            .all(&self.db)
            .await
            .context("find fresh fcm tokens")?;
        Ok(models.into_iter().map(fcm_token_from_model).collect())
    }
}

fn fcm_token_from_model(model: fcm_tokens::Model) -> FcmToken {
    FcmToken {
        id: model.id,
        user_id: model.user_id,
        token: model.token,
        updated_at: model.updated_at,
    }
}
