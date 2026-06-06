## Purpose
Define how the system extracts structured emotions, events, summaries, and topics from chat turns.

## Requirements

### Requirement: Emotion and event extraction
系统 SHALL 在每轮对话后调用 LLM，从用户消息和 AI 回复中提取情绪标签、事件类型、重要性评分、摘要和主题。

#### Scenario: Extract emotions from emotional message
- **WHEN** 用户消息包含明显情绪表达
- **THEN** 返回对应 emotions、`importance` 和一句话 `summary`

#### Scenario: Extract from neutral message
- **WHEN** 用户消息为日常对话或低信息闲聊
- **THEN** 返回空 emotions、低 importance、`event_type = null`

### Requirement: Emotion labels
系统 SHALL 使用预定义情绪标签集：`joy`、`sadness`、`anger`、`anxiety`、`fear`、`surprise`、`disgust`、`calm`、`overwhelm`、`hope`。

#### Scenario: Multiple emotions
- **WHEN** 用户消息同时表达多种情绪
- **THEN** 返回多个合法 emotions 标签

#### Scenario: Invalid emotion label
- **WHEN** LLM 返回未定义情绪标签
- **THEN** 系统过滤该标签

### Requirement: Event type classification
系统 SHALL 将识别到的事件分类为 `conflict`、`milestone`、`emotion`、`decision` 之一；无明确事件时返回 `null`。

#### Scenario: Conflict event
- **WHEN** 用户描述与他人的矛盾
- **THEN** `event_type` 为 `conflict`

#### Scenario: Decision event
- **WHEN** 用户描述重要选择或决定
- **THEN** `event_type` 为 `decision`

### Requirement: Topic extraction
系统 SHALL 从重要对话中抽取 topics，用于后续主题表和事件主题关联。

#### Scenario: Topics found
- **WHEN** 用户消息包含可持续追踪的主题
- **THEN** 抽取结果包含 topics 数组

#### Scenario: No topics found
- **WHEN** 对话没有稳定主题
- **THEN** topics SHALL 为空数组

### Requirement: Extraction output structure
情绪抽取 SHALL 返回结构化 JSON：`emotions`、`event_type`、`importance`、`summary`、`topics`。

#### Scenario: Full extraction output
- **WHEN** 情绪抽取完成
- **THEN** 返回可反序列化为 `EmotionResult` 的 JSON 对象
