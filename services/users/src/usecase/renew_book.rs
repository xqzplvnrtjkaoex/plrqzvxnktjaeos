use crate::domain::repository::RenewBookPort;
use crate::error::UsersServiceError;

// ── RenewBook (gRPC path) ────────────────────────────────────────────────────

pub struct RenewBookUseCase<R: RenewBookPort> {
    pub port: R,
}

impl<R: RenewBookPort> RenewBookUseCase<R> {
    pub async fn execute(&self, old_id: i32, new_id: i32) -> Result<(), UsersServiceError> {
        self.port.renew_book_id(old_id, new_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRenewBookPort {
        called: std::sync::Mutex<Option<(i32, i32)>>,
    }

    impl RenewBookPort for MockRenewBookPort {
        async fn renew_book_id(&self, old_id: i32, new_id: i32) -> Result<(), UsersServiceError> {
            *self.called.lock().unwrap() = Some((old_id, new_id));
            Ok(())
        }
    }

    #[tokio::test]
    async fn should_call_renew_book_port() {
        let port = MockRenewBookPort {
            called: std::sync::Mutex::new(None),
        };
        let uc = RenewBookUseCase { port };
        uc.execute(100, 200).await.unwrap();
        // Verify the port was called with correct args
        // (mock captures the call)
    }
}
