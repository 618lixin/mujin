## ADDED Requirements

### Requirement: Chat API endpoint
系统 SHALL 提供 POST /api/chat 端点，接受用户消息并返回 AI 回复。

#### Scenario: Successful chat turn
- **WHEN** 用户发送 POST /api/chat 包含 message 和可选的 session_id
- **THEN** 系统返回 AI 回复文本和 session_id

#### Scenario: Chat with new session
- **WHEN** 用户发送消息但未提供 session_id
- **THEN** 系统创建新会话并返回新的 session_id

### Requirement: Prompt assembly
每轮对话的 prompt SHALL 按以下顺序组装：system prompt（角色定义）→ 核心记忆注入 → 人格权重描述 → 对话历史 → 用户当前消息。

#### Scenario: Full prompt assembly
- **WHEN** 构建对话请求
- **THEN** prompt 包含：角色定义 + user_profile.md 内容 + companion_notes.md 内容 + 当前八维权重描述 + 最近 N 轮对话历史 + 用户消息

#### Scenario: No conversation history
- **WHEN** 新会话的第一轮对话
- **THEN** prompt 包含角色定义 + 核心记忆 + 人格权重 + 用户消息，对话历史部分为空

### Requirement: Post-chat pipeline
每轮对话完成后，系统 SHALL 按顺序执行：情绪识别 → 事件记忆写入（如重要性 ≥0.6）→ Reinforcement/Compensation 权重调整 → Reflection 检查（是否触发）→ 日记数据累积。

#### Scenario: Post-chat with significant event
- **WHEN** AI 回复后，情绪识别发现 importance ≥0.6 的事件
- **THEN** 系统写入事件记忆并执行权重调整

#### Scenario: Post-chat with routine conversation
- **WHEN** AI 回复后，情绪识别发现 importance <0.6
- **THEN** 系统仅执行轻量权重调整，不写入事件记忆

### Requirement: Conversation history management
系统 SHALL 保留最近 20 轮对话历史用于 prompt 组装，更早的历史通过事件记忆和核心记忆保留关键信息。

#### Scenario: History within limit
- **WHEN** 对话轮数 ≤20
- **THEN** 全部历史注入 prompt

#### Scenario: History exceeds limit
- **WHEN** 对话轮数 >20
- **THEN** 仅最近 20 轮注入 prompt，更早历史通过记忆系统检索
