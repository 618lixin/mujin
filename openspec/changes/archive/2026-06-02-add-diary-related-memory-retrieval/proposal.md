## Why

Current diary generation summarizes only same-day events, conversation turns, notes, and core memory. This produces useful daily summaries, but it cannot naturally connect today's experience with earlier related events, so the generated diary feels less like it was written by someone who remembers the user's life.

The memory system is likely to be refactored later, so diary-related memory retrieval should be introduced behind a small module boundary instead of coupling diary generation directly to the current SQLite/LIKE implementation.

## What Changes

- Add a diary-specific related memory retrieval capability for diary generation.
- Retrieve a small set of past events and conversation summaries that are relevant to the diary date's source material.
- Add a new prompt section for "possibly related past memories" with instructions to use them only when the connection is natural and evidence-based.
- Keep the retrieval implementation modular so the current SQLite/topic/LIKE strategy can later be replaced by semantic search, topic graph traversal, or another memory backend.
- Track which retrieved events were actually surfaced to the diary prompt so existing recall mechanics can be applied consistently.
- No breaking changes to current Tauri commands or diary APIs.

## Capabilities

### New Capabilities
- `diary-related-memory-retrieval`: Retrieves and formats past memories relevant to a diary date through a replaceable module boundary.

### Modified Capabilities
- `diary-generator`: Diary generation will include an optional related-memory context section in addition to same-day source material.
- `event-memory`: Event memory will expose retrieval primitives suitable for diary-related memory lookup and recall tracking.

## Impact

- `frontend/src-tauri/src/services/diary.rs`: consume related diary memories while building diary prompts.
- `frontend/src-tauri/src/services/database.rs`: may add focused query helpers for past events, topic links, and/or keyword lookup.
- New Rust module under `frontend/src-tauri/src/services/` for diary memory retrieval orchestration.
- `openspec/specs/diary-generator/spec.md`: update diary generation requirements.
- `openspec/specs/event-memory/spec.md`: update memory retrieval/recall requirements.
- Tests: add unit tests for retrieval selection, formatting, prompt inclusion, exclusion of same-day events, and recall tracking behavior.
