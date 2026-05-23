## ADDED Requirements

### Requirement: Emotion extraction from message
系统 SHALL 在每轮对话后调用 LLM 从用户消息和 AI 回复中提取情绪标签、事件类型和重要性评分。

#### Scenario: Extract emotions from emotional message
- **WHEN** 用户消息包含明显情绪表达（如"我好难过"、"我崩溃了"）
- **THEN** 返回 emotions 包含对应标签（如 ["sadness"]）、importance ≥0.7

#### Scenario: Extract from neutral message
- **WHEN** 用户消息为日常对话（如"今天天气不错"）
- **THEN** 返回 emotions 为空、importance <0.3、event_type 为 null

### Requirement: Emotion labels
系统 SHALL 使用预定义的情绪标签集：joy, sadness, anger, anxiety, fear, surprise, disgust, calm, overwhelm, hope。每条提取结果可包含多个标签。

#### Scenario: Multiple emotions
- **WHEN** 用户消息同时表达悲伤和愤怒（如"他太过分了，我好难过"）
- **THEN** 返回 emotions: ["sadness", "anger"]

### Requirement: Event type classification
系统 SHALL 将识别到的事件分类为以下类型之一：conflict（冲突）、milestone（里程碑）、emotion（情绪事件）、decision（决策）。当无明确事件时返回 null。

#### Scenario: Conflict event
- **WHEN** 用户描述与他人的矛盾
- **THEN** event_type 为 "conflict"

#### Scenario: Milestone event
- **WHEN** 用户描述一个重要转变或成就
- **THEN** event_type 为 "milestone"

#### Scenario: No event
- **WHEN** 对话为闲聊或日常问答
- **THEN** event_type 为 null

### Requirement: Importance scoring
系统 SHALL 为每次提取结果生成 0.0~1.0 的重要性评分。评分基于情绪强度、事件影响范围、是否涉及人生重大变化。

#### Scenario: High importance event
- **WHEN** 用户描述分手、失业、重大决策等
- **THEN** importance ≥0.8

#### Scenario: Low importance event
- **WHEN** 用户描述日常小事或一般性讨论
- **THEN** importance <0.3

### Requirement: Extraction output structure
情绪识别 SHALL 返回结构化 JSON：{"emotions": [...], "event_type": "...|null", "importance": 0.0~1.0, "summary": "一句话摘要"}。

#### Scenario: Full extraction output
- **WHEN** 情绪识别完成
- **THEN** 返回包含 emotions 数组、event_type 字符串或 null、importance 浮点数、summary 字符串的 JSON 对象
