//! Site-habits tracker — learns which services the user prefers for each intent.
//!
//! Every time a task completes successfully, [`record_outcome`] is called to
//! increment the use-count for the `(intent_type, keyword, service)` triple.
//! [`find_preferred_service`] returns the most-used service for a given keyword,
//! which the planner can use to pre-select a destination.

use chrono::Utc;
use uuid::Uuid;

use super::db::AppDb;
use crate::error::CntrlError;

/// A single site-habit entry returned from the database.
#[derive(Debug, Clone)]
pub struct SiteHabit {
    /// The intent classification this habit belongs to (e.g. `"navigate"`).
    pub intent_type: String,
    /// The keyword that triggered the intent (e.g. `"lo-fi"`).
    pub keyword: String,
    /// The preferred service domain (e.g. `"youtube.com"`).
    pub preferred_service: String,
    /// How many times this combination has been used.
    pub use_count: i64,
}

/// Records that a task with `intent_type` and `keyword` resolved to `service`.
///
/// If the `(intent_type, keyword, service)` triple already exists its
/// `use_count` is incremented and `last_used_at` is updated. Otherwise a new
/// row is inserted.
///
/// # Errors
/// Returns [`CntrlError::Database`] on SQL failure.
pub async fn record_outcome(
    db: &AppDb,
    intent_type: &str,
    keyword: &str,
    service: &str,
) -> Result<(), CntrlError> {
    let now = Utc::now().to_rfc3339();

    // Attempt an upsert: increment count if exists, insert otherwise.
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM site_habits WHERE intent_type = ? AND keyword = ? AND preferred_service = ?",
    )
    .bind(intent_type)
    .bind(keyword)
    .bind(service)
    .fetch_optional(db)
    .await?;

    if let Some((id,)) = existing {
        sqlx::query(
            "UPDATE site_habits SET use_count = use_count + 1, last_used_at = ? WHERE id = ?",
        )
        .bind(&now)
        .bind(&id)
        .execute(db)
        .await?;
    } else {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO site_habits(id, intent_type, keyword, preferred_service, use_count, last_used_at)
             VALUES (?, ?, ?, ?, 1, ?)",
        )
        .bind(&id)
        .bind(intent_type)
        .bind(keyword)
        .bind(service)
        .bind(&now)
        .execute(db)
        .await?;
    }

    Ok(())
}

/// Returns the most-used preferred service for the given `keyword`, across all
/// intent types, or `None` if no habit has been recorded yet.
///
/// # Errors
/// Returns [`CntrlError::Database`] on SQL failure.
pub async fn find_preferred_service(
    db: &AppDb,
    keyword: &str,
) -> Result<Option<SiteHabit>, CntrlError> {
    let row: Option<(String, String, String, i64)> = sqlx::query_as(
        "SELECT intent_type, keyword, preferred_service, use_count
         FROM site_habits
         WHERE keyword = ?
         ORDER BY use_count DESC
         LIMIT 1",
    )
    .bind(keyword)
    .fetch_optional(db)
    .await?;

    Ok(row.map(
        |(intent_type, keyword, preferred_service, use_count)| SiteHabit {
            intent_type,
            keyword,
            preferred_service,
            use_count,
        },
    ))
}

/// Returns all recorded habits, ordered by most-used first.
///
/// # Errors
/// Returns [`CntrlError::Database`] on SQL failure.
pub async fn list_habits(db: &AppDb) -> Result<Vec<SiteHabit>, CntrlError> {
    let rows: Vec<(String, String, String, i64)> = sqlx::query_as(
        "SELECT intent_type, keyword, preferred_service, use_count
         FROM site_habits
         ORDER BY use_count DESC",
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(intent_type, keyword, preferred_service, use_count)| SiteHabit {
                intent_type,
                keyword,
                preferred_service,
                use_count,
            },
        )
        .collect())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::memory::db::open_in_memory;

    #[tokio::test]
    async fn record_and_retrieve_habit() {
        let db = open_in_memory().await.expect("DB must open");

        record_outcome(&db, "navigate", "lo-fi", "youtube.com")
            .await
            .expect("record must succeed");

        let habit = find_preferred_service(&db, "lo-fi")
            .await
            .expect("query must succeed")
            .expect("habit must exist");

        assert_eq!(habit.preferred_service, "youtube.com");
        assert_eq!(habit.use_count, 1);
    }

    #[tokio::test]
    async fn use_count_increments_on_repeat() {
        let db = open_in_memory().await.expect("DB must open");

        for _ in 0..5 {
            record_outcome(&db, "navigate", "music", "spotify.com")
                .await
                .expect("record must succeed");
        }

        let habit = find_preferred_service(&db, "music")
            .await
            .expect("query must succeed")
            .expect("habit must exist");

        assert_eq!(habit.use_count, 5, "use_count must be 5 after 5 records");
    }

    #[tokio::test]
    async fn most_used_wins_when_multiple_services() {
        let db = open_in_memory().await.expect("DB must open");

        // youtube used 3 times, soundcloud only once
        for _ in 0..3 {
            record_outcome(&db, "navigate", "jazz", "youtube.com")
                .await
                .unwrap();
        }
        record_outcome(&db, "navigate", "jazz", "soundcloud.com")
            .await
            .unwrap();

        let habit = find_preferred_service(&db, "jazz")
            .await
            .unwrap()
            .expect("habit must exist");

        assert_eq!(
            habit.preferred_service, "youtube.com",
            "youtube must win with 3 uses"
        );
    }

    #[tokio::test]
    async fn no_habit_returns_none() {
        let db = open_in_memory().await.expect("DB must open");
        let result = find_preferred_service(&db, "keyword_never_recorded")
            .await
            .expect("query must succeed");
        assert!(result.is_none(), "should return None for unknown keyword");
    }
}
