use chrono::Utc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use madome_domain::pagination::PageRequest;
use madome_proto::notification::{
    Empty as NotifEmpty, notification_service_server::NotificationService,
};
use madome_proto::user::{
    BookTagTaste, BookTaste, Empty, GetTastesRequest, GetUserRequest, RenewBookRequest, Taste,
    TasteList, User, taste::Kind, user_service_server::UserService,
};

use crate::domain::types::{self as domain, NotificationBook};
use crate::state::AppState;
use crate::usecase::notification::CreateNotificationUseCase;
use crate::usecase::renew_book::RenewBookUseCase;
use crate::usecase::taste::GetTastesUseCase;
use crate::usecase::user::GetUserUseCase;

#[derive(Clone)]
pub struct UsersGrpcServer {
    pub state: AppState,
}

#[tonic::async_trait]
impl UserService for UsersGrpcServer {
    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<User>, Status> {
        let user_id = request
            .into_inner()
            .user_id
            .parse::<Uuid>()
            .map_err(|_| Status::invalid_argument("invalid user_id"))?;

        let uc = GetUserUseCase {
            repo: self.state.user_repo(),
        };
        let user = uc
            .execute(user_id)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        Ok(Response::new(User {
            id: user.id.to_string(),
            name: user.name,
            email: user.email,
            handle: user.handle,
            role: user.role as u32,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        }))
    }

    async fn get_tastes(
        &self,
        request: Request<GetTastesRequest>,
    ) -> Result<Response<TasteList>, Status> {
        let req = request.into_inner();
        let user_id = req
            .user_id
            .parse::<Uuid>()
            .map_err(|_| Status::invalid_argument("invalid user_id"))?;

        let is_dislike = if req.dislikes_only { Some(true) } else { None };
        let page = PageRequest {
            per_page: 100,
            page: 1,
        };

        let uc = GetTastesUseCase {
            repo: self.state.taste_repo(),
        };
        let domain_tastes = uc
            .execute(user_id, Default::default(), is_dislike, page)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let tastes: Vec<Taste> = domain_tastes
            .into_iter()
            .map(|t| match t {
                domain::Taste::Book(b) => Taste {
                    kind: Some(Kind::Book(BookTaste {
                        book_id: b.book_id as u32,
                        is_dislike: b.is_dislike,
                    })),
                },
                domain::Taste::BookTag(t) => Taste {
                    kind: Some(Kind::BookTag(BookTagTaste {
                        tag_kind: t.tag_kind,
                        tag_name: t.tag_name,
                        is_dislike: t.is_dislike,
                    })),
                },
            })
            .collect();

        Ok(Response::new(TasteList { tastes }))
    }

    async fn renew_book(
        &self,
        request: Request<RenewBookRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        let uc = RenewBookUseCase {
            port: self.state.renew_book_port(),
        };
        uc.execute(req.old_book_id as i32, req.new_book_id as i32)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(Empty {}))
    }
}

#[tonic::async_trait]
impl NotificationService for UsersGrpcServer {
    async fn create_notification(
        &self,
        request: Request<madome_proto::notification::CreateNotificationRequest>,
    ) -> Result<Response<NotifEmpty>, Status> {
        let req = request.into_inner();
        let user_id = req
            .user_id
            .parse::<Uuid>()
            .map_err(|_| Status::invalid_argument("invalid user_id"))?;

        let book_tags = req
            .book_tags
            .into_iter()
            .map(|t| (t.kind, t.name))
            .collect();

        let notification = NotificationBook {
            id: Uuid::now_v7(),
            user_id,
            book_id: req.book_id as i32,
            book_tags,
            created_at: Utc::now(),
        };

        let uc = CreateNotificationUseCase {
            repo: self.state.notification_repo(),
        };
        uc.execute(notification)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(NotifEmpty {}))
    }
}
