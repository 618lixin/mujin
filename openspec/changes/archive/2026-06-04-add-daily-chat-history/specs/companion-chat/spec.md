## ADDED Requirements

### Requirement: Daily chat conversations
The system SHALL organize short-term chat history into one conversation per user per local calendar date.

#### Scenario: Today's conversation is selected by default
- **WHEN** the user opens the chat page
- **THEN** the system loads the conversation for the current local date

#### Scenario: First message creates today's conversation
- **WHEN** the user sends the first message on a local date with no existing chat history
- **THEN** the system creates that date's conversation automatically

#### Scenario: Prior days do not enter today's rolling prompt history
- **WHEN** the user sends a message today after chatting on a prior date
- **THEN** the rolling chat history injected into the prompt MUST only include today's conversation history

### Requirement: Chat history date list
The system SHALL expose a per-user list of dates that have chat history so the chat page can display a history area.

#### Scenario: History area lists dated conversations
- **WHEN** the user has chat history on multiple dates
- **THEN** the chat page shows those dates in reverse chronological order

#### Scenario: Selecting a prior date
- **WHEN** the user selects a prior date in the history area
- **THEN** the chat page displays that date's conversation without appending new messages to it

### Requirement: Legacy chat history compatibility
The system SHALL preserve access to the existing single-file chat history format during the transition to dated conversations without incorrectly treating older legacy history as today's conversation.

#### Scenario: Legacy history appears under its own date
- **WHEN** the old single `history.json` file exists
- **THEN** the system lists that legacy conversation under the file's local modified date

#### Scenario: New writes use dated storage
- **WHEN** a chat turn is saved after daily history support is enabled
- **THEN** the system writes the conversation to the dated history storage
