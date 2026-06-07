## 1. Weekly Summary Editing Backend

- [x] 1.1 Add failing Rust tests for saving weekly summary edits, preserving note id, preserving ISO title, and rejecting missing weeks.
- [x] 1.2 Add weekly summary edit request/result types if the existing result shape is not sufficient.
- [x] 1.3 Implement a weekly summary update service function that locates the existing note by ISO year/week and updates content only.
- [x] 1.4 Add a Tauri command for saving weekly summary edits and register it in the invoke handler.

## 2. Life Chapter Editing Backend

- [x] 2.1 Add failing Rust tests for saving life chapter title/content edits, preserving note id, preserving date metadata, and rejecting non-chapter notes.
- [x] 2.2 Add life chapter edit request/result types if the existing result shape is not sufficient.
- [x] 2.3 Implement a life chapter update service function that re-wraps edited content with existing date-range metadata before saving.
- [x] 2.4 Add a Tauri command for saving life chapter edits and register it in the invoke handler.

## 3. Frontend API

- [x] 3.1 Add frontend API wrappers for weekly summary and life chapter update commands.
- [x] 3.2 Add frontend API tests that verify command names and payload shapes for both update wrappers.
- [x] 3.3 Update shared TypeScript types for edit requests/results as needed.

## 4. Growth Review UI

- [x] 4.1 Add edit/view mode state for the detail panel without enabling save when no weekly summary or life chapter is selected.
- [x] 4.2 Add Markdown content editing for weekly summaries with save, cancel, loading, and error states.
- [x] 4.3 Add title and Markdown content editing for life chapters with save, cancel, loading, and error states.
- [x] 4.4 Refresh selected artifact state after save and preserve view mode after successful save.
- [x] 4.5 Prevent stale edit buffers when switching tabs or selecting another artifact.

## 5. Verification

- [x] 5.1 Run `cargo test` from `frontend/src-tauri`.
- [x] 5.2 Run `npm test -- --run` from `frontend`.
- [x] 5.3 Run `npm run build` from `frontend`.
- [x] 5.4 Run `npm run lint` from `frontend`.
- [x] 5.5 Run `openspec validate add-editable-growth-artifacts --strict`.
