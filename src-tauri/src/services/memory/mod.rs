//! Memory subsystem — persistent learning and context recall.
//!
//! # Modules
//! - [`db`]      — SQLite connection pool and schema migrations.
//! - [`habits`]  — Records and retrieves site/service preferences.
//! - [`recall`]  — Keyword-based context retrieval with optional LanceDB
//!   vector search when Ollama is available.

pub mod db;
pub mod habits;
pub mod recall;
