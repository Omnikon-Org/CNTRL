//! Database service for persisting session, history, and bookmarks using SQLite.

use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

use crate::error::CntrlError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEntry {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub visited_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BookmarkEntry {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TabSessionEntry {
    pub id: i64,
    pub url: String,
    pub position: i32,
    pub is_active: bool,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TabSessionInput {
    pub url: String,
    pub position: i32,
    pub is_active: bool,
    pub title: Option<String>,
}

#[derive(Clone)]
pub struct DbService {
    conn: Arc<Mutex<Connection>>,
}

impl DbService {
    pub fn new(db_path: &Path) -> Result<Self, CntrlError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(db_path)?;
        let service = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        service.init()?;
        Ok(service)
    }

    pub fn new_in_memory() -> Result<Self, CntrlError> {
        let conn = Connection::open_in_memory()?;
        let service = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        service.init()?;
        Ok(service)
    }

    fn init(&self) -> Result<(), CntrlError> {
        let conn = self.conn.lock();
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;
            ",
        )?;

        self.run_migrations_with_conn(&conn)?;
        Ok(())
    }

    fn run_migrations_with_conn(&self, conn: &Connection) -> Result<(), CntrlError> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                title TEXT,
                visited_at INTEGER NOT NULL
            );",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS bookmarks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                title TEXT,
                created_at INTEGER NOT NULL
            );",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tabs_session (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                position INTEGER NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT 0,
                title TEXT
            );",
            [],
        )?;

        Ok(())
    }

    // ── History Methods ────────────────────────────────────────────────────────

    pub fn add_history_entry(
        &self,
        url: &str,
        title: Option<&str>,
    ) -> Result<HistoryEntry, CntrlError> {
        let conn = self.conn.lock();
        let visited_at = Utc::now().timestamp();
        conn.execute(
            "INSERT INTO history (url, title, visited_at) VALUES (?1, ?2, ?3)",
            params![url, title, visited_at],
        )?;
        let id = conn.last_insert_rowid();

        Ok(HistoryEntry {
            id,
            url: url.to_string(),
            title: title.map(|s| s.to_string()),
            visited_at,
        })
    }

    pub fn get_history(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<HistoryEntry>, CntrlError> {
        let conn = self.conn.lock();
        let limit_val = limit.unwrap_or(100) as i64;
        let offset_val = offset.unwrap_or(0) as i64;

        let mut stmt = conn.prepare(
            "SELECT id, url, title, visited_at FROM history ORDER BY visited_at DESC, id DESC LIMIT ?1 OFFSET ?2",
        )?;

        let history_iter = stmt.query_map(params![limit_val, offset_val], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                visited_at: row.get(3)?,
            })
        })?;

        let mut entries = Vec::new();
        for entry in history_iter {
            entries.push(entry?);
        }

        Ok(entries)
    }

    // ── Bookmark Methods ──────────────────────────────────────────────────────

    pub fn add_bookmark(
        &self,
        url: &str,
        title: Option<&str>,
    ) -> Result<BookmarkEntry, CntrlError> {
        let conn = self.conn.lock();
        let created_at = Utc::now().timestamp();
        conn.execute(
            "INSERT INTO bookmarks (url, title, created_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(url) DO UPDATE SET title = excluded.title, created_at = excluded.created_at",
            params![url, title, created_at],
        )?;

        let mut stmt = conn.prepare(
            "SELECT id, url, title, created_at FROM bookmarks WHERE url = ?1",
        )?;
        let entry = stmt.query_row(params![url], |row| {
            Ok(BookmarkEntry {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        Ok(entry)
    }

    pub fn remove_bookmark(&self, id: i64) -> Result<(), CntrlError> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM bookmarks WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_bookmarks(&self) -> Result<Vec<BookmarkEntry>, CntrlError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, url, title, created_at FROM bookmarks ORDER BY created_at DESC, id DESC",
        )?;

        let bookmark_iter = stmt.query_map([], |row| {
            Ok(BookmarkEntry {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        let mut entries = Vec::new();
        for entry in bookmark_iter {
            entries.push(entry?);
        }

        Ok(entries)
    }

    // ── Session Methods ───────────────────────────────────────────────────────

    pub fn save_session(&self, tabs: Vec<TabSessionInput>) -> Result<(), CntrlError> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM tabs_session", [])?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO tabs_session (url, position, is_active, title) VALUES (?1, ?2, ?3, ?4)",
            )?;

            for tab in tabs {
                stmt.execute(params![
                    tab.url,
                    tab.position,
                    if tab.is_active { 1 } else { 0 },
                    tab.title
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn restore_session(&self) -> Result<Vec<TabSessionEntry>, CntrlError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, url, position, is_active, title FROM tabs_session ORDER BY position ASC",
        )?;

        let session_iter = stmt.query_map([], |row| {
            let active_int: i32 = row.get(3)?;
            Ok(TabSessionEntry {
                id: row.get(0)?,
                url: row.get(1)?,
                position: row.get(2)?,
                is_active: active_int != 0,
                title: row.get(4)?,
            })
        })?;

        let mut entries = Vec::new();
        for entry in session_iter {
            entries.push(entry?);
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_db_migrations() {
        let db = DbService::new_in_memory().expect("failed to init in-memory DB");
        assert!(db.get_history(None, None).unwrap().is_empty());
        assert!(db.get_bookmarks().unwrap().is_empty());
        assert!(db.restore_session().unwrap().is_empty());
    }

    #[test]
    fn test_history_crud() {
        let db = DbService::new_in_memory().unwrap();
        let entry1 = db
            .add_history_entry("https://example.com", Some("Example"))
            .unwrap();
        let entry2 = db
            .add_history_entry("https://rust-lang.org", Some("Rust"))
            .unwrap();

        let history = db.get_history(Some(10), Some(0)).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(entry1.url, "https://example.com");
        assert_eq!(entry2.url, "https://rust-lang.org");

        // Test pagination
        let paginated = db.get_history(Some(1), Some(0)).unwrap();
        assert_eq!(paginated.len(), 1);
    }

    #[test]
    fn test_bookmarks_crud() {
        let db = DbService::new_in_memory().unwrap();
        let bm1 = db
            .add_bookmark("https://github.com", Some("GitHub"))
            .unwrap();
        assert_eq!(bm1.url, "https://github.com");
        assert_eq!(bm1.title.as_deref(), Some("GitHub"));

        let bookmarks = db.get_bookmarks().unwrap();
        assert_eq!(bookmarks.len(), 1);

        // Test upsert on duplicate URL
        let bm1_updated = db
            .add_bookmark("https://github.com", Some("GitHub Updated"))
            .unwrap();
        assert_eq!(bm1_updated.id, bm1.id);
        assert_eq!(bm1_updated.title.as_deref(), Some("GitHub Updated"));

        // Delete bookmark
        db.remove_bookmark(bm1.id).unwrap();
        assert!(db.get_bookmarks().unwrap().is_empty());
    }

    #[test]
    fn test_session_save_and_restore() {
        let db = DbService::new_in_memory().unwrap();

        let tabs = vec![
            TabSessionInput {
                url: "https://google.com".to_string(),
                position: 0,
                is_active: true,
                title: Some("Google".to_string()),
            },
            TabSessionInput {
                url: "https://news.ycombinator.com".to_string(),
                position: 1,
                is_active: false,
                title: Some("Hacker News".to_string()),
            },
        ];

        db.save_session(tabs).unwrap();

        let restored = db.restore_session().unwrap();
        assert_eq!(restored.len(), 2);
        assert_eq!(restored[0].url, "https://google.com");
        assert!(restored[0].is_active);
        assert_eq!(restored[1].url, "https://news.ycombinator.com");
        assert!(!restored[1].is_active);

        // Overwrite session
        let new_tabs = vec![TabSessionInput {
            url: "about:blank".to_string(),
            position: 0,
            is_active: true,
            title: Some("New Tab".to_string()),
        }];
        db.save_session(new_tabs).unwrap();
        let restored_new = db.restore_session().unwrap();
        assert_eq!(restored_new.len(), 1);
        assert_eq!(restored_new[0].url, "about:blank");
    }
}
