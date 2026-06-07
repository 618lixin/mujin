## Context

The growth review suite currently generates and lists diary reviews, weekly summaries, life chapters, and qualitative records. Weekly summaries and life chapters are stored as NoteStore Markdown notes, but the growth review UI only supports viewing and regeneration. This makes AI output feel like a final answer instead of a draft the user can correct and keep.

The existing NoteStore already supports stable note ids and title/content updates. Weekly summaries additionally have a semantic identity based on ISO year/week. Life chapters carry date-range metadata in stored Markdown front matter while returning clean Markdown content to the UI.

## Goals / Non-Goals

**Goals:**

- Let users edit and save generated weekly summary content from the growth review surface.
- Let users edit and save life chapter title and content from the growth review surface.
- Preserve weekly ISO identity, life chapter note identity, and life chapter date-range metadata across edits.
- Use existing NoteStore update behavior instead of introducing a new persistence layer.
- Cover edit behavior with focused Rust and frontend API tests.

**Non-Goals:**

- Rich text editing, collaborative editing, autosave, version history, or diff review.
- Editing diary generation prompts or raw source records.
- Changing the generation prompts beyond what is required to keep edited artifacts stable.
- Adding numeric personality-weight history.

## Decisions

1. **Use explicit update commands instead of overloading regenerate.**
   - Decision: Add separate Tauri commands for editing weekly summaries and life chapters.
   - Rationale: Regeneration means "ask AI again"; editing means "save user-authored changes." Keeping them separate avoids accidental AI overwrite and clearer UI states.
   - Alternative considered: Reuse existing regenerate commands with optional content/title fields. Rejected because it conflates two different workflows.

2. **Keep weekly summary title and identity tied to ISO year/week.**
   - Decision: Weekly summary edit saves content only, leaving `week-YYYY-WNN` title and existing note id unchanged.
   - Rationale: The prior decision made ISO week the stable identity. Allowing arbitrary weekly titles would complicate lookup and regeneration for little value.
   - Alternative considered: Editable weekly titles. Rejected for this phase to keep identity unambiguous.

3. **Allow life chapter title edits while preserving note id and metadata.**
   - Decision: Life chapter edit accepts title and content, updates the existing note, and reattaches existing start/end date metadata.
   - Rationale: The PRD explicitly expects chapter titles to be AI-generated but user-editable. Note id already decouples identity from mutable title.
   - Alternative considered: Store chapter metadata in a sidecar table. Rejected because NoteStore metadata/front matter is already sufficient for this local single-user phase.

4. **Use a lightweight Markdown textarea editor in the growth review panel.**
   - Decision: Add view/edit mode with a textarea for Markdown, save/cancel buttons, dirty-state handling, and error feedback.
   - Rationale: This is enough to close the edit loop without pulling in a rich editor dependency or changing the current Markdown preview model.
   - Alternative considered: Integrate a full Markdown editor. Rejected as unnecessary surface area for the first edit pass.

## Risks / Trade-offs

- [Risk] User edits can be overwritten by regeneration. -> Mitigation: Keep regenerate as an explicit separate action and refresh detail state after save/regenerate.
- [Risk] Life chapter date metadata can be lost during content edits. -> Mitigation: update service parses existing range metadata and re-wraps edited content before saving.
- [Risk] Weekly summary edit could target the wrong note if title changes. -> Mitigation: weekly edit locates by ISO year/week identity and updates only that matching note.
- [Risk] Unsaved edits can be lost when switching selection. -> Mitigation: show dirty state and require explicit cancel/save before overwriting editor buffers where practical.

## Migration Plan

No data migration is expected. Existing weekly summary and life chapter notes remain valid. New edit commands operate on existing NoteStore records.

Rollback is straightforward: remove the edit commands and UI edit controls. Existing edited notes remain Markdown notes and can still be listed/viewed.

## Open Questions

- Should weekly summaries eventually allow a user-facing display title separate from ISO identity?
- Should edited artifacts record an `edited_at` or `edited_by_user` marker in metadata for future audit/version history?
