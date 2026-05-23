## ADDED Requirements

### Requirement: Core memory editor
前端 SHALL 提供核心记忆的编辑界面，支持直接编辑 user_profile.md 和 companion_notes.md 的内容。

#### Scenario: Edit user profile
- **WHEN** 用户在记忆页面修改用户画像文本并点击保存
- **THEN** 调用后端 API 更新核心记忆文件

### Requirement: Event management list
前端 SHALL 显示所有事件记忆的列表，支持按类型筛选和删除。

#### Scenario: View events with filter
- **WHEN** 用户进入记忆页面
- **THEN** 显示事件列表，顶部提供 event_type 筛选按钮

#### Scenario: Delete event
- **WHEN** 用户点击某事件的删除按钮
- **THEN** 调用后端删除 API 并刷新列表

### Requirement: Personality history visualization
前端 SHALL 以时间线形式展示人格权重的变化历史。

#### Scenario: View personality history
- **WHEN** 用户进入记忆页面的人格标签
- **THEN** 显示人格权重变化时间线，每条记录显示时间、权重柱状图和反思摘要

### Requirement: Life chapter generation UI
前端 SHALL 提供人生章节生成界面，支持选择日期范围和自定义标题。

#### Scenario: Generate life chapter
- **WHEN** 用户选择日期范围并点击"生成章节"
- **THEN** 调用后端 API 生成章节并显示结果

#### Scenario: View life chapters
- **WHEN** 用户进入人生章节区域
- **THEN** 显示所有已生成章节的列表，点击可查看完整内容
