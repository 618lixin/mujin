# Phase 7: Diary Generation Development Guide

This document is the implementation brief for the next Rust/Tauri migration step.
It is intended for DeepSeek to implement, and for Codex to review afterwards.

## Goal

Implement the missing **diary generation** module in the Tauri/Rust app.

The user-facing product promise is "conversation becomes diary". The current Rust
migration can chat, stream responses, extract events, store memory, and display
panels, but it still lacks the generated daily Markdown diary artifact. Phase 7
must close that product loop.

## Non-Goals

Do not implement these in Phase 7:

- Weekly reflection / personality engine.
- Project auto-merge.
- Notes updater for `companion_notes.md`.
- Multi-provider redesign.
- API key encryption.
- New database schema unless absolutely required.
- Major UI redesign.

Use existing storage, APIs, and UI patterns wherever possible.

## Current Context

Workspace root:

```text
D:\人格画像
```

Tauri app:

```text
D:\人格画像\frontend
```

Important existing files:

```text
frontend/src-tauri/src/lib.rs
frontend/src-tauri/src/services/chat.rs
frontend/src-tauri/src/services/config.rs
frontend/src-tauri/src/services/database.rs
frontend/src-tauri/src/services/llm.rs
frontend/src-tauri/src/services/memory.rs
frontend/src-tauri/src/services/notes.rs
frontend/src-tauri/src/services/types.rs
frontend/src/features/api/
frontend/src/components/panels/
```

Use Windows PowerShell commands. For frontend commands, use `npm.cmd`, not
`npm`, because `npm.ps1` may be blocked by execution policy.

## Existing Capabilities To Reuse

Rust side already has:

- `AiConfig` in `services/config.rs`.
- `call_llm` and `call_cheap_llm` in `services/llm.rs`.
- Event/query storage in `services/database.rs`.
- Core memory and chat history helpers in `services/memory.rs`.
- Note CRUD via `services/notes.rs`.
- Tauri command registration in `lib.rs`.

Frontend side already has:

- API wrappers under `frontend/src/features/api/`.
- Panels under `frontend/src/components/panels/`.
- Existing diary/note UI through the original note editor.

## Desired User Flow

Minimum expected flow:

1. User chats during the day.
2. Chat post-processing stores important events and conversation turns.
3. User triggers daily diary generation, or app triggers it later.
4. Rust gathers that date's events/conversation summaries.
5. LLM generates a Markdown diary.
6. Diary is saved as a note/artifact.
7. Frontend can list/read the generated diary.

Automatic daily trigger can be basic in this phase. Manual generation is required.

## Storage Decision

Prefer storing generated diaries as Markdown files through the existing note/file
system instead of inventing a new table.

Recommended category:

```text
diary
```

Generated note title:

```text
YYYY-MM-DD
```

Generated Markdown shape:

```markdown
# YYYY-MM-DD

...
```

If the existing notes API makes direct category insertion awkward, add small
helper functions in `services/notes.rs` rather than bypassing the note store with
ad hoc file writes. Keep metadata consistent.

## Rust Module To Add

Add:

```text
frontend/src-tauri/src/services/diary.rs
```

Register it in:

```text
frontend/src-tauri/src/services/mod.rs
frontend/src-tauri/src/lib.rs
```

## Tauri Commands

Implement these commands:

```rust
ai_generate_diary(user_id: String, date: Option<String>) -> Result<DiaryGenerateResult, AppError>
ai_get_diary_list(user_id: String, limit: Option<usize>) -> Result<Vec<DiaryEntry>, AppError>
ai_get_diary(user_id: String, date: String) -> Result<Option<DiaryEntry>, AppError>
ai_regenerate_diary(user_id: String, date: String) -> Result<DiaryGenerateResult, AppError>
```

Use camelCase serialization for returned structs.

Recommended types in `services/types.rs`:

```rust
pub struct DiaryEntry {
    pub date: String,
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct DiaryGenerateResult {
    pub date: String,
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub source_event_count: usize,
    pub source_turn_count: usize,
    pub regenerated: bool,
}
```

If using existing `Note`/`NoteMetadata` directly is cleaner, that is acceptable,
but the frontend API wrapper should expose a diary-specific shape.

## Date Handling

Use local date input format:

```text
YYYY-MM-DD
```

If `date` is `None`, use the current local date. Keep parsing simple and explicit.
Reject invalid date strings with `AppError::new("invalidDate", ...)`.

Do not use relative words like "today" in APIs.

## Source Data Selection

For a given date, gather:

- Events whose `created_at` falls within the local day.
- Conversation turns whose `created_at` falls within the local day.
- Core memory as high-level context.

If exact local timezone filtering is too invasive, use RFC3339 prefix filtering as
a first pass only if existing timestamps are UTC/local consistently. Prefer robust
range filtering if feasible:

```text
date 00:00:00 <= created_at < next_date 00:00:00
```

If database helper functions are missing, add narrowly scoped helpers:

```rust
query_events_by_date(user_id, date, limit)
query_conversation_turns_by_date(user_id, date, limit)
```

Do not query all data unbounded.

## LLM Prompt Requirements

The diary prompt should produce a diary, not a report.

