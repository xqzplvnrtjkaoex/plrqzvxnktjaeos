use tower_http::request_id::{MakeRequestId, RequestId, SetRequestIdLayer};
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct MakeUuidRequestId;

impl MakeRequestId for MakeUuidRequestId {
    fn make_request_id<B>(&mut self, _request: &axum::http::Request<B>) -> Option<RequestId> {
        let id = Uuid::new_v4().to_string();
        Some(RequestId::new(id.parse().unwrap()))
    }
}

/// Build the request-id layer. Apply with `.layer(request_id_layer())` in router.
pub fn request_id_layer() -> SetRequestIdLayer<MakeUuidRequestId> {
    SetRequestIdLayer::new(
        axum::http::HeaderName::from_static("x-request-id"),
        MakeUuidRequestId,
    )
}
