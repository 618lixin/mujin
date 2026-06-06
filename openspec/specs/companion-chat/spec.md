## Purpose
Define the current Tauri/Rust chat loop, including prompt assembly, streaming behavior, post-chat memory deposition, and daily chat history management.

## Requirements

### Requirement: Tauri chat commands
The system SHALL provide chat capability through Tauri commands rather than the old Python/FastAPI REST endpoint.

#### Scenario: Non-streaming chat turn
- **WHEN** the frontend calls `chat_send` with `user_id` and `message`
- **THEN** the system returns the AI reply, emotion extraction result, and current turn count

#### Scenario: Streaming chat turn
- **WHEN** the frontend calls `chat_stream_start` with `user_id` and `message`
- **THEN** the system returns a `stream_id` and emits streaming result events through `chat-token`, `chat-done`, and `chat-error`

### Requirement: Prompt assembly
Each chat prompt SHALL be assembled by the current Tauri/Rust implementation in this order: role definition, core memory, relevant historical memory, recent rolling chat history, and the current user message.

#### Scenario: Full prompt assembly
- **WHEN** the system builds a chat request
- **THEN** the system prompt includes the natural friend-style role definition, `user_profile.md`, `companion_notes.md`, and available relevant historical memory

#### Scenario: Retrieved memories available
- **WHEN** the current message matches historical conversations or recent important events
- **THEN** the system prompt includes a related historical memory section

#### Scenario: No retrieved memories
- **WHEN** no related historical memory is available
- **THEN** the system prompt still includes role definition and core memory, and continues the chat

### Requirement: Post-chat pipeline
After each completed chat turn, the system SHALL execute the implemented post-chat pipeline: save rolling history, update turn counter, extract emotion/events, save `conversation_turns`, save important events, link topics, and save last activity.

#### Scenario: Significant event saved
- **WHEN** emotion extraction returns `importance >= 0.6` and an `event_type`
- **THEN** the system writes an event memory and attempts to link extracted topics

#### Scenario: Routine conversation
- **WHEN** emotion extraction does not reach the important event threshold
- **THEN** the system saves the conversation turn and rolling history but does not write an event memory

#### Scenario: Reflection not yet implemented
- **WHEN** the post-chat pipeline completes
- **THEN** `reflection` and `notes_update` currently return `None` until Reflection/Notes automatic update is implemented

### Requirement: Conversation history management
The system SHALL retain the most recent `max_history_turns` turns of the current day's rolling conversation for prompt assembly. Earlier content SHALL rely on deposited `conversation_turns`, event memory, and core memory retrieval.

#### Scenario: History within limit
- **WHEN** the current day's conversation history has not exceeded `max_history_turns`
- **THEN** the full current-day rolling history is injected into the prompt

#### Scenario: History exceeds limit
- **WHEN** the current day's conversation history exceeds `max_history_turns`
- **THEN** only the most recent history is retained for prompt assembly, while earlier content relies on deposited memory retrieval

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
