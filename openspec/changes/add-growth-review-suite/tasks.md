## 1. Weekly Growth Summary Backend

- [ ] 1.1 Add failing Rust tests for weekly summary source aggregation, sparse-source handling, and same-week regeneration.
- [ ] 1.2 Implement weekly summary request/result types in the Rust service type layer.
- [ ] 1.3 Implement a `weekly_summary` service module that gathers diaries, events, turns, notes, and observations for a local week.
- [ ] 1.4 Add a grounded weekly summary prompt builder with tests for source counts and anti-hallucination wording.
- [ ] 1.5 Store generated weekly summaries as NoteStore Markdown notes and replace the existing week note on regeneration.

## 2. Life Chapter Backend

- [ ] 2.1 Add failing Rust tests for chapter date-range validation, sparse-source handling, and long-range source limiting.
- [ ] 2.2 Implement life chapter request/result types in the Rust service type layer.
- [ ] 2.3 Implement a `life_chapter` service module that gathers date-range sources from NoteStore and SQLite.
- [ ] 2.4 Add a grounded chapter prompt builder with tests for missing-detail constraints.
- [ ] 2.5 Store generated chapters as NoteStore Markdown notes with date-range metadata.

## 3. Tauri Commands And Frontend API

- [ ] 3.1 Add Tauri commands for listing, generating, and regenerating weekly summaries.
- [ ] 3.2 Add Tauri commands for listing and generating life chapters.
- [ ] 3.3 Add frontend API wrappers and tests for weekly summary commands.
- [ ] 3.4 Add frontend API wrappers and tests for life chapter commands.

## 4. Growth Review UI

- [ ] 4.1 Add or update the growth review panel entry in the main React UI.
- [ ] 4.2 Expose diary list, detail, generate, and regenerate controls through the review panel.
- [ ] 4.3 Add weekly summary list, generate, regenerate, loading, empty, and error states.
- [ ] 4.4 Add life chapter date-range generation, list/detail, loading, empty, and error states.
- [ ] 4.5 Show qualitative observations without numeric personality-weight timelines.

## 5. Verification

- [ ] 5.1 Run `cargo test` from `frontend/src-tauri`.
- [ ] 5.2 Run `npm test -- --run` from `frontend`.
- [ ] 5.3 Run `npm run build` from `frontend`.
- [ ] 5.4 Run `openspec validate add-growth-review-suite --strict`.
- [ ] 5.5 Perform a manual UI pass for diary review, weekly summaries, chapters, and observation history.
