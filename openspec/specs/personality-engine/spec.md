## ADDED Requirements

### Requirement: Eight-dimension personality state
系统 SHALL 维护一个八维人格权重向量 [Ti, Te, Fi, Fe, Si, Se, Ni, Ne]，每个维度为 0.0~1.0 的浮点数。权重 SHALL 持久化存储并在会话间保持。

#### Scenario: Load personality state
- **WHEN** 新对话开始
- **THEN** 系统加载持久化的八维权重向量

#### Scenario: Weights bounded to valid range
- **WHEN** 任何机制调整权重
- **THEN** 每个维度 SHALL 限制在 [0.0, 1.0] 范围内

### Requirement: Reinforcement mechanism
系统 SHALL 在每轮对话后根据用户反馈方向增强对应人格维度。当用户表达需要共情/安慰/理解时增强 Fe；当用户需要分析/逻辑/结构化时增强 Te/Ti；当用户需要实际建议/行动计划时增强 Te/Se。

#### Scenario: User seeks emotional support
- **WHEN** 用户表达负面情绪且情绪识别标签包含 sadness 或 anxiety
- **THEN** Fe 权重增加 0.03~0.05

#### Scenario: User seeks analytical help
- **WHEN** 用户提出需要分析或决策的问题
- **THEN** Ti/Te 权重增加 0.03~0.05

### Requirement: Compensation mechanism
系统 SHALL 在检测到用户遇到特殊情境时临时增强补偿维度。当用户面临强压力或决策困难时，临时提升 Te/Ti；当用户处于社交困难时，临时提升 Fe。

#### Scenario: User under high stress
- **WHEN** 事件记忆中出现 importance ≥0.8 的 stress 类事件，且用户当前情绪标签包含 overwhelm
- **THEN** Te 权重临时增加 0.1（仅当轮有效，不持久化）

#### Scenario: Compensation is temporary
- **WHEN** 补偿机制在某一轮被触发
- **THEN** 补偿增量仅应用于当轮对话的 prompt，不修改持久化的基础权重

### Requirement: Reflection mechanism
系统 SHALL 每 10 轮对话或每日首次对话时执行反思流程：LLM 回顾近期对话摘要和事件，评估当前人格配置是否适合用户需求，输出权重调整建议。

#### Scenario: Trigger reflection after 10 turns
- **WHEN** 累计对话轮数达到 10 的倍数
- **THEN** 系统调用 LLM 执行反思，输入近期对话摘要和当前权重，输出调整后的权重

#### Scenario: Reflection updates persistent weights
- **WHEN** Reflection 产出新的权重建议
- **THEN** 系统将建议权重持久化为基础权重，覆盖之前的值

#### Scenario: Reflection insight saved to companion notes
- **WHEN** Reflection 产出关于用户的重要洞察
- **THEN** 系统 SHALL 尝试将洞察写入 companion_notes.md（受容量限制）

### Requirement: Personality state API
系统 SHALL 提供 REST API 端点查看当前人格权重。

#### Scenario: GET personality state
- **WHEN** 调用 GET /api/personality
- **THEN** 返回当前八维权重向量和上次 Reflection 时间
