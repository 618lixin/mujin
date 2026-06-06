## Purpose
Expose diary review, weekly summaries, life chapters, and qualitative growth records in one desktop growth review surface.

## Requirements

### Requirement: Growth review navigation
The React app SHALL provide a growth review surface reachable from the main desktop UI.

#### Scenario: Open growth review
- **WHEN** the user selects the growth review area
- **THEN** the app shows diary review, weekly summary, life chapter, and qualitative observation sections without requiring a separate web server

### Requirement: Diary review controls
The growth review UI SHALL expose existing diary list, detail, generate, and regenerate behavior.

#### Scenario: Review diary entry
- **WHEN** the user selects a diary entry
- **THEN** the UI shows the diary content and source counts when available

#### Scenario: Regenerate diary from review UI
- **WHEN** the user regenerates a diary from the review UI
- **THEN** the UI calls the existing diary regeneration command and refreshes the selected diary entry

### Requirement: Weekly summary controls
The growth review UI SHALL allow the user to list, generate, inspect, and regenerate weekly summaries.

#### Scenario: Generate weekly summary from UI
- **WHEN** the user selects a week and clicks generate
- **THEN** the UI calls the weekly summary command and displays loading, success, and error states

### Requirement: Life chapter controls
The growth review UI SHALL allow the user to generate and inspect life chapters for selected date ranges.

#### Scenario: Generate life chapter from UI
- **WHEN** the user selects a valid date range and clicks generate
- **THEN** the UI calls the life chapter command and displays the resulting chapter

### Requirement: Qualitative observation history
The growth review UI SHALL show qualitative growth observations without presenting removed numeric personality weights.

#### Scenario: Show observations
- **WHEN** qualitative observations exist
- **THEN** the UI lists observation text, category, and timestamp using the current observation data model
