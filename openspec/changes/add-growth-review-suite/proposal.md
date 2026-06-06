## Why

The old `growth-companion-mvp2` change captured a useful product direction, but it was written for the retired Python/FastAPI and single-HTML architecture. The desktop app now needs a clean Tauri/Rust version of the same growth-review intent: help the user review days, weeks, and longer life periods without reintroducing stale APIs or numeric personality weights.

## What Changes

- Add weekly growth summaries generated from local diary entries, event memories, conversation summaries, notes, and qualitative growth observations.
- Add life chapter generation for user-selected date ranges, producing longer narrative retrospectives grounded in existing records.
- Add a growth review UI surface that exposes diary review, weekly summaries, life chapters, and qualitative growth observations from the current React app.
- Preserve the current diary generation model: diary facts remain grounded in same-day direct sources; weekly summaries and chapters may connect longer-term patterns but must not invent unsupported events.
- Do not restore old Python/FastAPI endpoints, single-HTML UI assumptions, MBTI/eight-dimension personality weights, or standalone `data/{user_id}/diaries/` storage.

## Capabilities

### New Capabilities
- `weekly-growth-summary`: Generate, store, list, and regenerate weekly growth summaries in the desktop app.
- `life-chapter`: Generate, store, list, and view longer narrative chapters for a user-selected date range.
- `growth-review-ui`: Provide a React UI surface for diary review, weekly summaries, life chapters, and qualitative growth observations.

### Modified Capabilities
- `personality-engine`: Clarify that growth review uses qualitative observations and does not revive numeric personality-weight history.

## Impact

- Rust service layer under `frontend/src-tauri/src/services/`
- Tauri commands in `frontend/src-tauri/src/lib.rs`
- Frontend API wrappers under `frontend/src/features/api/`
- React panels/components under `frontend/src/components/`
- NoteStore and SQLite-backed memory data
- OpenSpec main specs when this change is eventually archived
