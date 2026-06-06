## ADDED Requirements

### Requirement: Weekly summary generation
系统 SHALL 支持为指定周生成成长总结，汇总该周的事件记忆和日记数据，调用 LLM 生成结构化周报。

#### Scenario: Generate summary for a week with events
- **WHEN** 调用周报生成接口并指定周编号（如 2026-W21）
- **THEN** 系统查询该周的事件记忆和日记，调用 LLM 生成包含情绪变化、重要事件、成长观察的周报

#### Scenario: Generate summary for a week with few events
- **WHEN** 该周事件记忆少于 3 条
- **THEN** 系统在周报中注明"本周对话较少，总结可能不完整"

### Requirement: Weekly summary storage
周报 SHALL 存储为 `data/{user_id}/summaries/week-YYYY-WNN.md` 文件。

#### Scenario: Summary file created
- **WHEN** 周报生成完成
- **THEN** 对应的 Markdown 文件存在

#### Scenario: Summary already exists
- **WHEN** 该周的周报文件已存在
- **THEN** 系统 SHALL 返回冲突提示，不覆盖

### Requirement: Weekly summary API
系统 SHALL 提供 REST API 端点管理周报。

#### Scenario: POST generate weekly summary
- **WHEN** 调用 POST /api/summary/weekly 并指定 year 和 week
- **THEN** 生成对应周的周报

#### Scenario: GET weekly summary
- **WHEN** 调用 GET /api/summary/weekly/{year}/{week}
- **THEN** 返回对应周的周报内容

#### Scenario: GET list weekly summaries
- **WHEN** 调用 GET /api/summary/weekly
- **THEN** 返回所有已生成周报的列表（年份、周编号、标题）
