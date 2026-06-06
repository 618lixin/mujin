## Context

The current implementation stores one rolling chat history per user in `history.json`. `ChatPanel` loads that single history on mount, and each new message appends to the same file through the Rust chat service. This conflicts with the diary product shape: users think in days, and diary generation should not make yesterday's conversation feel like today's input.

## Goals / Non-Goals

**Goals:**

- Store and retrieve chat history by local calendar date.
- Show a compact date-based history area on the chat page.
- Keep today's conversation active by default and create it automatically.
- Make previous dates reviewable without mixing them into today's prompt history.
- Preserve access to existing single-file history without letting it pollute today's new conversation.

**Non-Goals:**

- Multi-thread conversations within the same day.
- Manual renaming, pinning, deleting, or merging historical conversations.
- Changing long-term memory retrieval or diary memory retrieval behavior.

## Decisions

1. Store daily histories under a dated history directory.

   Use one JSON file per local date, for example `history/2026-06-03.json`, while keeping the existing message schema. This keeps the migration small and avoids a database dependency for short rolling prompt history.

   Alternative considered: move all chat history into SQLite. That would make listing/querying richer, but it expands the change into a data-model migration and duplicates the existing `conversation_turns` persistence path.

2. Use local calendar date as the conversation key.

   Chat history follows the user's local day because the product's diary and note workflows are day-based. The date key uses `YYYY-MM-DD`.

   Alternative considered: UTC dates. UTC would be easier to compute consistently but would surprise users around midnight and misalign with diary days.

3. Make historical days view-only in the chat page.

   The active input remains tied to today's conversation. Selecting an older date shows its messages but does not append new turns there, which avoids accidental backfilling and keeps diary inputs clear.

   Alternative considered: allow sending into any selected date. That creates ambiguous event dates and would need additional guardrails.

4. Keep legacy `history.json` readable under its own local file date.

   The backend lists and reads the old `history.json` under the file's local modified date. New writes go to the dated path for the current local date.

   Alternative considered: one-time eager migration or treating legacy history as today's initial history. Development data is not important enough to justify migration, and treating old data as today breaks the daily conversation boundary.

## Risks / Trade-offs

- Existing old history may represent multiple days, but the legacy file has no per-message timestamps. Mitigation: development data is not migrated; the file is exposed under its modified date and new writes become date-separated.
- The date list is file-backed, so very large numbers of daily files could be slower to scan. Mitigation: daily JSON files are small, and the list can be sorted from filenames without loading every full conversation.
- View-only history may surprise users who expect to continue an old day. Mitigation: the UI labels older days as historical and keeps today's entry easy to return to.

## Migration Plan

1. Add date-aware helpers while retaining existing `load_history`/`save_history` behavior for today's date.
2. Add commands for listing daily histories and reading/clearing a specific date.
3. Update the frontend to call date-aware APIs.
4. Verify legacy `history.json` appears under its own local modified date and does not populate today's new conversation unless that is also its modified date.
