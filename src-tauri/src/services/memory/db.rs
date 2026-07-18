use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use crate::error::CntrlError;

pub type AppDb = SqlitePool;

pub async fn open(db_path: &str) -> Result<AppDb, CntrlError> {
    let connection_string = format!("sqlite:{}?mode=rwc", db_path);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await?;

    run_migrations(&pool).await?;

    Ok(pool)
}

pub async fn open_in_memory() -> Result<AppDb, CntrlError> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;

    run_migrations(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &AppDb) -> Result<(), CntrlError> {
    sqlx::query("PRAGMA journal_mode=WAL;")
        .execute(pool)
        .await?;

    sqlx::query("PRAGMA foreign_keys=ON;").execute(pool).await?;

    let migration_sql = include_str!("../../../migrations/001_initial.sql");

    for stmt in migration_sql.split(';') {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(pool).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn open_in_memory_creates_all_tables() {
        let pool = open_in_memory().await.expect("in-memory DB must open");

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
