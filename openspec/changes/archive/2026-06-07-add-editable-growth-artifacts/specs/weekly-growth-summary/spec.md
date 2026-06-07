## ADDED Requirements

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
