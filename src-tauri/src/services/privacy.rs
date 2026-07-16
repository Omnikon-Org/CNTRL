//! Privacy mode — blocks all remote AI calls when enabled.
//!
//! When privacy mode is active:
//! - All Tier 2 (Freemium) and Tier 3 (Premium) AI calls are rejected.
//! - Only the local Tier 1 (Ollama) provider may be used.
//! - The mode is persisted in the `preferences` SQLite table.
//!
//! The [`PrivacyGuard`] struct is stored in Tauri's managed state and holds a
//! cached boolean so every router call does not need a DB round-trip.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::error::CntrlError;
use crate::services::memory::db::AppDb;

// Preference key used in the `preferences` table.
const PREF_PRIVACY_MODE: &str = "privacy_mode";

/// Thread-safe privacy-mode guard.
///
/// Clone freely — it is an `Arc` wrapper.
#[derive(Clone)]
pub struct PrivacyGuard {
    enabled: Arc<AtomicBool>,
}

impl PrivacyGuard {
    /// Creates a [`PrivacyGuard`] initialised from the `privacy_mode`
    /// preference in the database.
    ///
    /// # Errors
    /// Returns [`CntrlError::Database`] on SQL failure.
    pub async fn load(db: &AppDb) -> Result<Self, CntrlError> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM preferences WHERE key = ?")
            .bind(PREF_PRIVACY_MODE)
            .fetch_optional(db)
            .await?;

        let enabled = row.map(|(v,)| v == "true").unwrap_or(false);

        Ok(Self {
            enabled: Arc::new(AtomicBool::new(enabled)),
        })
    }

    /// Returns `true` if privacy mode is currently active.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    /// Enables privacy mode and persists the change to the database.
    ///
    /// # Errors
    /// Returns [`CntrlError::Database`] on SQL failure.
    pub async fn enable(&self, db: &AppDb) -> Result<(), CntrlError> {
        self.enabled.store(true, Ordering::Release);
        persist(db, true).await
    }

    /// Disables privacy mode and persists the change to the database.
    ///
    /// # Errors
    /// Returns [`CntrlError::Database`] on SQL failure.
    pub async fn disable(&self, db: &AppDb) -> Result<(), CntrlError> {
        self.enabled.store(false, Ordering::Release);
        persist(db, false).await
    }

    /// Sets privacy mode to `enabled` and persists the change to the database.
    ///
    /// # Errors
    /// Returns [`CntrlError::Database`] on SQL failure.
    pub async fn set(&self, db: &AppDb, enabled: bool) -> Result<(), CntrlError> {
        self.enabled.store(enabled, Ordering::Release);
        persist(db, enabled).await
    }

    /// Returns an error if privacy mode is active and `tier` is not `"Local"`.
    ///
    /// Call this in the AI router before dispatching to any remote provider.
    ///
    /// # Errors
    /// Returns [`CntrlError::Ai`] if the call would break the privacy contract.
    pub fn check_tier(&self, tier: &str) -> Result<(), CntrlError> {
        if self.is_enabled() && tier != "Local" {
            Err(CntrlError::Ai(format!(
                "Privacy mode is active — remote AI calls (tier: {tier}) are blocked"
            )))
        } else {
            Ok(())
        }
    }
}

/// Writes the privacy mode value to the `preferences` table.
async fn persist(db: &AppDb, enabled: bool) -> Result<(), CntrlError> {
    let value = if enabled { "true" } else { "false" };
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO preferences(key, value, updated_at) VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(PREF_PRIVACY_MODE)
    .bind(value)
    .bind(&now)
    .execute(db)
    .await?;

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::memory::db::open_in_memory;

    #[tokio::test]
    async fn default_privacy_mode_is_false() {
        let db = open_in_memory().await.expect("DB must open");
        let guard = PrivacyGuard::load(&db).await.expect("load must succeed");
        assert!(!guard.is_enabled(), "privacy mode must be off by default");
    }

    #[tokio::test]
    async fn enable_persists_and_blocks_remote_tiers() {
        let db = open_in_memory().await.expect("DB must open");
        let guard = PrivacyGuard::load(&db).await.expect("load must succeed");

        guard.enable(&db).await.expect("enable must succeed");
        assert!(guard.is_enabled());

        // Remote tiers must be blocked
        assert!(guard.check_tier("Freemium").is_err());
        assert!(guard.check_tier("Premium").is_err());

        // Local is always allowed
        assert!(guard.check_tier("Local").is_ok());
    }

    #[tokio::test]
    async fn disable_unblocks_remote_tiers() {
        let db = open_in_memory().await.expect("DB must open");
        let guard = PrivacyGuard::load(&db).await.expect("load must succeed");

        guard.enable(&db).await.unwrap();
        guard.disable(&db).await.expect("disable must succeed");

        assert!(!guard.is_enabled());
        assert!(guard.check_tier("Freemium").is_ok());
    }

    #[tokio::test]
    async fn privacy_mode_persisted_across_reload() {
        let db = open_in_memory().await.expect("DB must open");
        let guard = PrivacyGuard::load(&db).await.expect("load must succeed");
        guard.enable(&db).await.unwrap();

        // Simulate reload by loading a new guard from the same DB.
        let reloaded = PrivacyGuard::load(&db).await.expect("reload must succeed");
        assert!(reloaded.is_enabled(), "enabled state must survive a reload");
    }
}
