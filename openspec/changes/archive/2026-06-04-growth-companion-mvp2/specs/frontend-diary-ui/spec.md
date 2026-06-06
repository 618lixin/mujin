## ADDED Requirements

### Requirement: Diary list view
前端 SHALL 提供日记列表页面，显示所有已生成日记的日期和摘要。

#### Scenario: View diary list
- **WHEN** 用户进入日记页面
- **THEN** 页面显示所有已生成日记的卡片列表，每张卡片显示日期和内容摘要

### Requirement: Diary detail view
前端 SHALL 支持点击日记卡片查看完整内容。

#### Scenario: Click diary card
- **WHEN** 用户点击某一天的日记卡片
- **THEN** 展开显示完整日记内容

### Requirement: Diary regeneration button
前端 SHALL 在每条日记旁提供重新生成按钮。

#### Scenario: Click regenerate
- **WHEN** 用户点击"重新生成"按钮
- **THEN** 调用后端重新生成 API 并刷新日记内容

### Requirement: Batch generate button
前端 SHALL 提供批量生成按钮，为缺失的日期补生成日记。

#### Scenario: Click batch generate
- **WHEN** 用户点击"补生成日记"按钮
- **THEN** 调用批量生成 API 并显示结果

### Requirement: Weekly summary view
前端 SHALL 在日记页面提供"周成长总结"区域，支持查看和生成周报。

#### Scenario: View weekly summaries
- **WHEN** 用户进入日记页面
- **THEN** 页面右侧或底部显示已生成的周报列表

#### Scenario: Generate weekly summary
- **WHEN** 用户点击"生成本周总结"
- **THEN** 调用后端 API 生成并显示周报
