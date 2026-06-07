## ADDED Requirements

### Requirement: Life chapter editing
The system SHALL allow users to save manual title and Markdown content edits to an existing life chapter.

#### Scenario: Save life chapter edit
- **WHEN** the frontend saves an edited title or Markdown body for an existing life chapter note id
- **THEN** the system updates that life chapter and returns the same note id with the new title and content

#### Scenario: Life chapter edit preserves metadata
- **WHEN** a life chapter with date-range metadata is edited
- **THEN** the system preserves the existing start date and end date metadata in storage and returns clean user-facing Markdown content

#### Scenario: Edit missing life chapter
- **WHEN** the frontend saves an edit for a missing or non-life-chapter note id
- **THEN** the system rejects the request instead of creating a new chapter silently
