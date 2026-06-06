## Purpose
Define the diary-specific related memory retrieval boundary used to connect daily diary generation with relevant past memories.

## Requirements

### Requirement: Diary related memory retrieval module
The system SHALL provide a diary-specific related memory retrieval module that returns past memories relevant to a target diary date.

#### Scenario: Retrieve related memories from same-day source material
- **WHEN** diary generation provides same-day events, conversation summaries, and notes to the retrieval module
- **THEN** the module returns a bounded list of past related memories with date, content, reason, score, and optional event id

#### Scenario: No related memories found
- **WHEN** no past memory is related to the diary source material
- **THEN** the module returns an empty list without failing diary generation

### Requirement: Exclude same-day memories
The retrieval module SHALL exclude memories whose creation date falls on the target diary date.

#### Scenario: Same-day event candidate
- **WHEN** an event occurred on the target diary date
- **THEN** it SHALL NOT appear in the related past memories list

### Requirement: Prefer structured relation signals
The retrieval module SHALL prefer topic-linked or otherwise structured relation signals over unfiltered recency.

#### Scenario: Topic-linked past event exists
- **WHEN** today's material and a past event share a topic link
- **THEN** the topic-linked past event ranks ahead of unrelated recent events

#### Scenario: Missing topic links
- **WHEN** no topic-linked candidates exist
- **THEN** the module MAY use keyword or conversation-summary fallback retrieval

### Requirement: Bounded retrieval result
The retrieval module SHALL cap related memories to a small configurable maximum.

#### Scenario: Too many candidates
- **WHEN** retrieval finds more candidates than the configured maximum
- **THEN** the module returns only the highest-ranked candidates

### Requirement: Stable retrieval boundary
Diary generation SHALL consume related memories through the retrieval module boundary, not through direct ad hoc SQL embedded in diary prompt construction.

#### Scenario: Retrieval backend changes
- **WHEN** the underlying memory backend is replaced
- **THEN** diary prompt construction SHALL continue to consume the same related-memory output shape
