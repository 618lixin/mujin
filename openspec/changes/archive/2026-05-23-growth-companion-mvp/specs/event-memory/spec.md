## ADDED Requirements

### Requirement: Event storage
系统 SHALL 将识别到的重要事件存入 SQLite 数据库（events 表），包含 id、content、emotions（JSON 数组）、importance（0.0~1.0）、event_type（conflict/milestone/emotion/decision）、created_at、updated_at 字段。

#### Scenario: Store a significant event
- **WHEN** 情绪识别模块检测到重要性 ≥0.6 的事件
- **THEN** 系统将事件写入 events 表，包含完整的元数据

#### Scenario: Store a low-importance event
- **WHEN** 情绪识别模块检测到重要性 <0.6 的事件
- **THEN** 系统 SHALL NOT 写入 events 表

### Requirement: Event retrieval
系统 SHALL 支持按重要性、事件类型、时间范围检索事件。

#### Scenario: Retrieve recent important events
- **WHEN** 请求最近 N 条重要性 >0.7 的事件
- **THEN** 返回按 created_at 降序排列的事件列表，最多 N 条

#### Scenario: Retrieve events by type
- **WHEN** 请求 event_type 为 "milestone" 的所有事件
- **THEN** 返回所有里程碑事件，按时间降序排列

#### Scenario: Retrieve events by date range
- **WHEN** 请求指定日期范围的事件
- **THEN** 返回该范围内的事件列表

### Requirement: Event memory API
系统 SHALL 提供 REST API 端点用于查询事件记忆。

#### Scenario: GET events with filters
- **WHEN** 调用 GET /api/memory/events 并提供 limit、min_importance、event_type、start_date、end_date 参数
- **THEN** 返回符合条件的事件列表

#### Scenario: DELETE single event
- **WHEN** 调用 DELETE /api/memory/events/{event_id}
- **THEN** 删除该事件并返回确认
