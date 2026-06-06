## ADDED Requirements

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

### Requirement: Weekly summary listing
The system SHALL expose weekly summary entries through Tauri commands.

#### Scenario: List weekly summaries
- **WHEN** the frontend requests weekly summaries
- **THEN** the system returns summary entries ordered by week descending
