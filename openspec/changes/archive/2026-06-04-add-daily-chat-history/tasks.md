## 1. Backend Storage

- [x] 1.1 Add date-aware chat history path helpers and date validation.
- [x] 1.2 Add tests for per-date isolation and legacy `history.json` file-date compatibility.
- [x] 1.3 Update chat load/save flow to use today's dated history.

## 2. Backend Commands

- [x] 2.1 Add a Tauri command that lists chat history dates with message counts.
- [x] 2.2 Make history read and clear commands accept an optional date.
- [x] 2.3 Add or update command-level tests where practical.

## 3. Frontend API

- [x] 3.1 Add typed API wrappers for listing chat days and loading a selected date.
- [x] 3.2 Update existing chat API tests for the new command payloads.

## 4. Chat Page UI

- [x] 4.1 Add a history area to `ChatPanel` with reverse-chronological daily conversations.
- [x] 4.2 Load today's conversation by default and refresh the day list after sending.
- [x] 4.3 Make historical dates review-only and keep message sending tied to today's conversation.

## 5. Verification

- [x] 5.1 Run Rust tests.
- [x] 5.2 Run frontend tests.
- [x] 5.3 Validate the OpenSpec change.
