## Purpose
Define structured long-term event memory, conversation search, forgetting behavior, and topic links.

## Requirements

### Requirement: Event storage
系统 SHALL 将识别到的重要事件存入 SQLite `events` 表，并保存遗忘曲线字段。

#### Scenario: Store a significant event
- **WHEN** 情绪抽取检测到 `importance >= 0.6` 且存在 `event_type`
- **THEN** 系统写入事件，包含 content、emotions、importance、event_type、strength、stability、recall_count、timestamps

#### Scenario: Store a low-importance event
- **WHEN** 情绪抽取检测到 `importance < 0.6`
- **THEN** 系统 SHALL NOT 写入 `events` 表

### Requirement: Forgetting curve
事件记忆 SHALL 使用 `strength` 和 `stability` 表示随时间衰减的记忆强度。

#### Scenario: New event initialized
- **WHEN** 新事件写入
- **THEN** `strength` 初始为 1.0，`stability` 根据基础稳定性和 importance 计算

#### Scenario: Memory maintenance
- **WHEN** 执行 `ai_maintain_memory` 或心跳维护
- **THEN** 系统衰减事件强度，并清理低于 `forget_min_strength` 的事件

#### Scenario: Event recalled
- **WHEN** 系统调用 `record_recall`
- **THEN** 增加 recall_count、更新 last_recalled_at，并提升 stability

### Requirement: Event retrieval
系统 SHALL 支持按重要性、事件类型、时间范围和最低 strength 检索事件。

#### Scenario: Retrieve recent important events
- **WHEN** 查询最近 N 条重要事件
- **THEN** 返回按时间降序排列的事件列表，最多 N 条

#### Scenario: Retrieve events by date range
- **WHEN** 请求指定 UTC 时间范围内的事件
- **THEN** 返回该范围内的事件列表

### Requirement: Conversation turn storage and search
系统 SHALL 保存对话轮次摘要、情绪和时间，并提供 FTS5 与中文兼容 LIKE 搜索。

#### Scenario: Save conversation turn
- **WHEN** 对话后处理完成
- **THEN** 系统写入 `conversation_turns`，并同步写入 FTS5 `turn_search`

#### Scenario: Search conversations with FTS5
- **WHEN** 前端调用 `ai_search_conversations`
- **THEN** 系统使用 FTS5 MATCH 返回相关对话摘要

#### Scenario: Search Chinese conversations for prompt retrieval
- **WHEN** chat prompt 需要检索中文相关记忆
- **THEN** 系统 MAY 使用 `search_conversations_like` 对 summary 和 user_msg 做 LIKE 搜索

### Requirement: Topic linking
系统 SHALL 支持 topics 和 topic_links 表，用于将事件与主题关联。

#### Scenario: Topic extracted from event
- **WHEN** post-chat pipeline 得到 topics
- **THEN** 系统创建或更新 topic，并将 topic 与 event 建立 link

### Requirement: Tauri memory commands
系统 SHALL 通过 Tauri command 查询和维护事件记忆。

#### Scenario: Get events with filters
- **WHEN** 前端调用 `ai_get_events`
- **THEN** 返回符合 QueryEventsParams 的事件列表

#### Scenario: Delete single event
- **WHEN** 前端调用 `ai_delete_event`
- **THEN** 删除该事件并返回是否成功

### Requirement: Past event lookup for related diary memory
Event memory SHALL expose retrieval primitives that can find past events related to diary source material while excluding the target diary date.

#### Scenario: Query past events by topic
- **WHEN** diary memory retrieval asks for events linked to a set of topics before the diary date
- **THEN** event memory returns matching past events ordered by relevance inputs

#### Scenario: Query past events by keyword
- **WHEN** diary memory retrieval asks for events matching derived keywords before the diary date
- **THEN** event memory returns matching past events without including same-day events

### Requirement: Conversation summary lookup for related diary memory
Event memory SHALL expose retrieval primitives for past conversation summaries relevant to diary source material.

#### Scenario: Query past conversation summaries
- **WHEN** diary memory retrieval asks for conversation summaries matching derived keywords before the diary date
- **THEN** event memory returns matching past summaries without including same-day turns

### Requirement: Recall can be recorded from diary generation
Event memory SHALL allow diary generation to record recall for surfaced past event memories.

#### Scenario: Record diary recall
- **WHEN** diary generation reports that a past event was surfaced to the prompt
- **THEN** event memory updates that event's recall count, last recalled timestamp, and stability according to the existing recall mechanism
