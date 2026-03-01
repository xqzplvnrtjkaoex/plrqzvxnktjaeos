use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;
use webauthn_rs::prelude::*;

use crate::domain::repository::{PasskeyCache, PasskeyRepository, UserRepository};
use crate::domain::types::PasskeyRecord;
use crate::error::AuthServiceError;
use crate::usecase::token::{CreateTokenOutput, issue_access_token, issue_refresh_token};

// ── List passkeys ─────────────────────────────────────────────────────────────

pub struct ListPasskeysUseCase<P>
where
    P: PasskeyRepository,
{
    pub passkeys: P,
}

pub struct PasskeyInfo {
    pub credential_id: Vec<u8>,
    pub created_at: chrono::DateTime<Utc>,
}

impl<P> ListPasskeysUseCase<P>
where
    P: PasskeyRepository,
{
    pub async fn execute(&self, user_id: Uuid) -> Result<Vec<PasskeyInfo>, AuthServiceError> {
        let records = self.passkeys.list_by_user(user_id).await?;
        Ok(records
            .into_iter()
            .map(|r| PasskeyInfo {
                credential_id: r.credential_id,
                created_at: r.created_at,
            })
            .collect())
    }
}

// ── Delete passkey ────────────────────────────────────────────────────────────

pub struct DeletePasskeyUseCase<P>
where
    P: PasskeyRepository,
{
    pub passkeys: P,
}

impl<P> DeletePasskeyUseCase<P>
where
    P: PasskeyRepository,
{
    /// Returns 404 if not found or belongs to a different user.
    pub async fn execute(
        &self,
        credential_id: &[u8],
        user_id: Uuid,
    ) -> Result<(), AuthServiceError> {
        let deleted = self.passkeys.delete(credential_id, user_id).await?;
        if !deleted {
            return Err(AuthServiceError::CredentialNotFound);
        }
        Ok(())
    }
}

// ── Start registration ────────────────────────────────────────────────────────

pub struct StartRegistrationOutput {
    pub registration_id: String,
    pub challenge: CreationChallengeResponse,
}

pub struct StartRegistrationUseCase<U, P, C>
where
    U: UserRepository,
    P: PasskeyRepository,
    C: PasskeyCache,
{
    pub users: U,
    pub passkeys: P,
    pub cache: C,
    pub webauthn: Arc<Webauthn>,
}

impl<U, P, C> StartRegistrationUseCase<U, P, C>
where
    U: UserRepository,
    P: PasskeyRepository,
    C: PasskeyCache,
{
    pub async fn execute(
        &self,
        user_id: Uuid,
    ) -> Result<StartRegistrationOutput, AuthServiceError> {
        let user = self
            .users
            .find_by_id(user_id)
            .await?
            .ok_or(AuthServiceError::UserNotFound)?;

        // Build exclude list from existing passkeys.
        let existing = self.passkeys.list_by_user(user_id).await?;
        let exclude: Option<Vec<CredentialID>> = if existing.is_empty() {
            None
        } else {
            Some(
                existing
                    .iter()
                    .map(|r| CredentialID::from(r.credential_id.clone()))
                    .collect(),
            )
        };

        let (ccr, reg_state) = self
            .webauthn
            .start_passkey_registration(user_id, &user.email, &user.email, exclude)
            .map_err(|e| AuthServiceError::Internal(anyhow::anyhow!("{e}")))?;

        let reg_id = Uuid::new_v4().to_string();
        let state_json =
            serde_json::to_vec(&reg_state).map_err(|e| AuthServiceError::Internal(e.into()))?;
        self.cache
            .set_registration_state(user_id, &reg_id, &state_json)
            .await?;

        Ok(StartRegistrationOutput {
            registration_id: reg_id,
            challenge: ccr,
        })
    }
}

// ── Finish registration ───────────────────────────────────────────────────────

pub struct FinishRegistrationUseCase<P, C>
where
    P: PasskeyRepository,
    C: PasskeyCache,
{
    pub passkeys: P,
    pub cache: C,
    pub webauthn: Arc<Webauthn>,
}

impl<P, C> FinishRegistrationUseCase<P, C>
where
    P: PasskeyRepository,
    C: PasskeyCache,
{
    pub async fn execute(
        &self,
        user_id: Uuid,
        registration_id: &str,
        credential: RegisterPublicKeyCredential,
    ) -> Result<(), AuthServiceError> {
        let state_json = self
            .cache
            .take_registration_state(user_id, registration_id)
            .await?
            .ok_or(AuthServiceError::InvalidSession)?;

        let reg_state: PasskeyRegistration =
            serde_json::from_slice(&state_json).map_err(|_| AuthServiceError::InvalidSession)?;

        let passkey = self
            .webauthn
            .finish_passkey_registration(&credential, &reg_state)
            .map_err(|_| AuthServiceError::InvalidCredential)?;

        let cred_id = passkey.cred_id().to_vec();
        let aaguid = parse_aaguid_from_credential(&credential).unwrap_or(Uuid::nil());
        let credential_bytes =
            serde_json::to_vec(&passkey).map_err(|e| AuthServiceError::Internal(e.into()))?;

        let record = PasskeyRecord {
            credential_id: cred_id,
            user_id,
            aaguid,
            credential: credential_bytes,
            created_at: Utc::now(),
        };
        self.passkeys.create(&record).await?;
        Ok(())
    }
}

