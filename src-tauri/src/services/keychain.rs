use keyring::Entry;
use std::sync::OnceLock;

use crate::error::CntrlError;
use crate::services::memory::db::AppDb;

pub const APP_SERVICE: &str = "cntrl-browser";

static DB_INSTANCE: OnceLock<AppDb> = OnceLock::new();

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

pub fn store_secret(key: &str, value: &str) -> Result<(), CntrlError> {
    log_access(key, "write");
    let entry = Entry::new(APP_SERVICE, key)
        .map_err(|e| CntrlError::Keychain(format!("Failed to create keychain entry: {e}")))?;
    entry
        .set_password(value)
        .map_err(|e| CntrlError::Keychain(format!("Failed to store secret '{key}': {e}")))
}

pub fn retrieve_secret(key: &str) -> Result<String, CntrlError> {
    log_access(key, "read");
    let entry = Entry::new(APP_SERVICE, key)
        .map_err(|e| CntrlError::Keychain(format!("Failed to create keychain entry: {e}")))?;
    entry
        .get_password()
        .map_err(|e| CntrlError::Keychain(format!("Failed to retrieve secret '{key}': {e}")))
}

pub fn delete_secret(key: &str) -> Result<(), CntrlError> {
    log_access(key, "delete");
    let entry = Entry::new(APP_SERVICE, key)
        .map_err(|e| CntrlError::Keychain(format!("Failed to create keychain entry: {e}")))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(CntrlError::Keychain(format!(
            "Failed to delete secret '{key}': {e}"
        ))),
    }
}

pub fn secret_exists(key: &str) -> bool {
    retrieve_secret(key).is_ok()
}

pub const MASKED_SENTINEL: &str = "***stored***";

pub const KEY_OPENROUTER: &str = "openrouter_api_key";
pub const KEY_GEMINI: &str = "gemini_api_key";
pub const KEY_GROQ: &str = "groq_api_key";
pub const KEY_HF_TOKEN: &str = "hf_access_token";
pub const KEY_OPENAI_COMPAT: &str = "openai_compat_api_key";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_retrieve_delete_roundtrip() {
        let test_key = "cntrl_test_key_roundtrip";
        let test_value = "test-secret-value-do-not-use";

        let _ = delete_secret(test_key);

        if let Err(e) = store_secret(test_key, test_value) {
            eprintln!("Keychain unavailable ({e}), skipping roundtrip test");
            return;
        }

        let retrieved = retrieve_secret(test_key).expect("should retrieve stored secret");
        assert_eq!(
            retrieved, test_value,
            "retrieved secret must match stored value"
        );

        assert!(!retrieved.is_empty(), "retrieved secret must not be empty");

        delete_secret(test_key).expect("should delete secret");

        assert!(
            retrieve_secret(test_key).is_err(),
            "retrieve after delete must return Err"
        );
    }

    #[test]
    fn delete_nonexistent_key_is_ok() {
        let result = delete_secret("cntrl_test_key_definitely_does_not_exist_xyz");
        assert!(result.is_ok(), "deleting non-existent key must return Ok");
    }

    #[test]
    fn secret_exists_false_for_unknown_key() {
        assert!(
            !secret_exists("cntrl_test_key_that_was_never_stored_abc123"),
            "secret_exists must return false for unknown key"
        );
    }
}
