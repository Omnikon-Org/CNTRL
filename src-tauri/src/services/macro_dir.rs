// services/macro_dir.rs
//
// Manages the `~/.vibe/macros/` directory where .vibe macro files are stored.
// All file I/O in this module is synchronous (called from async Tauri commands
// via `tokio::task::spawn_blocking` at the call site).

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::CntrlError;
use crate::services::macro_format::Vibemacro;

/// Returns the canonical macros directory path, creating it if absent.
pub fn macros_dir() -> Result<PathBuf, CntrlError> {
    let home =
        dirs::home_dir().ok_or_else(|| CntrlError::Macro("Cannot locate home directory".into()))?;
    let dir = home.join(".vibe").join("macros");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Save a `Vibemacro` to `~/.vibe/macros/<safe-filename>.vibe`.
/// Returns the full path of the written file.
pub fn save_macro(mac: &Vibemacro) -> Result<PathBuf, CntrlError> {
    let dir = macros_dir()?;
    let filename = mac.safe_filename();
    let path = dir.join(&filename);
    let json = mac.to_json()?;
    fs::write(&path, json.as_bytes())?;
    Ok(path)
}

/// Load a single `.vibe` file by path.
pub fn load_macro(path: &Path) -> Result<Vibemacro, CntrlError> {
    let json = fs::read_to_string(path)?;
    Vibemacro::from_json(&json)
}

/// List all macros in `~/.vibe/macros/` as deserialized `Vibemacro` values.
/// Files that fail to parse are skipped with a warning.
pub fn list_macros() -> Result<Vec<Vibemacro>, CntrlError> {
    let dir = macros_dir()?;
    let mut macros = Vec::new();

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("vibe") {
            match load_macro(&path) {
                Ok(m) => macros.push(m),
                Err(e) => eprintln!("[macro_dir] Skipping {:?}: {e}", path.file_name()),
            }
        }
    }

    // Sort by creation time, newest first
    macros.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(macros)
}

/// Delete a macro file by its ID.
/// Scans the macros directory for the file whose embedded `id` matches.
pub fn delete_macro(macro_id: &str) -> Result<(), CntrlError> {
    let dir = macros_dir()?;
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("vibe") {
            if let Ok(m) = load_macro(&path) {
                if m.id == macro_id {
                    fs::remove_file(&path)?;
                    return Ok(());
                }
            }
        }
    }
    Err(CntrlError::Macro(format!(
        "Macro '{macro_id}' not found in ~/.vibe/macros/"
    )))
}

/// Load a macro by ID.
pub fn get_macro(macro_id: &str) -> Result<Vibemacro, CntrlError> {
    let dir = macros_dir()?;
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("vibe") {
            if let Ok(m) = load_macro(&path) {
                if m.id == macro_id {
                    return Ok(m);
                }
            }
        }
    }
    Err(CntrlError::Macro(format!("Macro '{macro_id}' not found")))
}
