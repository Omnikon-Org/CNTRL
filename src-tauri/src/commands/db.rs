use tauri::State;

use crate::services::db::{
    BookmarkEntry, DbService, HistoryEntry, TabSessionEntry, TabSessionInput,
};

#[tauri::command]
pub fn db_add_history_entry(
    url: String,
    title: Option<String>,
    db: State<'_, DbService>,
) -> Result<HistoryEntry, String> {
    db.add_history_entry(&url, title.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn db_get_history(
    limit: Option<u32>,
    offset: Option<u32>,
    db: State<'_, DbService>,
) -> Result<Vec<HistoryEntry>, String> {
    db.get_history(limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn db_add_bookmark(
    url: String,
    title: Option<String>,
    db: State<'_, DbService>,
) -> Result<BookmarkEntry, String> {
    db.add_bookmark(&url, title.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn db_remove_bookmark(id: i64, db: State<'_, DbService>) -> Result<(), String> {
    db.remove_bookmark(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn db_get_bookmarks(db: State<'_, DbService>) -> Result<Vec<BookmarkEntry>, String> {
    db.get_bookmarks().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn db_save_session(tabs: Vec<TabSessionInput>, db: State<'_, DbService>) -> Result<(), String> {
    db.save_session(tabs).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn db_restore_session(db: State<'_, DbService>) -> Result<Vec<TabSessionEntry>, String> {
    db.restore_session().map_err(|e| e.to_string())
}
