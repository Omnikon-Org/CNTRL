-- Migration 001: Initial schema for CNTRL Browser Phase 5
-- Memory Engine & Security Layer

-- ── Task History ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS task_history (
    id          TEXT    PRIMARY KEY,
    intent_raw  TEXT    NOT NULL,
    intent_type TEXT    NOT NULL,
    slots       TEXT    NOT NULL DEFAULT '{}',
    status      TEXT    NOT NULL,
    result      TEXT,
    created_at  TEXT    NOT NULL,
    updated_at  TEXT    NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_task_history_intent_type ON task_history(intent_type);
CREATE INDEX IF NOT EXISTS idx_task_history_created_at  ON task_history(created_at DESC);

-- ── Preferences ───────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS preferences (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
INSERT OR IGNORE INTO preferences(key, value, updated_at)
VALUES
    ('theme',        'dark',  datetime('now')),
    ('privacy_mode', 'false', datetime('now'));

-- ── Site Habits ───────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS site_habits (
    id                TEXT PRIMARY KEY,
    intent_type       TEXT NOT NULL,
    keyword           TEXT NOT NULL,
    preferred_service TEXT NOT NULL,
    use_count         INTEGER NOT NULL DEFAULT 1,
    last_used_at      TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_site_habits_unique
    ON site_habits(intent_type, keyword, preferred_service);
CREATE INDEX IF NOT EXISTS idx_site_habits_keyword ON site_habits(keyword);

-- ── Macro Library ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS macro_library (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL,
    description   TEXT NOT NULL DEFAULT '',
    author        TEXT NOT NULL DEFAULT '',
    steps_json    TEXT NOT NULL,
    triggers_json TEXT NOT NULL DEFAULT '[]',
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

-- ── Audit Log ─────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS audit_log (
    id                 TEXT    PRIMARY KEY,
    entry_type         TEXT    NOT NULL,
    intent             TEXT,
    tier_used          TEXT,
    provider_name      TEXT,
    latency_ms         INTEGER,
    tokens_used        INTEGER,
    success            INTEGER,
    credential_service TEXT,
    credential_key     TEXT,
    access_type        TEXT,
    created_at         TEXT    NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_audit_log_created_at ON audit_log(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_log_entry_type ON audit_log(entry_type);
