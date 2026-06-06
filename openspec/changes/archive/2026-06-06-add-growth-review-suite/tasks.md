## 1. Weekly Growth Summary Backend

- [x] 1.1 Add failing Rust tests for weekly summary source aggregation, sparse-source handling, and same-week regeneration.
- [x] 1.2 Implement weekly summary request/result types in the Rust service type layer.
- [x] 1.3 Implement a `weekly_summary` service module that gathers diaries, events, turns, notes, and observations for a local week.
- [x] 1.4 Add a grounded weekly summary prompt builder with tests for source counts and anti-hallucination wording.
- [x] 1.5 Store generated weekly summaries as NoteStore Markdown notes and replace the existing week note on regeneration.
- [x] 1.6 Use ISO year/week as the weekly summary identity and include `week_display_range` for UI display.

## 2. Life Chapter Backend

- [x] 2.1 Add failing Rust tests for chapter date-range validation, sparse-source handling, and long-range source limiting.
- [x] 2.2 Implement life chapter request/result types in the Rust service type layer.
- [x] 2.3 Implement a `life_chapter` service module that gathers date-range sources from NoteStore and SQLite.
- [x] 2.4 Add a grounded chapter prompt builder with tests for missing-detail constraints.
- [x] 2.5 Store generated chapters as NoteStore Markdown notes with date-range metadata.
- [x] 2.6 Generate editable chapter titles from date range and source themes without requiring title input.
- [x] 2.7 Use stable chapter filenames or note identities that remain valid when a title is edited.

## 3. Tauri Commands And Frontend API

- [x] 3.1 Add Tauri commands for listing, generating, and regenerating weekly summaries.
- [x] 3.2 Add Tauri commands for listing and generating life chapters.
- [x] 3.3 Add frontend API wrappers and tests for weekly summary commands.
- [x] 3.4 Add frontend API wrappers and tests for life chapter commands.

## 4. Growth Review UI

- [x] 4.1 Add or update the growth review panel entry in the main React UI.
- [x] 4.2 Expose diary list, detail, generate, and regenerate controls through the review panel.
- [x] 4.3 Add weekly summary list, generate, regenerate, loading, empty, and error states.
- [x] 4.4 Add life chapter date-range generation, list/detail, loading, empty, and error states.
- [x] 4.5 Show qualitative observations without numeric personality-weight timelines.

## 5. Verification

- [x] 5.1 Run `cargo test` from `frontend/src-tauri`.
- [x] 5.2 Run `npm test -- --run` from `frontend`.
- [x] 5.3 Run `npm run build` from `frontend`.
- [x] 5.4 Run `openspec validate add-growth-review-suite --strict`.
- [x] 5.5 Perform a manual UI pass for diary review, weekly summaries, chapters, and observation history.
