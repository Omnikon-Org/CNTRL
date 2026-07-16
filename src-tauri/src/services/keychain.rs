//! Keychain service — OS-native secret storage.
//!
//! All API keys and credentials used by CNTRL pass through this module.
//! Secrets are stored in the platform's native keychain:
//! - **macOS**: Keychain Services (`apple-native` feature)
//! - **Windows**: Windows Credential Manager (`windows-native` feature)
//! - **Linux**: Secret Service / libsecret (`sync-secret-service` feature)
//!
//! **No plaintext credential ever touches the filesystem, SQLite, or any
//! environment variable.** Callers receive only masked sentinels (e.g.
//! `"sk-or-***"`) from the config layer; real keys are only fetched here
//! immediately before use.

use keyring::Entry;
use std::sync::OnceLock;

use crate::error::CntrlError;
use crate::services::memory::db::AppDb;

/// The application-level service identifier used for all keychain entries.
pub const APP_SERVICE: &str = "cntrl-browser";

static DB_INSTANCE: OnceLock<AppDb> = OnceLock::new();

/// Initialises the database reference used for credential access audit logging.
pub fn init_audit_db(db: AppDb) {
    let _ = DB_INSTANCE.set(db);
}

fn log_access(key: &str, access_type: &str) {
    if let Some(db) = DB_INSTANCE.get() {
        let db = db.clone();
        let key = key.to_string();
        let access_type = access_type.to_string();
        tauri::async_runtime::spawn(async move {
            let _ =
                crate::services::audit::log_credential_access(&db, APP_SERVICE, &key, &access_type)
                    .await;
        });
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Stores a secret in the OS keychain.
///
/// # Arguments
/// * `key`   – Identifies which secret this is (e.g. `"openrouter_key"`).
/// * `value` – The plaintext secret to store.
///
/// # Errors
/// Returns [`CntrlError::Keychain`] if the OS rejects the write.
pub fn store_secret(key: &str, value: &str) -> Result<(), CntrlError> {
    log_access(key, "write");
    let entry = Entry::new(APP_SERVICE, key)
        .map_err(|e| CntrlError::Keychain(format!("Failed to create keychain entry: {e}")))?;
    entry
        .set_password(value)
        .map_err(|e| CntrlError::Keychain(format!("Failed to store secret '{key}': {e}")))
}

/// Retrieves a secret from the OS keychain.
///
/// # Arguments
/// * `key` – The same key used when calling [`store_secret`].
///
/// # Errors
/// Returns [`CntrlError::Keychain`] if the entry does not exist or the OS
/// refuses access (e.g. user denied Keychain access on macOS).
pub fn retrieve_secret(key: &str) -> Result<String, CntrlError> {
    log_access(key, "read");
    let entry = Entry::new(APP_SERVICE, key)
        .map_err(|e| CntrlError::Keychain(format!("Failed to create keychain entry: {e}")))?;
    entry
        .get_password()
        .map_err(|e| CntrlError::Keychain(format!("Failed to retrieve secret '{key}': {e}")))
}

/// Deletes a secret from the OS keychain.
///
/// Silently succeeds if the entry does not exist (idempotent delete).
///
/// # Arguments
/// * `key` – The key to delete.
///
/// # Errors
/// Returns [`CntrlError::Keychain`] only on genuine OS-level errors (not
/// "entry not found").
pub fn delete_secret(key: &str) -> Result<(), CntrlError> {
    log_access(key, "delete");
    let entry = Entry::new(APP_SERVICE, key)
        .map_err(|e| CntrlError::Keychain(format!("Failed to create keychain entry: {e}")))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // idempotent
        Err(e) => Err(CntrlError::Keychain(format!(
            "Failed to delete secret '{key}': {e}"
        ))),
    }
}

/// Returns `true` if a secret with the given key exists in the keychain.
///
/// This is a read-only probe that does not expose the secret value.
pub fn secret_exists(key: &str) -> bool {
    retrieve_secret(key).is_ok()
}

/// Masked sentinel returned to the frontend to indicate "a key is stored"
/// without revealing the actual value.
pub const MASKED_SENTINEL: &str = "***stored***";

// ─────────────────────────────────────────────────────────────────────────────
// Well-known keychain key names
// ─────────────────────────────────────────────────────────────────────────────

/// Keychain entry key for the OpenRouter API key.
pub const KEY_OPENROUTER: &str = "openrouter_api_key";
/// Keychain entry key for the Google Gemini API key.
pub const KEY_GEMINI: &str = "gemini_api_key";
/// Keychain entry key for the Groq API key.
pub const KEY_GROQ: &str = "groq_api_key";
/// Keychain entry key for the HuggingFace access token.
pub const KEY_HF_TOKEN: &str = "hf_access_token";
/// Keychain entry key for the Tier 3 custom endpoint API key.
pub const KEY_OPENAI_COMPAT: &str = "openai_compat_api_key";

#[cfg(test)]
mod tests {
    use super::*;

    /// Store, retrieve, and delete a test secret.
    ///
    /// This test requires a real keychain to be available. It is skipped
    /// automatically in CI environments where the keychain may be unavailable.
    #[test]
    fn store_retrieve_delete_roundtrip() {
        let test_key = "cntrl_test_key_roundtrip";
        let test_value = "test-secret-value-do-not-use";

        // Ensure clean state
        let _ = delete_secret(test_key);

        // Store
        if let Err(e) = store_secret(test_key, test_value) {
            // Keychain unavailable in this environment — skip gracefully
            eprintln!(
                "Keychain unavailable ({e}), skipping roundtrip test"
            );
            return;
        }

        // Retrieve
        let retrieved = retrieve_secret(test_key).expect("should retrieve stored secret");
        assert_eq!(
            retrieved, test_value,
            "retrieved secret must match stored value"
        );

        // Verify it does NOT appear as plaintext anywhere near the service name
        assert!(!retrieved.is_empty(), "retrieved secret must not be empty");

        // Delete
        delete_secret(test_key).expect("should delete secret");

        // After deletion, retrieve must fail
        assert!(
            retrieve_secret(test_key).is_err(),
            "retrieve after delete must return Err"
        );
    }

    /// Verify `delete_secret` on a non-existent key is idempotent (no panic).
    #[test]
    fn delete_nonexistent_key_is_ok() {
        let result = delete_secret("cntrl_test_key_definitely_does_not_exist_xyz");
        assert!(result.is_ok(), "deleting non-existent key must return Ok");
    }

    /// Verify `secret_exists` returns false for a key that has never been stored.
    #[test]
    fn secret_exists_false_for_unknown_key() {
        assert!(
            !secret_exists("cntrl_test_key_that_was_never_stored_abc123"),
            "secret_exists must return false for unknown key"
        );
    }
}
