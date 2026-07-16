//! Audit log — append-only record of every AI call and credential access.
//!
//! All entries are written to the `audit_log` SQLite table. The table is
//! append-only: no rows are ever deleted or updated. A read-only view is
//! exposed to the Settings > Privacy UI.
//!
//! **Security note**: this module deliberately stores only metadata about
//! credential accesses (which key was read/written/deleted) — never the
//! credential value itself.

use chrono::Utc;
use uuid::Uuid;

use crate::error::CntrlError;
use crate::services::memory::db::AppDb;

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/// A single entry in the audit log.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AuditEntry {
    /// Unique identifier for this log entry.
    pub id: String,
    /// `"ai_call"` or `"credential_access"`.
    pub entry_type: String,
    // AI-call fields
    pub intent: Option<String>,
    pub tier_used: Option<String>,
    pub provider_name: Option<String>,
    pub latency_ms: Option<i64>,
    pub tokens_used: Option<i64>,
    pub success: Option<bool>,
    // Credential-access fields
    pub credential_service: Option<String>,
    pub credential_key: Option<String>,
    pub access_type: Option<String>,
    // Common
    pub created_at: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Writers
// ─────────────────────────────────────────────────────────────────────────────

/// Logs a completed AI provider call.
///
/// # Arguments
/// * `intent`        – The user's original intent string.
/// * `tier_used`     – Tier that handled the request (`"Local"`, `"Freemium"`, `"Premium"`).
/// * `provider_name` – Name of the specific provider (e.g. `"Ollama"`).
/// * `latency_ms`    – Round-trip time in milliseconds.
/// * `tokens_used`   – Token count returned by the provider, if available.
/// * `success`       – Whether the call returned a successful response.
///
/// # Errors
/// Returns [`CntrlError::Database`] on SQL failure.
pub async fn log_ai_call(
    db: &AppDb,
    intent: &str,
    tier_used: &str,
    provider_name: &str,
    latency_ms: i64,
    tokens_used: Option<i64>,
    success: bool,
) -> Result<(), CntrlError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO audit_log(id, entry_type, intent, tier_used, provider_name, latency_ms, tokens_used, success, created_at)
         VALUES (?, 'ai_call', ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(intent)
    .bind(tier_used)
    .bind(provider_name)
    .bind(latency_ms)
    .bind(tokens_used)
    .bind(success as i64)
    .bind(&now)
    .execute(db)
    .await?;

    Ok(())
}

