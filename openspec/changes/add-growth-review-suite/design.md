## Context

`growth-companion-mvp2` is a stale predecessor change. Its product goal still matters: the user should be able to look back across time, not just chat and generate one daily diary at a time. Its implementation assumptions no longer fit the codebase. The current app is Tauri 2 + Rust services + SQLite + NoteStore + React 19.

The current memory model is also qualitative. The old numeric personality-weight history must not return as a hidden dependency of the review experience.

## Goals / Non-Goals

**Goals:**
- Implement growth review as local-first desktop features using Tauri commands and Rust services.
- Reuse existing event memory, conversation summaries, notes, diary entries, topics, projects, growth lines, and observations.
- Keep generated summaries grounded in source material and visibly scoped by date range.
- Provide a frontend review surface that makes generated artifacts easy to inspect and regenerate.

**Non-Goals:**
- Do not add or depend on a Python/FastAPI runtime.
- Do not restore the retired single-HTML frontend.
- Do not restore numeric MBTI/eight-dimension personality weights.
- Do not make automatic background scheduling part of this change.
- Do not change daily diary anti-hallucination rules except where the review UI calls existing diary commands.

## Decisions

### D1: Store generated reviews in NoteStore

Weekly summaries and life chapters are generated narrative artifacts, closer to diary entries than structured facts. Store them as Markdown notes with metadata tags/categories so they can be listed, opened, edited, and exported with the rest of the user's notes.

Alternative: store generated Markdown directly in SQLite. This would make listing simple but would split authored/generated text across two storage models.

### D2: Keep retrieval modular

Create dedicated service modules for weekly summaries and life chapters instead of folding them into `diary.rs`. Each module should accept explicit query parameters, retrieve source material through existing database and NoteStore interfaces, build a constrained prompt, and return a typed result.

Alternative: add everything to `diary.rs`. That would be faster initially but would make the diary hallucination controls and long-range review logic harder to evolve independently.

### D3: Use qualitative observations only

Growth review may include `observations`, topics, projects, and growth lines, but it must not use old personality-weight snapshots. The review should describe patterns in natural language and cite source periods rather than graphing numeric personality dimensions.

Alternative: reintroduce historical weights for visualization. That conflicts with the current `personality-engine` spec and would make the memory model harder to refactor.

### D4: Manual generation first

Weekly summaries and life chapters should be manually generated or regenerated from the UI. Scheduling can be proposed later after storage, prompts, and review quality are stable.

Alternative: generate summaries automatically in a background task. That adds lifecycle and error-state complexity before the generation quality is proven.

## Risks / Trade-offs

**[Sparse source material]** -> The prompt and service result should surface source counts and allow short/empty summaries instead of filling gaps with plausible stories.

**[Prompt drift across diary, week, and chapter generation]** -> Keep prompt builders separate, test them directly, and share grounding rules only where they are truly common.

**[Large date ranges]** -> Cap the amount of source material included in a chapter prompt and prefer ranked summaries over raw transcript expansion.

**[UI scope creep]** -> Start with list, detail, generate, regenerate, and source-count states before adding charts or advanced filters.

## Migration Plan

1. Leave existing diary notes and memory tables unchanged.
2. Add new generated note categories/tags for weekly summaries and life chapters.
3. Expose new Tauri commands and frontend wrappers without removing existing diary commands.
4. Archive the stale `growth-companion-mvp2` change without syncing its old delta specs.

## Open Questions

- Should weekly summaries use ISO week numbers, local calendar week ranges, or both in the note metadata?
- Should life chapters require a user-provided title or generate one from the date range and source material?
