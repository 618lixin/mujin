## ADDED Requirements

### Requirement: Growth artifact edit controls
The growth review UI SHALL let users edit, save, and cancel edits for generated weekly summaries and life chapters.

#### Scenario: Enter edit mode
- **WHEN** the user opens an existing weekly summary or life chapter and selects edit
- **THEN** the UI shows editable Markdown content and, for life chapters, an editable title field

#### Scenario: Save edited artifact
- **WHEN** the user saves a weekly summary or life chapter edit
- **THEN** the UI calls the matching update command, refreshes the selected artifact, and returns to view mode

#### Scenario: Cancel edited artifact
- **WHEN** the user cancels an edit
- **THEN** the UI discards unsaved local changes and restores the last saved artifact content

#### Scenario: Editing unavailable for empty selection
- **WHEN** no weekly summary or life chapter is selected
- **THEN** the UI SHALL NOT show an actionable save control
