use anyhow::Context as _;
use tonic::transport::Channel;

use madome_proto::library::{
    GetBookRequest, HasBookTagRequest, library_service_client::LibraryServiceClient,
};

use crate::domain::repository::LibraryQueryPort;
use crate::error::UsersServiceError;

/// gRPC client implementing `LibraryQueryPort` via `library.LibraryService`.
#[derive(Clone)]
pub struct GrpcLibraryClient {
    client: LibraryServiceClient<Channel>,
}

impl GrpcLibraryClient {
    pub async fn connect(url: &str) -> Result<Self, UsersServiceError> {
        let client = LibraryServiceClient::connect(url.to_owned())
            .await
            .context("connect to library gRPC")?;
        Ok(Self { client })
    }

    /// Create a client with lazy connection (connects on first RPC call).
    /// Useful for tests where the library service may not be running.
    pub fn lazy(url: &str) -> Self {
        let channel = Channel::from_shared(url.to_owned())
            .expect("valid URI")
            .connect_lazy();
        Self {
            client: LibraryServiceClient::new(channel),
        }
    }
}

impl LibraryQueryPort for GrpcLibraryClient {
    async fn has_book(&self, book_id: i32) -> Result<bool, UsersServiceError> {
        let resp = self
            .client
            .clone()
            .get_book(GetBookRequest {
                book_id: book_id as u32,
            })
            .await
            .context("gRPC GetBook")?;
        Ok(resp.into_inner().found)
    }

    async fn has_book_tag(
        &self,
        tag_kind: &str,
        tag_name: &str,
    ) -> Result<bool, UsersServiceError> {
        let resp = self
            .client
            .clone()
            .has_book_tag(HasBookTagRequest {
                tag_kind: tag_kind.to_owned(),
                tag_name: tag_name.to_owned(),
            })
            .await
            .context("gRPC HasBookTag")?;
        Ok(resp.into_inner().exists)
    }
}
