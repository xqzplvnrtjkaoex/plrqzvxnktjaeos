use madome_auth::error::AuthServiceError;
use madome_auth::usecase::token::{
    CreateTokenInput, CreateTokenUseCase, RefreshTokenUseCase, issue_access_token,
    issue_refresh_token, validate_token,
};

use crate::helpers::{MockAuthCodeRepo, MockUserRepo, TEST_JWT_SECRET, test_auth_code, test_user};

// ── issue_access_token / validate_token ──────────────────────────────────────

#[tokio::test]
async fn should_issue_access_token_that_validates_successfully() {
    let user = test_user();
    let (token, exp) = issue_access_token(&user, TEST_JWT_SECRET).unwrap();

    assert!(!token.is_empty());
    assert!(exp > 0);

    let claims = validate_token(&token, TEST_JWT_SECRET).unwrap();
    assert_eq!(claims.sub, user.id.to_string());
    assert_eq!(claims.role, user.role);
    assert_eq!(claims.exp, exp);
}

#[tokio::test]
async fn should_reject_token_signed_with_wrong_secret() {
    let user = test_user();
    let (token, _) = issue_access_token(&user, TEST_JWT_SECRET).unwrap();

    let result = validate_token(&token, "wrong-secret");
    assert!(
        matches!(result, Err(AuthServiceError::InvalidRefreshToken)),
        "expected InvalidRefreshToken, got {result:?}"
    );
}

#[tokio::test]
async fn should_reject_invalid_token_string() {
    let result = validate_token("not-a-jwt", TEST_JWT_SECRET);
    assert!(
        matches!(result, Err(AuthServiceError::InvalidRefreshToken)),
        "expected InvalidRefreshToken, got {result:?}"
    );
}

#[tokio::test]
async fn should_issue_refresh_token_that_validates_successfully() {
    let user = test_user();
    let token = issue_refresh_token(&user, TEST_JWT_SECRET).unwrap();

    assert!(!token.is_empty());

    let claims = validate_token(&token, TEST_JWT_SECRET).unwrap();
    assert_eq!(claims.sub, user.id.to_string());
    assert_eq!(claims.role, user.role);
}

// ── CreateTokenUseCase ───────────────────────────────────────────────────────

#[tokio::test]
async fn should_create_token_pair_with_valid_auth_code() {
    let user = test_user();
    let code = test_auth_code(user.id);
    let code_str = code.code.clone();

    let usecase = CreateTokenUseCase {
        users: MockUserRepo::new(vec![user.clone()]),
        auth_codes: MockAuthCodeRepo::new(vec![code], 1),
        jwt_secret: TEST_JWT_SECRET.to_owned(),
    };

    let output = usecase
        .execute(CreateTokenInput {
            email: user.email.clone(),
            code: code_str,
        })
        .await
        .unwrap();

    assert_eq!(output.user.id, user.id);
    assert!(!output.access_token.is_empty());
    assert!(!output.refresh_token.is_empty());
    assert!(output.access_token_exp > 0);

    // Verify tokens are valid JWTs.
    let access_claims = validate_token(&output.access_token, TEST_JWT_SECRET).unwrap();
    assert_eq!(access_claims.sub, user.id.to_string());

    let refresh_claims = validate_token(&output.refresh_token, TEST_JWT_SECRET).unwrap();
    assert_eq!(refresh_claims.sub, user.id.to_string());
}

#[tokio::test]
async fn should_mark_auth_code_as_used_after_create_token() {
    let user = test_user();
    let code = test_auth_code(user.id);
    let code_str = code.code.clone();
    let code_id = code.id;

    let mock_repo = MockAuthCodeRepo::new(vec![code], 1);
    let codes_handle = mock_repo.codes_handle();

    let usecase = CreateTokenUseCase {
        users: MockUserRepo::new(vec![user.clone()]),
        auth_codes: mock_repo,
        jwt_secret: TEST_JWT_SECRET.to_owned(),
    };

    usecase
        .execute(CreateTokenInput {
            email: user.email.clone(),
            code: code_str,
        })
        .await
        .unwrap();

    // Verify the code was marked as used via the shared handle.
    let codes = codes_handle.lock().unwrap();
    let used_code = codes.iter().find(|c| c.id == code_id).unwrap();
    assert!(
        used_code.used_at.is_some(),
        "auth code should be marked as used after token creation"
    );
}

