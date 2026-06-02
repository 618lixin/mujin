## ADDED Requirements

### Requirement: Diary prompt includes optional related past memories
Diary generation SHALL include an optional related past memories section when the retrieval module returns relevant memories.

#### Scenario: Related memories available
- **WHEN** related past memories are found for the diary date
- **THEN** the diary prompt contains a distinct related memories section after same-day material

#### Scenario: No related memories available
- **WHEN** related memory retrieval returns an empty list
- **THEN** diary generation proceeds using same-day material and core memory only

### Requirement: Diary uses related memories conservatively
The diary prompt SHALL instruct the LLM to use past related memories only when the connection is natural and supported by the provided material.

#### Scenario: Weak relationship
- **WHEN** a related memory has only a weak or uncertain connection to today's material
- **THEN** the generated diary SHALL NOT be required to mention it

#### Scenario: Strong relationship
- **WHEN** a past memory clearly continues today's event, topic, or emotional thread
- **THEN** the generated diary MAY connect today with that past memory in natural diary prose

### Requirement: Related memory recall tracking
After successful diary generation, the system SHALL record recall for event memories that were surfaced in the related-memory prompt section.

#### Scenario: Related event included in prompt
- **WHEN** a related memory with an event id is included in the diary prompt and diary generation succeeds
- **THEN** the system records a recall for that event

#### Scenario: Diary generation fails
- **WHEN** diary generation fails before saving
- **THEN** the system SHALL NOT record recall for related memories from that failed generation
