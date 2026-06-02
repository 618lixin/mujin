## Context

The current memory system is implemented in the Tauri/Rust backend:

- `services/memory.rs` manages core memory files (`user_profile.md`, `companion_notes.md`) and recent chat history.
- `services/database.rs` manages SQLite tables for events, conversation turns, observations, topics, projects, growth lines, and FTS5 search.
- `services/chat.rs` saves conversation turns, extracts emotions/events, writes important events, links topics, and injects related memories into chat prompts.
- `services/diary.rs` generates diary notes by aggregating same-day events, same-day conversation turns, same-day notes, and core memory.

The current diary generator does not retrieve past related events. It can describe "what happened today", but it cannot naturally connect today's material to prior episodes.

Because the memory system may be refactored later, diary generation should not directly depend on one retrieval implementation such as SQLite LIKE queries. The change should introduce a small retrieval module boundary that can be kept stable while storage and scoring evolve.

## Goals / Non-Goals

**Goals:**

- Add diary-specific related memory retrieval for past events and conversation summaries.
- Keep diary generation dependent on an interface-like module boundary, not on ad hoc SQL calls scattered through `diary.rs`.
- Support the current implementation using SQLite, topic links, event metadata, and keyword fallback.
- Exclude same-day events from "past related memories" to avoid duplication with today's source material.
- Format retrieved memories as optional context that the LLM may use only when naturally relevant.
- Preserve existing diary Tauri command signatures.
- Add tests around retrieval selection, prompt inclusion, and same-day exclusion.

**Non-Goals:**

- No embedding/vector database dependency in this change.
- No full rewrite of the memory database schema.
- No automatic project/growth-line generation.
- No guarantee that every diary mentions past events; the diary should mention them only when supported by the retrieved context.
- No UI change unless needed to expose source counts in the future.

## Decisions

### Decision 1: Introduce a diary memory retrieval module

Create a dedicated module, for example `services/diary_memory.rs`, with a public function similar to:

```rust
pub struct DiaryMemoryQuery<'a> {
    pub user_id: &'a str,
    pub diary_date: &'a str,
    pub day_events: &'a [Event],
    pub day_turns: &'a [ConversationTurn],
    pub day_notes: &'a [(String, String)],
    pub max_results: usize,
}

pub struct RelatedDiaryMemory {
    pub event_id: Option<String>,
    pub date: String,
    pub content: String,
    pub reason: String,
    pub score: f64,
}

pub fn retrieve_related_diary_memories(
    db: &DbState,
    query: &DiaryMemoryQuery,
) -> Result<Vec<RelatedDiaryMemory>, AppError>;
```

Rationale: `diary.rs` should ask for related memories without knowing whether they came from topic links, LIKE search, FTS5, embeddings, or another future backend.

Alternatives considered:

- Put retrieval directly in `diary.rs`: simpler now, but makes future memory refactors harder.
- Reuse `chat.rs::retrieve_memories`: inappropriate because chat retrieval uses the current user message as the query, while diary retrieval needs to infer anchors from multiple same-day sources.

### Decision 2: Use a layered retrieval strategy

The initial implementation should use current data structures in this order:

1. Topic-linked events: use topics attached to today's events and find earlier events linked to the same topics.
2. Keyword fallback: derive compact anchors from today's event contents, conversation summaries, and note titles; search past event content and conversation summaries.
3. Recent important context: optionally include a very small number of recent high-importance events only if no stronger topic/keyword matches exist.

Rationale: topic links provide the best available structure today. Keyword fallback gives recall when topics are missing. Unfiltered recent events should not dominate because they can make diaries feel randomly connected.

### Decision 3: Score and cap aggressively

Score candidates using a simple weighted formula:

- topic match: high weight
- keyword match: medium weight
- importance: medium weight
- strength: medium weight
- recency: low-to-medium weight
- same-day candidate: excluded

Return at most 3-8 related memories by default.

Rationale: diary prompts should receive a small set of meaningful memories. Too many past events will make the diary overfit history and feel forced.

### Decision 4: Keep prompt wording conservative

Add a section such as:

```text
--- 可能相关的过往记忆 ---
这些记忆可能与今天有关。只有当联系自然、能被材料支持时才写进日记；不要硬凑因果。
```

Rationale: the goal is a human-like diary, not a mechanical "memory citation" list. The LLM should be allowed to ignore weak connections.

### Decision 5: Apply recall tracking only for surfaced event memories

When a past event is selected and included in the diary prompt, call `record_recall` for that event after successful diary generation.

Rationale: the system's forgetting curve says recalled memories become more stable. This should apply when a memory is actually surfaced to generation, not merely when it appears somewhere in the database.

## Risks / Trade-offs

- Weak keyword extraction can produce irrelevant memories -> cap results, prefer topic links, and include conservative prompt instructions.
- Topic links are currently sparse -> keep keyword fallback and tests for missing-topic cases.
- Related memories may make diaries too analytical -> prompt explicitly says to use them only when natural.
- Recall tracking may over-stabilize noisy matches -> only record recall after successful diary generation and only for included event IDs.
- Future memory refactor may change storage types -> keep `RelatedDiaryMemory` small and avoid leaking SQL row shapes into diary generation.

## Migration Plan

1. Add the retrieval module and tests while leaving existing diary behavior unchanged when no related memories are found.
2. Update diary prompt construction to accept an optional related-memory block.
3. Add database helper queries only as private/current-backend support functions.
4. Run existing frontend/Rust tests.
5. Later memory refactors can replace the internals of `retrieve_related_diary_memories` without changing diary generation.

Rollback is straightforward: remove the optional related-memory call from `diary.rs`; same-day diary generation remains intact.

## Open Questions

- Should the diary generation result expose a `sourceRelatedMemoryCount` metric in the UI?
- Should related memory retrieval include past diary entries, or only raw event/conversation memory? Initial design excludes past diaries to avoid self-referential style drift.
- Should note content participate in keyword extraction beyond titles and short snippets? Initial design uses short snippets only to keep retrieval cheap.
