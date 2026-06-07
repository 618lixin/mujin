## Why

Weekly summaries and life chapters are now generated, but the user still cannot directly correct, refine, or save those AI-written artifacts from the growth review surface. For a diary product whose core promise is low input cost plus high output value, AI output needs an edit loop so generated records become the user's own long-term archive rather than one-shot drafts.

## What Changes

- Add update commands for weekly summaries and life chapters so title/content edits can be saved without regenerating.
- Add growth review UI editing states for generated artifacts: view, edit, save, cancel, dirty-state handling, and error feedback.
- Preserve stable identities: weekly summaries remain keyed by ISO year/week, and life chapters remain keyed by note identity even when titles change.
- Keep stored metadata intact while allowing user-facing Markdown content and titles to change.
- Do not introduce numeric personality-weight history or AI-only edit claims.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `weekly-growth-summary`: Weekly summary notes can be edited and saved without changing ISO week identity or creating duplicates.
- `life-chapter`: Life chapter titles and content can be edited while preserving stable note identity and date-range metadata.
- `growth-review-ui`: The growth review surface exposes edit/save/cancel workflows for generated weekly summaries and life chapters.

## Impact

- Rust services: weekly summary and life chapter update functions, Tauri commands, tests for stable identity and metadata preservation.
- Frontend API: wrappers and types for update commands.
- React UI: editable detail panel states for weekly summaries and life chapters.
- NoteStore: uses existing note update behavior; no new dependency or database migration expected.
