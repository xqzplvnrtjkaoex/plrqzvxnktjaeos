use madome_auth::error::AuthServiceError;
use madome_auth::usecase::authcode::{CreateAuthcodeInput, CreateAuthcodeUseCase};

use crate::helpers::{MockAuthCodeRepo, MockUserRepo, test_user};

#[tokio::test]
async fn should_create_authcode_for_known_user() {
    let user = test_user();

    let mock_repo = MockAuthCodeRepo::empty();
    let codes_handle = mock_repo.codes_handle();

    let uc = CreateAuthcodeUseCase {
        users: MockUserRepo::new(vec![user.clone()]),
        auth_codes: mock_repo,
    };

    uc.execute(CreateAuthcodeInput {
        email: user.email.clone(),
    })
    .await
    .unwrap();

    // Verify a code was actually created in the mock.
    let codes = codes_handle.lock().unwrap();
    assert_eq!(
        codes.len(),
        1,
        "expected exactly one auth code to be created"
    );

    let created = &codes[0];
    assert_eq!(created.user_id, user.id);
    assert_eq!(created.code.len(), 12, "auth code should be 12 characters");
    assert!(created.used_at.is_none(), "new code should not be used");
    assert!(
        created.expires_at > chrono::Utc::now(),
        "code should expire in the future"
    );
}

#[tokio::test]
async fn should_return_not_found_when_user_unknown_for_authcode() {
    let uc = CreateAuthcodeUseCase {
        users: MockUserRepo::empty(),
        auth_codes: MockAuthCodeRepo::empty(),
    };

    let result = uc
        .execute(CreateAuthcodeInput {
            email: "nobody@example.com".to_owned(),
        })
        .await;

    assert!(
        matches!(result, Err(AuthServiceError::UserNotFound)),
        "expected UserNotFound, got {result:?}"
    );
}

#[tokio::test]
async fn should_return_too_many_requests_when_active_code_limit_reached() {
    let user = test_user();

    let uc = CreateAuthcodeUseCase {
        users: MockUserRepo::new(vec![user.clone()]),
        auth_codes: MockAuthCodeRepo::new(vec![], 5), // at the limit
    };

    let result = uc
        .execute(CreateAuthcodeInput {
            email: user.email.clone(),
        })
        .await;

    assert!(
        matches!(result, Err(AuthServiceError::TooManyAuthcodes)),
        "expected TooManyAuthcodes, got {result:?}"
    );
}

#[tokio::test]
async fn should_return_too_many_requests_when_active_code_count_exceeds_limit() {
    let user = test_user();

    let uc = CreateAuthcodeUseCase {
        users: MockUserRepo::new(vec![user.clone()]),
        auth_codes: MockAuthCodeRepo::new(vec![], 10), // well over limit
    };

    let result = uc
        .execute(CreateAuthcodeInput {
            email: user.email.clone(),
        })
        .await;

    assert!(
        matches!(result, Err(AuthServiceError::TooManyAuthcodes)),
        "expected TooManyAuthcodes, got {result:?}"
    );
}
