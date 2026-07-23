# `.vibe` Macro Format Specification

## Overview

The `.vibe` macro format defines structured, repeatable intent execution sequences. Macros can be recorded visually using the CNTRL Macro Recorder (`Cmd+M`), exported to JSON files, and scheduled to run automatically on a cron schedule.

---

## File Schema Specification

```json
{
  "id": "macro-uuid-v4",
  "name": "Daily Morning News Summary",
  "description": "Opens news sites and generates an AI summary",
  "created_at": "2026-07-19T10:00:00Z",
  "steps": [
    {
      "step_index": 0,
      "intent": "Navigate to news.ycombinator.com",
      "action_type": "navigate",
      "target": "https://news.ycombinator.com"
    },
    {
      "step_index": 1,
      "intent": "Summarize top 5 headlines",
      "action_type": "extract",
      "target": "table.itemlist"
    }
  ],
  "schedule": {
    "cron": "0 8 * * *",
    "enabled": true
  }
}
```

---

## Storage Location

Macros are saved locally on disk at:
- **macOS**: `~/.vibe/macros/<macro-id>.vibe`
- **Linux**: `~/.vibe/macros/<macro-id>.vibe`
- **Windows**: `%USERPROFILE%\.vibe\macros\<macro-id>.vibe`

---

## Cron Schedule Syntax

Macro schedules follow standard 5-field cron syntax:
- `0 8 * * *` — Every day at 08:00 AM.
- `*/30 * * * *` — Every 30 minutes.
- `0 9 * * 1` — Every Monday at 09:00 AM.
