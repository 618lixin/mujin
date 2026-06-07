## Purpose
Provide grounded weekly growth summaries that turn local diary and memory records into a reviewable Markdown note while keeping ISO week identity stable.

## Requirements

### Requirement: Weekly growth summary generation
The system SHALL generate a weekly growth summary for a specified local calendar week using source material from that week.

#### Scenario: Generate weekly summary with sources
- **WHEN** the frontend requests a weekly summary for a week that has diary entries, event memories, conversation summaries, notes, or qualitative observations
- **THEN** the system generates a grounded Markdown summary and returns source counts for each source type

#### Scenario: Generate weekly summary with sparse sources
- **WHEN** the requested week has little or no source material
- **THEN** the system returns a short summary or empty-state result without inventing unsupported events

### Requirement: Weekly summary storage
The system SHALL store weekly summaries as Markdown notes in the current NoteStore.

#### Scenario: Weekly summary note created
- **WHEN** weekly summary generation succeeds for a week with no existing summary note
- **THEN** the system creates a Markdown note tagged or categorized as a weekly growth summary

#### Scenario: Weekly summary regenerated
- **WHEN** the frontend requests regeneration for a week with an existing weekly summary note
- **THEN** the system replaces the existing weekly summary content instead of creating a duplicate summary for the same week

### Requirement: Weekly summary identity and display range
The system SHALL use ISO year/week as the stable identity for weekly summaries and SHALL expose a human-readable date range for display.

#### Scenario: Weekly summary has stable ISO identity
- **WHEN** the system stores or regenerates a weekly summary
- **THEN** the note identity, filename, and command parameters use ISO year/week rather than a local calendar week number

#### Scenario: Weekly summary display range shown
- **WHEN** the frontend lists or opens a weekly summary
- **THEN** the response includes a `week_display_range` formatted from ISO dates for user-facing display

### Requirement: Weekly summary listing
The system SHALL expose weekly summary entries through Tauri commands.

#### Scenario: List weekly summaries
- **WHEN** the frontend requests weekly summaries
- **THEN** the system returns summary entries ordered by week descending

### Requirement: Weekly summary editing
The system SHALL allow users to save manual content edits to an existing weekly summary without changing its ISO week identity.

#### Scenario: Save weekly summary edit
- **WHEN** the frontend saves edited Markdown content for an existing ISO year/week summary
- **THEN** the system updates the existing weekly summary note and returns the same ISO year/week identity and note id

#### Scenario: Edit missing weekly summary
- **WHEN** the frontend saves an edit for an ISO year/week that has no weekly summary note
- **THEN** the system rejects the request with a not-found error instead of creating an unrelated note

#### Scenario: Weekly edit preserves identity
- **WHEN** a weekly summary is edited
- **THEN** the note title, filename identity, and subsequent regenerate lookup remain based on `week-YYYY-WNN`