// ── Start authentication ──────────────────────────────────────────────────────

pub struct StartAuthenticationOutput {
    pub authentication_id: String,
    pub challenge: RequestChallengeResponse,
}

pub struct StartAuthenticationUseCase<U, P, C>
where
    U: UserRepository,
    P: PasskeyRepository,
    C: PasskeyCache,
{
    pub users: U,
    pub passkeys: P,
    pub cache: C,
    pub webauthn: Arc<Webauthn>,
}

impl<U, P, C> StartAuthenticationUseCase<U, P, C>
where
    U: UserRepository,
    P: PasskeyRepository,
    C: PasskeyCache,
{
    pub async fn execute(
        &self,
        email: &str,
    ) -> Result<StartAuthenticationOutput, AuthServiceError> {
        let user = self
            .users
            .find_by_email(email)
            .await?
            .ok_or(AuthServiceError::UserNotFound)?;

        let stored = self.passkeys.list_by_user(user.id).await?;
        if stored.is_empty() {
            return Err(AuthServiceError::CredentialNotFound);
        }
        let passkey_list: Vec<Passkey> = stored
            .iter()
            .filter_map(|r| serde_json::from_slice(&r.credential).ok())
            .collect();

        let (rcr, auth_state) = self
            .webauthn
            .start_passkey_authentication(&passkey_list)
            .map_err(|e| AuthServiceError::Internal(anyhow::anyhow!("{e}")))?;

        let auth_id = Uuid::new_v4().to_string();
        let state_json =
            serde_json::to_vec(&auth_state).map_err(|e| AuthServiceError::Internal(e.into()))?;
        self.cache
            .set_authentication_state(email, &auth_id, &state_json)
            .await?;

        Ok(StartAuthenticationOutput {
            authentication_id: auth_id,
            challenge: rcr,
        })
    }
}

// ── Finish authentication ─────────────────────────────────────────────────────

pub struct FinishAuthenticationUseCase<U, P, C>
where
    U: UserRepository,
    P: PasskeyRepository,
    C: PasskeyCache,
{
    pub users: U,
    pub passkeys: P,
    pub cache: C,
    pub webauthn: Arc<Webauthn>,
    pub jwt_secret: String,
}

impl<U, P, C> FinishAuthenticationUseCase<U, P, C>
where
    U: UserRepository,
    P: PasskeyRepository,
    C: PasskeyCache,
{
    pub async fn execute(
        &self,
        email: &str,
        authentication_id: &str,
        credential: PublicKeyCredential,
    ) -> Result<CreateTokenOutput, AuthServiceError> {
        let user = self
            .users
            .find_by_email(email)
            .await?
            .ok_or(AuthServiceError::UserNotFound)?;

        let state_json = self
            .cache
            .take_authentication_state(email, authentication_id)
            .await?
            .ok_or(AuthServiceError::InvalidSession)?;

        let auth_state: PasskeyAuthentication =
            serde_json::from_slice(&state_json).map_err(|_| AuthServiceError::InvalidSession)?;

        let stored = self.passkeys.list_by_user(user.id).await?;
        let mut passkey_list: Vec<Passkey> = stored
            .iter()
            .filter_map(|r| serde_json::from_slice(&r.credential).ok())
            .collect();

        let auth_result = self
            .webauthn
            .finish_passkey_authentication(&credential, &auth_state)
            .map_err(|_| AuthServiceError::InvalidCredential)?;

        // Persist counter updates for any passkey that changed.
        for (pk, record) in passkey_list.iter_mut().zip(stored.iter()) {
            if pk.update_credential(&auth_result) == Some(true) {
                let updated_bytes =
                    serde_json::to_vec(&pk).map_err(|e| AuthServiceError::Internal(e.into()))?;
                self.passkeys
                    .update_credential(&record.credential_id, &updated_bytes)
                    .await?;
            }
        }

        let (access_token, access_token_exp) = issue_access_token(&user, &self.jwt_secret)?;
        let refresh_token = issue_refresh_token(&user, &self.jwt_secret)?;

        Ok(CreateTokenOutput {
            user,
            access_token,
            access_token_exp,
            refresh_token,
        })
    }
}

// ── AAGUID extraction ─────────────────────────────────────────────────────────

/// Extract the AAGUID from a `RegisterPublicKeyCredential` by parsing its
/// raw attestation object (CBOR). Per the WebAuthn spec the AAGUID occupies
/// bytes 37..53 of the `authData` field inside the attestation object.
///
/// Mirrors the legacy `parse_aaguid` implementation in
/// `previous/auth-madome-app`.
fn parse_aaguid_from_credential(credential: &RegisterPublicKeyCredential) -> Option<Uuid> {
    // https://www.rfc-editor.org/rfc/rfc8949.html#section-3.2.2
    let mut decoder = minicbor::Decoder::new(&credential.response.attestation_object);
    decoder.map().ok()?;

    // fmt
    decoder.skip().ok()?;
    decoder.skip().ok()?;

    // attStmt
    decoder.skip().ok()?;
    decoder.skip().ok()?;

    let key = decoder.str().ok()?;
    if key != "authData" {
        return None;
    }
    let auth_data = decoder.bytes().ok()?;
    if auth_data.len() < 53 {
        return None;
    }

    let mut aaguid = [0u8; 16];
    aaguid.copy_from_slice(&auth_data[37..53]);
    Some(Uuid::from_bytes(aaguid))
}
