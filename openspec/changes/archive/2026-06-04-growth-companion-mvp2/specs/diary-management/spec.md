## ADDED Requirements

### Requirement: Diary list query
系统 SHALL 支持查询日记列表，返回所有已生成日记的日期和摘要。

#### Scenario: GET diary list
- **WHEN** 调用 GET /api/diary
- **THEN** 返回所有已生成日记的列表，每条包含 date 和 content 前 100 字符

#### Scenario: GET diary list with date range
- **WHEN** 调用 GET /api/diary?start_date=2026-05-01&end_date=2026-05-23
- **THEN** 返回该日期范围内的日记列表

### Requirement: Diary regeneration
系统 SHALL 支持重新生成指定日期的日记，覆盖已有文件。

#### Scenario: POST regenerate diary
- **WHEN** 调用 POST /api/diary/{date}/regenerate
- **THEN** 系统重新生成该日期的日记并覆盖原文件

#### Scenario: Regenerate with no data
- **WHEN** 该日期无累积数据
- **THEN** 返回 404 提示"该日期无数据可生成"

### Requirement: Diary batch generation
系统 SHALL 支持批量生成指定日期范围内缺失的日记。

#### Scenario: POST batch generate
- **WHEN** 调用 POST /api/diary/batch-generate 并提供 start_date 和 end_date
- **THEN** 系统为范围内有数据但未生成日记的日期逐一生成日记，返回生成结果列表
