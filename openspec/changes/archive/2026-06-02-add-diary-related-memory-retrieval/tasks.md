## 1. Retrieval Module Boundary

- [x] 1.1 Add `services/diary_memory.rs` with `DiaryMemoryQuery` and `RelatedDiaryMemory` data structures
- [x] 1.2 Add unit tests for formatting an empty and non-empty related-memory prompt block
- [x] 1.3 Add unit tests proving same-day events are excluded from related past memories

## 2. Current Backend Retrieval Strategy

- [x] 2.1 Add database helper queries for past events by topic before a target date
- [x] 2.2 Add database helper queries for past events by keyword before a target date
- [x] 2.3 Add database helper queries for past conversation summaries by keyword before a target date
- [x] 2.4 Implement retrieval scoring using topic match, keyword match, importance, strength, and recency
- [x] 2.5 Add tests proving topic-linked events rank ahead of unrelated recent events
- [x] 2.6 Add tests proving keyword fallback works when topic links are absent

## 3. Diary Generation Integration

- [x] 3.1 Extend `build_diary_prompt` to accept an optional related-memory section
- [x] 3.2 Add prompt tests for related memories available and absent
- [x] 3.3 Call `retrieve_related_diary_memories` from `generate_diary_inner`
- [x] 3.4 Ensure empty-day diary generation still skips LLM and related-memory retrieval

## 4. Recall Tracking

- [x] 4.1 Record recall only for related event memories surfaced to a successfully generated diary prompt
- [x] 4.2 Add tests proving recall is not recorded when diary generation fails before save
- [x] 4.3 Add tests proving related memories without event ids do not attempt event recall

## 5. Validation

- [x] 5.1 Run `cargo test` in `frontend/src-tauri`
- [x] 5.2 Run `npm.cmd test` in `frontend`
- [x] 5.3 Run `npm.cmd run build` in `frontend`
- [x] 5.4 Run `openspec.cmd validate --all --strict --json`