/// Logs a credential access event (read / write / delete).
///
/// **Never pass the credential value** — only its key name.
///
/// # Arguments
/// * `credential_service` – The keychain service name (e.g. `"cntrl-browser"`).
/// * `credential_key`     – The key identifier (e.g. `"openrouter_api_key"`).
/// * `access_type`        – One of `"read"`, `"write"`, or `"delete"`.
///
/// # Errors
/// Returns [`CntrlError::Database`] on SQL failure.
pub async fn log_credential_access(
    db: &AppDb,
    credential_service: &str,
    credential_key: &str,
    access_type: &str,
) -> Result<(), CntrlError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO audit_log(id, entry_type, credential_service, credential_key, access_type, created_at)
         VALUES (?, 'credential_access', ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(credential_service)
    .bind(credential_key)
    .bind(access_type)
    .bind(&now)
    .execute(db)
    .await?;

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Readers
// ─────────────────────────────────────────────────────────────────────────────

/// Returns recent audit log entries, most recent first.
///
/// # Arguments
/// * `limit` – Maximum number of entries to return.
///
/// # Errors
/// Returns [`CntrlError::Database`] on SQL failure.
pub async fn get_recent_entries(db: &AppDb, limit: u32) -> Result<Vec<AuditEntry>, CntrlError> {
    use sqlx::Row;
    let rows = sqlx::query(
        r#"SELECT id, entry_type, intent, tier_used, provider_name, latency_ms,
                  tokens_used, success, credential_service, credential_key, access_type,
                  created_at
           FROM audit_log
           ORDER BY created_at DESC
           LIMIT ?"#,
    )
    .bind(limit)
    .fetch_all(db)
    .await?;

    let mut entries = Vec::new();
    for r in rows {
        let success_val: Option<i64> = r.try_get("success")?;
        entries.push(AuditEntry {
            id: r.try_get("id")?,
            entry_type: r.try_get("entry_type")?,
            intent: r.try_get::<Option<String>, _>("intent")?,
            tier_used: r.try_get::<Option<String>, _>("tier_used")?,
            provider_name: r.try_get::<Option<String>, _>("provider_name")?,
            latency_ms: r.try_get::<Option<i64>, _>("latency_ms")?,
            tokens_used: r.try_get::<Option<i64>, _>("tokens_used")?,
            success: success_val.map(|v| v != 0),
            credential_service: r.try_get::<Option<String>, _>("credential_service")?,
            credential_key: r.try_get::<Option<String>, _>("credential_key")?,
            access_type: r.try_get::<Option<String>, _>("access_type")?,
            created_at: r.try_get("created_at")?,
        });
    }

    Ok(entries)
}

/// Returns the total number of entries in the audit log.
///
/// # Errors
/// Returns [`CntrlError::Database`] on SQL failure.
pub async fn count_entries(db: &AppDb) -> Result<i64, CntrlError> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM audit_log")
        .fetch_one(db)
        .await?;
    Ok(row.0)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::memory::db::open_in_memory;

    #[tokio::test]
    async fn log_ai_call_creates_entry() {
        let db = open_in_memory().await.expect("DB must open");

        log_ai_call(
            &db,
            "search bitcoin",
            "Freemium",
            "OpenRouter",
            450,
            Some(120),
            true,
        )
        .await
        .expect("log must succeed");

        let count = count_entries(&db).await.expect("count must succeed");
        assert_eq!(count, 1);

        let entries = get_recent_entries(&db, 10)
            .await
            .expect("read must succeed");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry_type, "ai_call");
        assert_eq!(entries[0].intent.as_deref(), Some("search bitcoin"));
        assert_eq!(entries[0].tier_used.as_deref(), Some("Freemium"));
        assert_eq!(entries[0].success, Some(true));
    }

    #[tokio::test]
    async fn log_credential_access_creates_entry() {
        let db = open_in_memory().await.expect("DB must open");

        log_credential_access(&db, "cntrl-browser", "openrouter_api_key", "read")
            .await
            .expect("log must succeed");

        let entries = get_recent_entries(&db, 10)
            .await
            .expect("read must succeed");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry_type, "credential_access");
        assert_eq!(
            entries[0].credential_key.as_deref(),
            Some("openrouter_api_key")
        );
        assert_eq!(entries[0].access_type.as_deref(), Some("read"));
        // Verify no AI fields were accidentally populated
        assert!(entries[0].intent.is_none());
    }

    #[tokio::test]
    async fn audit_log_contains_no_plaintext_secrets() {
        let db = open_in_memory().await.expect("DB must open");

        // Log a credential access (correctly — no value stored)
        log_credential_access(&db, "cntrl-browser", "gemini_api_key", "write")
            .await
            .unwrap();

        // Dump the entire audit_log as raw text and verify no secret-like
        // string appears. In a real audit we'd also grep the SQLite file on disk.
        let entries = get_recent_entries(&db, 100).await.unwrap();
        for entry in &entries {
            let json = serde_json::to_string(entry).expect("must serialize");
            // A real API key starts with "AIza..." or "sk-..."; these patterns
            // must never appear in the audit log.
            assert!(
                !json.contains("AIza"),
                "audit log must not contain Gemini API key"
            );
            assert!(
                !json.contains("sk-"),
                "audit log must not contain OpenAI-style key"
            );
        }
    }

    #[tokio::test]
    async fn get_recent_respects_limit() {
        let db = open_in_memory().await.expect("DB must open");

        for i in 0..10 {
            log_ai_call(
                &db,
                &format!("intent {i}"),
                "Local",
                "Ollama",
                100,
                None,
                true,
            )
            .await
            .unwrap();
        }

        let entries = get_recent_entries(&db, 3).await.unwrap();
        assert_eq!(entries.len(), 3, "limit must be respected");
    }
}
