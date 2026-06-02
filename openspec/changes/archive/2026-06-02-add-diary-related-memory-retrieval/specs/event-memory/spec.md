## ADDED Requirements

### Requirement: Past event lookup for related diary memory
Event memory SHALL expose retrieval primitives that can find past events related to diary source material while excluding the target diary date.

#### Scenario: Query past events by topic
- **WHEN** diary memory retrieval asks for events linked to a set of topics before the diary date
- **THEN** event memory returns matching past events ordered by relevance inputs

#### Scenario: Query past events by keyword
- **WHEN** diary memory retrieval asks for events matching derived keywords before the diary date
- **THEN** event memory returns matching past events without including same-day events

### Requirement: Conversation summary lookup for related diary memory
Event memory SHALL expose retrieval primitives for past conversation summaries relevant to diary source material.

#### Scenario: Query past conversation summaries
- **WHEN** diary memory retrieval asks for conversation summaries matching derived keywords before the diary date
- **THEN** event memory returns matching past summaries without including same-day turns

### Requirement: Recall can be recorded from diary generation
Event memory SHALL allow diary generation to record recall for surfaced past event memories.

#### Scenario: Record diary recall
- **WHEN** diary generation reports that a past event was surfaced to the prompt
- **THEN** event memory updates that event's recall count, last recalled timestamp, and stability according to the existing recall mechanism