Must follow:

- Output Markdown only.
- Start with `# YYYY-MM-DD`.
- Use warm, plain, natural language.
- Ground claims in the provided events/conversation summaries.
- Avoid over-interpreting.
- Mention uncertainty when needed.
- Do not fabricate events.
- If there is little source data, write a short diary instead of padding.

Suggested structure:

```markdown
# YYYY-MM-DD

今天你提到...

...
```

Keep max tokens moderate, e.g. 1200-1800.

Use the main model (`llm_model`) rather than cheap model unless there is a strong
reason not to. Diary is a user-facing artifact.

## Empty Day Behavior

If the date has no events and no conversation turns:

- Do not call LLM.
- Generate a short Markdown note saying there was not enough material.
- Return `sourceEventCount = 0` and `sourceTurnCount = 0`.

Example:

```markdown
# 2026-05-30

今天还没有足够的记录生成日记。
```

## Idempotency

`ai_generate_diary` should be idempotent:

- If a diary for the date already exists, return the existing diary unless the
  caller uses `ai_regenerate_diary`.
- `ai_regenerate_diary` should replace/update the existing diary content.

Do not create duplicate diary notes for the same date.

Implementation options:

- Search notes in category `diary` by title `YYYY-MM-DD`.
- Or store deterministic file names if the note store supports it.

Prefer whichever matches existing `services/notes.rs` patterns.

## Frontend API Wrappers

Add or extend:

```text
frontend/src/features/api/diary.ts
frontend/src/features/api/types.ts
```

Wrapper functions:

```ts
generateDiary(date?: string, userId = DEFAULT_USER_ID)
getDiaryList(limit = 30, userId = DEFAULT_USER_ID)
getDiary(date: string, userId = DEFAULT_USER_ID)
regenerateDiary(date: string, userId = DEFAULT_USER_ID)
```

Add Vitest coverage for command names and payloads.

## Frontend UI

Keep UI small and consistent.

Minimum acceptable UI:

- Add controls in the diary/note area or a small panel action to generate today's diary.
- Show loading/error state.
- After generation, open/select the generated diary note if feasible.

If integrating directly into the existing diary tab is risky, add a small button
in `GrowthPanel` or `ChatPanel` for manual generation. Prefer the existing diary
tab if the code path is clear.

Do not build a large new landing page or redesign the app.

## Scheduler Hook

Optional but useful:

- During daily first chat, check whether yesterday's diary exists.
- If missing, generate it in the background.

This is optional for Phase 7. Manual generation is required. If implemented, keep
it conservative and non-blocking.

## Error Handling

Rules:

- Missing API key should return a clear `llmConfig` error for generation.
- Empty day should not be an error.
- LLM failure should not corrupt existing diary.
- Regeneration should update only after successful LLM output.

Use `AppError` consistently.

## Tests Required

Rust tests:

- Date validation accepts `YYYY-MM-DD`.
- Date validation rejects invalid strings.
- Empty day diary generation path does not call LLM helper if structured so it can
  be tested.
- Diary note idempotency: second generate for same date does not create duplicate.
- Regenerate replaces content.

Frontend tests:

- `generateDiary` invokes `ai_generate_diary` with default user.
- `getDiaryList` invokes `ai_get_diary_list`.
- `getDiary` invokes `ai_get_diary`.
- `regenerateDiary` invokes `ai_regenerate_diary`.

Run:

```powershell
cd D:\人格画像\frontend
npm.cmd test -- --run
npm.cmd run build
```

Run Rust tests:

```powershell
cd D:\人格画像
cargo test --manifest-path frontend/src-tauri/Cargo.toml
```

Optional:

```powershell
cd D:\人格画像\frontend
npm.cmd run lint
```

Do not run frontend build and Tauri build in parallel. Vite can race on `dist/`.

## Acceptance Criteria

Phase 7 is complete when:

- A user can generate a Markdown diary for a date from existing Rust/Tauri data.
- The diary is persisted and can be read/listed later.
- Re-running generation for the same date does not create duplicates.
- Regeneration replaces the existing diary.
- Frontend has a usable manual trigger or visible entry point.
- Frontend API wrapper tests pass.
- Rust tests pass.
- `npm.cmd run build` passes.
- Existing chat, memory, and quick extract tests remain green.

## Review Checklist For Codex

After DeepSeek finishes, Codex should review:

- No duplicate diary notes for same date.
- No unbounded database reads.
- No prompt fabrication risk.
- LLM errors do not overwrite existing diary.
- New Tauri commands are registered.
- New frontend wrappers use correct command names.
- Tests cover idempotency and regeneration.
- No P2 scope creep: no half-baked encryption/provider redesign.
- `npm.cmd test -- --run`, `npm.cmd run build`, and `cargo test` all pass.

## Suggested Implementation Order

1. Add failing frontend wrapper tests for diary commands.
2. Add Rust types and `services/diary.rs` skeleton.
3. Add date parsing and source data query helpers with tests.
4. Implement empty-day diary path.
5. Implement LLM prompt and save/update diary note.
6. Implement idempotent generate and regenerate.
7. Register Tauri commands.
8. Add minimal UI trigger.
9. Run full verification.

