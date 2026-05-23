## ADDED Requirements

### Requirement: Life chapter generation
系统 SHALL 支持为指定时间段生成人生章节叙事，汇总该时间段内的事件、日记和周报，调用 LLM 生成有标题的叙事文本。

#### Scenario: Generate chapter for a date range
- **WHEN** 调用人生章节生成接口并指定 start_date 和 end_date
- **THEN** 系统查询该时间段内的事件、日记和周报，调用 LLM 生成包含标题、摘要、关键事件列表、成长叙述的章节

#### Scenario: Generate chapter with custom title
- **WHEN** 调用时提供了 title 参数
- **THEN** 使用提供的标题作为章节标题

#### Scenario: No data in date range
- **WHEN** 指定时间段内没有任何事件或日记
- **THEN** 系统 SHALL 返回提示"该时间段内无数据"

### Requirement: Life chapter storage
章节 SHALL 存储为 `data/{user_id}/chapters/` 目录下的 Markdown 文件，文件名为时间戳或标题。

#### Scenario: Chapter file created
- **WHEN** 章节生成完成
- **THEN** 对应文件存在于 chapters 目录

### Requirement: Life chapter API
系统 SHALL 提供 REST API 端点管理人生章节。

#### Scenario: POST generate chapter
- **WHEN** 调用 POST /api/chapters/generate 并提供 start_date, end_date, 可选 title
- **THEN** 生成并返回章节

#### Scenario: GET list chapters
- **WHEN** 调用 GET /api/chapters
- **THEN** 返回所有已生成章节的列表

#### Scenario: GET single chapter
- **WHEN** 调用 GET /api/chapters/{filename}
- **THEN** 返回对应章节的内容
