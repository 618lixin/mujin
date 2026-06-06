## Why

The chat page currently behaves like one endless conversation, which makes it hard to browse prior days and increases the chance that old context feels like "today". The product needs a daily rhythm: each local date should have its own conversation while still preserving access to previous days.

## What Changes

- Add a dated conversation history area to the chat page.
- Automatically use a separate conversation for each local calendar date.
- Load today's conversation by default and create it implicitly on first message.
- Allow users to select prior dates and review that day's conversation.
- Keep historical conversations from being mixed into today's prompt context.
- Preserve access to the existing single `history.json` storage without treating it as today's conversation when its file date is older.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `companion-chat`: Chat history is organized by local date, exposed through history-list APIs, and shown in the chat page history area.

## Impact

- Rust chat memory storage changes from a single per-user history file to per-date history files.
- Tauri chat history commands gain date-aware behavior and a new daily history listing command.
- React chat API wrappers and `ChatPanel` gain daily history selection.
- Existing `history.json` remains readable under its local file-modified date; new writes use dated storage.
