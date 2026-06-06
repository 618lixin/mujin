## Purpose
Define the current qualitative growth model that replaced the removed numeric personality-weight system.

## Requirements

### Requirement: Qualitative growth model
系统 SHALL 使用定性成长观察替代旧的八维人格权重系统。

#### Scenario: No numeric personality weights
- **WHEN** 构建对话 prompt 或保存用户状态
- **THEN** 系统 SHALL NOT 注入或更新 Ti/Te/Fi/Fe/Si/Se/Ni/Ne 权重

#### Scenario: Growth observation represented as text
- **WHEN** 系统保存成长洞察
- **THEN** 使用自然语言 observation，而不是浮点人格权重

### Requirement: Observation storage
系统 SHALL 支持 `observations` 表，用于保存定性成长观察。

#### Scenario: Query observations
- **WHEN** 前端调用 `ai_get_observations`
- **THEN** 系统返回可按 category 过滤的观察列表

### Requirement: Topic, project, and growth-line storage
系统 SHALL 支持 topics、projects、growth_lines 等长期成长档案数据结构。

#### Scenario: Query topics
- **WHEN** 前端调用 `ai_get_topics`
- **THEN** 系统返回主题列表

#### Scenario: Query projects
- **WHEN** 前端调用 `ai_get_projects`
- **THEN** 系统返回项目档案列表

#### Scenario: Query growth lines
- **WHEN** 前端调用 `ai_get_growth_lines`
- **THEN** 系统返回成长线列表

### Requirement: Reflection status
Reflection 自动生成观察、项目归并和成长线更新 SHALL 作为后续功能处理；当前 post-chat pipeline 仅保留占位返回。

#### Scenario: Post-chat reflection placeholder
- **WHEN** post-chat pipeline 完成
- **THEN** `reflection` 字段当前为 `None`

#### Scenario: No automatic observation write
- **WHEN** 完成普通聊天
- **THEN** 系统 SHALL NOT 声称已经自动生成 observation，除非后续实现明确写入

### Requirement: Growth review avoids numeric personality history
Growth review features SHALL use qualitative observations, topics, projects, and growth lines without reintroducing numeric personality-weight snapshots.

#### Scenario: Render growth review
- **WHEN** the frontend displays growth history or review material
- **THEN** the system shows qualitative records and SHALL NOT show Ti/Te/Fi/Fe/Si/Se/Ni/Ne weight timelines

#### Scenario: Generate review prompt
- **WHEN** weekly summary or life chapter generation builds a prompt
- **THEN** the prompt may include qualitative observations but SHALL NOT include removed numeric personality weights
