use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::error::CntrlError;
use crate::services::memory::db::AppDb;

const PREF_PRIVACY_MODE: &str = "privacy_mode";

#[derive(Clone)]
pub struct PrivacyGuard {
    enabled: Arc<AtomicBool>,
}

impl PrivacyGuard {
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

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    pub async fn enable(&self, db: &AppDb) -> Result<(), CntrlError> {
        self.enabled.store(true, Ordering::Release);
        persist(db, true).await
    }

    pub async fn disable(&self, db: &AppDb) -> Result<(), CntrlError> {
        self.enabled.store(false, Ordering::Release);
        persist(db, false).await
    }

    pub async fn set(&self, db: &AppDb, enabled: bool) -> Result<(), CntrlError> {
        self.enabled.store(enabled, Ordering::Release);
        persist(db, enabled).await
    }

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

        assert!(guard.check_tier("Freemium").is_err());
        assert!(guard.check_tier("Premium").is_err());

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

        let reloaded = PrivacyGuard::load(&db).await.expect("reload must succeed");
        assert!(reloaded.is_enabled(), "enabled state must survive a reload");
    }
}
