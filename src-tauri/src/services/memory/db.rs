//! Database layer — SQLite connection pool and schema migrations.
//!
//! Provides a single shared [`AppDb`] handle that the rest of the memory
//! subsystem uses to execute queries. All queries use compile-time checked
//! macros where possible; runtime strings are used only for migration
//! bootstrapping.
//!
//! The database file lives in the Tauri app-data directory so it is
//! OS-appropriate (`~/Library/Application Support/com.cntrl.browser/` on macOS).
//! In tests an in-memory database is used instead.

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use crate::error::CntrlError;

/// Shared database handle — clone cheaply (it is an `Arc` internally).
pub type AppDb = SqlitePool;

/// Opens (or creates) the SQLite database at `db_path` and runs all pending
/// migrations from the embedded `migrations/` directory.
///
/// # Arguments
/// * `db_path` – Filesystem path including the filename, e.g.
///   `/Users/foo/Library/Application Support/com.cntrl.browser/cntrl.db`.
///
/// # Errors
/// Returns [`CntrlError::Database`] if the pool cannot be created or any
/// migration fails.
pub async fn open(db_path: &str) -> Result<AppDb, CntrlError> {
    // SQLite connection string; `?mode=rwc` creates the file if absent.
    let connection_string = format!("sqlite:{}?mode=rwc", db_path);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await?;

    run_migrations(&pool).await?;

    Ok(pool)
}

/// Opens an in-memory SQLite database. Used exclusively in tests.
///
/// # Errors
/// Returns [`CntrlError::Database`] if the pool cannot be created.
pub async fn open_in_memory() -> Result<AppDb, CntrlError> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        // Must use a named in-memory DB so multiple connections share state.
        .connect("sqlite::memory:")
        .await?;

    run_migrations(&pool).await?;

    Ok(pool)
}

/// Applies all SQL migrations embedded at compile time.
///
/// The migration SQL is embedded directly as a string constant rather than
/// using `sqlx::migrate!` macro to avoid requiring the `DATABASE_URL`
/// environment variable at compile time (incompatible with Tauri's cross-
/// compilation targets).
async fn run_migrations(pool: &AppDb) -> Result<(), CntrlError> {
    // Enable WAL mode for better concurrent read performance.
    sqlx::query("PRAGMA journal_mode=WAL;")
        .execute(pool)
        .await?;

    // Enable foreign key enforcement.
    sqlx::query("PRAGMA foreign_keys=ON;").execute(pool).await?;

    // Inline the migration SQL so no file I/O is required at runtime.
    // This is intentional: Tauri bundles everything into the binary.
    let migration_sql = include_str!("../../../migrations/001_initial.sql");

    // SQLite does not support multi-statement `execute`, so split on `;`.
    for stmt in migration_sql.split(';') {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(pool).await?;
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn open_in_memory_creates_all_tables() {
        let pool = open_in_memory().await.expect("in-memory DB must open");

        // Verify each expected table exists by querying sqlite_master.
        let tables = [
            "task_history",
            "preferences",
            "site_habits",
            "macro_library",
            "audit_log",
        ];
        for table in &tables {
            let row: (i64,) =
                sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?")
                    .bind(table)
                    .fetch_one(&pool)
                    .await
                    .unwrap_or_else(|_| panic!("failed to query sqlite_master for table {table}"));

            assert_eq!(row.0, 1, "table '{table}' must exist after migration");
        }
    }

    #[tokio::test]
    async fn default_preferences_are_seeded() {
        let pool = open_in_memory().await.expect("in-memory DB must open");

        let row: (String,) =
            sqlx::query_as("SELECT value FROM preferences WHERE key = 'privacy_mode'")
                .fetch_one(&pool)
                .await
                .expect("privacy_mode preference must be seeded");

        assert_eq!(row.0, "false", "default privacy_mode must be 'false'");
    }
}
