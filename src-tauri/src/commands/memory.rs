use tauri::State;

use crate::services::audit::{count_entries, get_recent_entries, AuditEntry};
use crate::services::memory::db::AppDb;
use crate::services::memory::habits::list_habits;
use crate::services::privacy::PrivacyGuard;

#[tauri::command]
pub async fn get_preference(key: String, db: State<'_, AppDb>) -> Result<Option<String>, String> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM preferences WHERE key = ?")
        .bind(key)
        .fetch_optional(db.inner())
        .await
        .map_err(|e| e.to_string())?;

    Ok(row.map(|r| r.0))
}

/// Sets a preference value in the database.
#[tauri::command]
pub async fn set_preference(
    key: String,
    value: String,
    db: State<'_, AppDb>,
) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO preferences(key, value, updated_at) VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(key)
    .bind(value)
    .bind(&now)
    .execute(db.inner())
    .await
    .map(|_| ())
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn is_privacy_mode_enabled(privacy_guard: State<'_, PrivacyGuard>) -> Result<bool, String> {
    Ok(privacy_guard.is_enabled())
}

/// Enables or disables privacy mode.
#[tauri::command]
pub async fn set_privacy_mode(
    enabled: bool,
    privacy_guard: State<'_, PrivacyGuard>,
    db: State<'_, AppDb>,
) -> Result<(), String> {
    privacy_guard
        .set(db.inner(), enabled)
        .await
        .map_err(|e| e.to_string())
}

/// Returns recent audit log entries.
#[tauri::command]
pub async fn get_recent_audit_log(
    limit: u32,
    db: State<'_, AppDb>,
) -> Result<Vec<AuditEntry>, String> {
    get_recent_entries(db.inner(), limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_audit_log_count(db: State<'_, AppDb>) -> Result<i64, String> {
    count_entries(db.inner()).await.map_err(|e| e.to_string())
}

/// Structure mapped for frontend consumption.
#[derive(serde::Serialize)]
pub struct FeSiteHabit {
    pub intent_type: String,
    pub keyword: String,
    pub preferred_service: String,
    pub use_count: i64,
}

/// Returns all recorded site habits.
#[tauri::command]
pub async fn get_site_habits(db: State<'_, AppDb>) -> Result<Vec<FeSiteHabit>, String> {
    let list = list_habits(db.inner()).await.map_err(|e| e.to_string())?;

    Ok(list
        .into_iter()
        .map(|h| FeSiteHabit {
            intent_type: h.intent_type,
            keyword: h.keyword,
            preferred_service: h.preferred_service,
            use_count: h.use_count,
        })
        .collect())
}
