## ADDED Requirements

### Requirement: Daily diary generation
系统 SHALL 在每日首次对话时检查是否有未生成的昨日日记，若存在则触发日记生成。

#### Scenario: Generate yesterday's diary
- **WHEN** 当日首次对话开始，且昨日有对话记录但未生成日记
- **THEN** 系统基于昨日的事件记忆和对话摘要，调用 LLM 生成日记文件

#### Scenario: No conversations yesterday
- **WHEN** 当日首次对话开始，但昨日无对话记录
- **THEN** 系统不生成日记

### Requirement: Diary content format
日记 SHALL 为 Markdown 格式，包含日期标题、关键事件列表、情绪变化描述、一个"成长观察"段落。

#### Scenario: Diary structure
- **WHEN** 日记生成完成
- **THEN** 文件包含 # YYYY-MM-DD 标题、"今天你提到了"段落、情绪描述、"成长观察"段落

### Requirement: Diary file storage
日记 SHALL 存储在 data/{user_id}/diaries/ 目录下，文件名为 YYYY-MM-DD.md。

#### Scenario: Diary file created
- **WHEN** 日记生成完成
- **THEN** data/{user_id}/diaries/YYYY-MM-DD.md 文件存在

#### Scenario: Diary already exists
- **WHEN** 对应日期的日记文件已存在
- **THEN** 系统 SHALL NOT 覆盖已有日记

### Requirement: Diary API
系统 SHALL 提供 REST API 端点查看和触发日记生成。

#### Scenario: GET diary by date
- **WHEN** 调用 GET /api/diary/{date}
- **THEN** 返回对应日期的日记内容，若不存在返回 404

#### Scenario: POST trigger diary generation
- **WHEN** 调用 POST /api/diary/generate 并指定日期
- **THEN** 系统为该日期生成日记（若已存在则返回冲突提示）
