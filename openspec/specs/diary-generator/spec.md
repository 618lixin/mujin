## Purpose
Define how the desktop app generates, stores, lists, and regenerates AI-assisted daily diary entries.

## Requirements

### Requirement: Daily diary generation
系统 SHALL 支持为指定日期生成日记，并在未指定日期时使用本地今天日期。

#### Scenario: Generate diary for date
- **WHEN** 前端调用 `ai_generate_diary` 并指定日期
- **THEN** 系统基于该日期的事件、对话、笔记和核心记忆调用 LLM 生成日记

#### Scenario: Generate diary without date
- **WHEN** 前端调用 `ai_generate_diary` 且未指定日期
- **THEN** 系统为本地今天生成日记

### Requirement: Diary source aggregation
日记生成 SHALL 聚合四路素材：当天事件、当天对话轮次、当天 Markdown 笔记和核心记忆。

#### Scenario: Diary source counts returned
- **WHEN** 日记生成完成
- **THEN** 返回 `source_event_count`、`source_turn_count`、`source_note_count`

#### Scenario: No source material
- **WHEN** 指定日期没有事件、对话或笔记
- **THEN** 系统仍 SHALL 生成空日记格式，而不是失败

### Requirement: Diary storage in notes
日记 SHALL 作为 Markdown 笔记存储在当前 NoteStore 中，而不是旧的 `data/{user_id}/diaries/YYYY-MM-DD.md` 独立目录。

#### Scenario: Diary note created
- **WHEN** 日记生成完成
- **THEN** 系统创建带日期标题和日记分类/标记的 Markdown 笔记，并返回 `note_id`

#### Scenario: Diary already exists
- **WHEN** 对应日期已有日记笔记
- **THEN** `ai_generate_diary` SHALL 重新聚合最新素材、重新生成内容，并替换该日期已有日记

### Requirement: Diary regeneration
系统 SHALL 支持重新生成指定日期日记并覆盖已有日记内容。

#### Scenario: Regenerate diary
- **WHEN** 前端调用 `ai_regenerate_diary`
- **THEN** 系统重新聚合素材、调用 LLM，并替换该日期已有日记内容

### Requirement: Diary grounded writing style
日记生成 SHALL 以当天对话摘要和用户随手笔记为主要依据，允许轻微整理和谨慎延伸，但不得编造未出现的情节。

#### Scenario: Sparse source material
- **WHEN** 当天素材很少或没有展开细节
- **THEN** 生成内容 SHALL 保持简短，并使用「记录里没有展开」「似乎」「可能」等不确定表达，而不是补全故事细节

#### Scenario: Rich source material
- **WHEN** 当天对话和笔记提供了明确事实
- **THEN** 生成内容 SHALL 基于这些事实组织日记，不添加未出现的人物、地点、动作、对白或戏剧化场景

### Requirement: Tauri diary commands
系统 SHALL 通过 Tauri command 提供日记查询和生成能力。

#### Scenario: List diaries
- **WHEN** 前端调用 `ai_get_diary_list`
- **THEN** 返回按时间排序的日记条目列表

#### Scenario: Get diary by date
- **WHEN** 前端调用 `ai_get_diary`
- **THEN** 返回对应日期的日记条目；不存在时返回 `None`

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

### Requirement: Diary generation avoids hallucinated events
Diary generation SHALL treat same-day chat summaries and same-day user notes as the only sources that can establish today's facts.

#### Scenario: Context-only sources
- **WHEN** only same-day events, core memory, or related past memories are available
- **THEN** the system SHALL NOT use them to invent concrete diary facts, causes, actions, dialogue, locations, decisions, outcomes, or relationship status

#### Scenario: Sparse direct material
- **WHEN** same-day chat summaries or notes contain only sparse material
- **THEN** the generated diary SHALL remain sparse and MUST NOT compensate by adding plausible details

#### Scenario: Low-creativity diary generation
- **WHEN** the system calls the LLM to generate a diary
- **THEN** it SHALL use a low-temperature generation setting suitable for factual rewriting rather than creative storytelling

### Requirement: Related memory recall tracking
After successful diary generation, the system SHALL record recall for event memories that were surfaced in the related-memory prompt section.

#### Scenario: Related event included in prompt
- **WHEN** a related memory with an event id is included in the diary prompt and diary generation succeeds
- **THEN** the system records a recall for that event

#### Scenario: Diary generation fails
- **WHEN** diary generation fails before saving
- **THEN** the system SHALL NOT record recall for related memories from that failed generation
