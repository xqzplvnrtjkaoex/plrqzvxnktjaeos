use tonic::transport::Channel;
use uuid::Uuid;

use madome_proto::user::{
    GetUserByEmailRequest, GetUserRequest, user_service_client::UserServiceClient,
};

use crate::domain::repository::UserPort;
use crate::domain::types::AuthUser;
use crate::error::AuthServiceError;

#[derive(Clone)]
pub struct GrpcUserPort {
    client: UserServiceClient<Channel>,
}

impl GrpcUserPort {
    pub fn new(channel: Channel) -> Self {
        Self {
            client: UserServiceClient::new(channel),
        }
    }
}

impl UserPort for GrpcUserPort {
    async fn find_by_email(&self, email: &str) -> Result<Option<AuthUser>, AuthServiceError> {
        let response = self
            .client
            .clone()
            .get_user_by_email(GetUserByEmailRequest {
                email: email.to_string(),
            })
            .await;
        match response {
            Ok(resp) => Ok(Some(proto_to_auth_user(resp.into_inner()))),
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!("gRPC get_user_by_email failed: {e}").into()),
        }
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuthUser>, AuthServiceError> {
        let response = self
            .client
            .clone()
            .get_user(GetUserRequest {
                user_id: id.to_string(),
            })
            .await;
        match response {
            Ok(resp) => Ok(Some(proto_to_auth_user(resp.into_inner()))),
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!("gRPC get_user failed: {e}").into()),
        }
    }
}

fn proto_to_auth_user(user: madome_proto::user::User) -> AuthUser {
    AuthUser {
        id: user.id.parse().expect("invalid UUID from users service"),
        email: user.email,
        role: user.role as u8,
    }
}
