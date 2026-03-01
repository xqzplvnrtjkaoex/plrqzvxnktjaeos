use uuid::Uuid;

use madome_auth::error::AuthServiceError;
use madome_auth::usecase::passkey::{DeletePasskeyUseCase, ListPasskeysUseCase};

use crate::helpers::{MockPasskeyRepo, test_passkey_record, test_user};

// ── ListPasskeysUseCase ──────────────────────────────────────────────────────

#[tokio::test]
async fn should_return_empty_list_for_user_with_no_passkeys() {
    let user = test_user();

    let usecase = ListPasskeysUseCase {
        passkeys: MockPasskeyRepo::empty(),
    };

    let result = usecase.execute(user.id).await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn should_return_passkey_records_for_user() {
    let user = test_user();
    let record = test_passkey_record(user.id);
    let expected_cred_id = record.credential_id.clone();

    let usecase = ListPasskeysUseCase {
        passkeys: MockPasskeyRepo::new(vec![record]),
    };

    let result = usecase.execute(user.id).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].credential_id, expected_cred_id);
}

#[tokio::test]
async fn should_not_return_passkeys_belonging_to_other_users() {
    let user = test_user();
    let other_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap();
    let record = test_passkey_record(other_user_id);

    let usecase = ListPasskeysUseCase {
        passkeys: MockPasskeyRepo::new(vec![record]),
    };

    let result = usecase.execute(user.id).await.unwrap();
    assert!(
        result.is_empty(),
        "should not return passkeys for other users"
    );
}

// ── DeletePasskeyUseCase ─────────────────────────────────────────────────────

#[tokio::test]
async fn should_delete_passkey_for_existing_credential() {
    let user = test_user();
    let record = test_passkey_record(user.id);
    let cred_id = record.credential_id.clone();

    let usecase = DeletePasskeyUseCase {
        passkeys: MockPasskeyRepo::new(vec![record]),
    };

    let result = usecase.execute(&cred_id, user.id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn should_return_not_found_when_deleting_missing_credential() {
    let user = test_user();

    let usecase = DeletePasskeyUseCase {
        passkeys: MockPasskeyRepo::empty(),
    };

    let result = usecase.execute(&[1, 2, 3], user.id).await;
    assert!(
        matches!(result, Err(AuthServiceError::CredentialNotFound)),
        "expected CredentialNotFound, got {result:?}"
    );
}

#[tokio::test]
async fn should_return_not_found_when_deleting_credential_of_other_user() {
    let user = test_user();
    let other_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap();
    let record = test_passkey_record(other_user_id);
    let cred_id = record.credential_id.clone();

    let usecase = DeletePasskeyUseCase {
        passkeys: MockPasskeyRepo::new(vec![record]),
    };

    // Try to delete another user's credential.
    let result = usecase.execute(&cred_id, user.id).await;
    assert!(
        matches!(result, Err(AuthServiceError::CredentialNotFound)),
        "expected CredentialNotFound when deleting other user's credential, got {result:?}"
    );
}
