## ADDED Requirements

### Requirement: Life chapter generation
The system SHALL generate a life chapter for a user-selected date range using available local records from that range.

#### Scenario: Generate chapter for date range
- **WHEN** the frontend requests a life chapter with a valid start date and end date
- **THEN** the system gathers diaries, weekly summaries when available, event memories, notes, topics, projects, growth lines, and qualitative observations from that date range

#### Scenario: Invalid chapter range
- **WHEN** the requested end date is before the start date
- **THEN** the system rejects the request with a validation error

### Requirement: Life chapter grounding
The system SHALL keep generated chapters grounded in retrieved source material.

#### Scenario: Sparse chapter sources
- **WHEN** the selected date range has little source material
- **THEN** the generated chapter remains brief and explicitly avoids filling in missing causes, dialogue, decisions, outcomes, or timelines

#### Scenario: Long chapter range
- **WHEN** the selected date range contains more source material than the prompt budget can include
- **THEN** the system ranks or summarizes source material before prompting instead of injecting unlimited raw records

### Requirement: Life chapter storage and listing
The system SHALL store and list life chapters as Markdown notes in the current NoteStore.

#### Scenario: Chapter note created
- **WHEN** life chapter generation succeeds
- **THEN** the system creates a Markdown note tagged or categorized as a life chapter with date-range metadata

#### Scenario: List chapters
- **WHEN** the frontend requests existing life chapters
- **THEN** the system returns chapter entries ordered by date range or creation time descending
