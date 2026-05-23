## ADDED Requirements

### Requirement: Personality weight snapshot
系统 SHALL 在每次 Reflection 执行后，将当前权重快照保存到 SQLite `personality_snapshots` 表。

#### Scenario: Save snapshot after reflection
- **WHEN** Reflection 完成并更新了基础权重
- **THEN** 系统将新权重、时间戳、反思摘要插入 personality_snapshots 表

### Requirement: Personality snapshot table schema
表结构 SHALL 包含 id、weights（JSON）、summary、created_at 字段。

#### Scenario: Table created on init
- **WHEN** 用户初始化
- **THEN** personality_snapshots 表自动创建

### Requirement: Personality history query
系统 SHALL 支持查询人格权重变化历史。

#### Scenario: GET personality history
- **WHEN** 调用 GET /api/personality/history
- **THEN** 返回所有权重快照列表，按时间降序

#### Scenario: GET personality history with limit
- **WHEN** 调用 GET /api/personality/history?limit=10
- **THEN** 返回最近 10 条快照
