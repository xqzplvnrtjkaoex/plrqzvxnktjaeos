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
            Ok(resp) => Ok(Some(resp.into_inner().try_into()?)),
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
            Ok(resp) => Ok(Some(resp.into_inner().try_into()?)),
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!("gRPC get_user failed: {e}").into()),
        }
    }
}

impl TryFrom<madome_proto::user::User> for AuthUser {
    type Error = AuthServiceError;

    fn try_from(user: madome_proto::user::User) -> Result<Self, Self::Error> {
        let id = user
            .id
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid UUID from users service"))?;
        let role = u8::try_from(user.role)
            .map_err(|_| anyhow::anyhow!("role out of u8 range: {}", user.role))?;
        Ok(AuthUser {
            id,
            email: user.email,
            role,
        })
    }
}
