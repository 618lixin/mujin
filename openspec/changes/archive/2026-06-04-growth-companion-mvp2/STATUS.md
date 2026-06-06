# Status: superseded by Tauri/Rust refactor

This change was created before the later desktop refactor and no longer reflects the current implementation boundary.

Current source of truth:

- Main capability specs: `openspec/specs/`
- Desktop app entry: `frontend/`
- Rust service layer: `frontend/src-tauri/src/services/`
- Frontend command wrappers: `frontend/src/features/api/`

Do not use this change as an implementation checklist without first rewriting it against the current Tauri/Rust architecture.

Known stale assumptions in this change:

- Python/FastAPI backend endpoints
- Old REST API routes
- MBTI/eight-dimension personality weights
- Personality snapshot/history flow
- Separate diary files under `data/{user_id}/diaries/`

The current implementation uses Tauri commands, Rust services, SQLite, and Markdown NoteStore-backed diaries.
