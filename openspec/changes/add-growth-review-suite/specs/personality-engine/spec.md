## ADDED Requirements

### Requirement: Growth review avoids numeric personality history
Growth review features SHALL use qualitative observations, topics, projects, and growth lines without reintroducing numeric personality-weight snapshots.

#### Scenario: Render growth review
- **WHEN** the frontend displays growth history or review material
- **THEN** the system shows qualitative records and SHALL NOT show Ti/Te/Fi/Fe/Si/Se/Ni/Ne weight timelines

#### Scenario: Generate review prompt
- **WHEN** weekly summary or life chapter generation builds a prompt
- **THEN** the prompt may include qualitative observations but SHALL NOT include removed numeric personality weights