#[tokio::test]
async fn should_return_not_found_when_user_unknown_for_create_token() {
    let usecase = CreateTokenUseCase {
        users: MockUserRepo::empty(),
        auth_codes: MockAuthCodeRepo::empty(),
        jwt_secret: TEST_JWT_SECRET.to_owned(),
    };

    let result = usecase
        .execute(CreateTokenInput {
            email: "nobody@example.com".to_owned(),
            code: "ABCDEF123456".to_owned(),
        })
        .await;

    assert!(
        matches!(result, Err(AuthServiceError::UserNotFound)),
        "expected UserNotFound, got {result:?}"
    );
}

#[tokio::test]
async fn should_return_not_found_when_auth_code_invalid_for_create_token() {
    let user = test_user();

    let usecase = CreateTokenUseCase {
        users: MockUserRepo::new(vec![user.clone()]),
        auth_codes: MockAuthCodeRepo::empty(), // no codes at all
        jwt_secret: TEST_JWT_SECRET.to_owned(),
    };

    let result = usecase
        .execute(CreateTokenInput {
            email: user.email.clone(),
            code: "WRONGCODE123".to_owned(),
        })
        .await;

    assert!(
        matches!(result, Err(AuthServiceError::InvalidAuthcode)),
        "expected InvalidAuthcode, got {result:?}"
    );
}

// ── RefreshTokenUseCase ──────────────────────────────────────────────────────

#[tokio::test]
async fn should_refresh_token_pair_with_valid_refresh_jwt() {
    let user = test_user();
    let refresh = issue_refresh_token(&user, TEST_JWT_SECRET).unwrap();

    let usecase = RefreshTokenUseCase {
        users: MockUserRepo::new(vec![user.clone()]),
        jwt_secret: TEST_JWT_SECRET.to_owned(),
    };

    let output = usecase.execute(&refresh).await.unwrap();

    assert_eq!(output.user_id, user.id);
    assert_eq!(output.user_role, user.role);
    assert!(!output.access_token.is_empty());
    assert!(!output.refresh_token.is_empty());
    assert!(output.access_token_exp > 0);

    // New tokens should be valid.
    let claims = validate_token(&output.access_token, TEST_JWT_SECRET).unwrap();
    assert_eq!(claims.sub, user.id.to_string());
}

#[tokio::test]
async fn should_return_unauthorized_when_refresh_jwt_invalid() {
    let user = test_user();

    let usecase = RefreshTokenUseCase {
        users: MockUserRepo::new(vec![user]),
        jwt_secret: TEST_JWT_SECRET.to_owned(),
    };

    let result = usecase.execute("not-a-valid-jwt").await;

    assert!(
        matches!(result, Err(AuthServiceError::InvalidRefreshToken)),
        "expected InvalidRefreshToken, got {result:?}"
    );
}

#[tokio::test]
async fn should_return_unauthorized_when_refresh_jwt_signed_with_wrong_secret() {
    let user = test_user();
    let refresh = issue_refresh_token(&user, "other-secret").unwrap();

    let usecase = RefreshTokenUseCase {
        users: MockUserRepo::new(vec![user]),
        jwt_secret: TEST_JWT_SECRET.to_owned(),
    };

    let result = usecase.execute(&refresh).await;

    assert!(
        matches!(result, Err(AuthServiceError::InvalidRefreshToken)),
        "expected InvalidRefreshToken, got {result:?}"
    );
}

#[tokio::test]
async fn should_return_unauthorized_when_user_deleted_during_refresh() {
    let user = test_user();
    let refresh = issue_refresh_token(&user, TEST_JWT_SECRET).unwrap();

    let usecase = RefreshTokenUseCase {
        users: MockUserRepo::empty(), // user no longer exists
        jwt_secret: TEST_JWT_SECRET.to_owned(),
    };

    let result = usecase.execute(&refresh).await;

    assert!(
        matches!(result, Err(AuthServiceError::InvalidRefreshToken)),
        "expected InvalidRefreshToken, got {result:?}"
    );
}
