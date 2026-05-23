## ADDED Requirements

### Requirement: Personality initialization questionnaire
系统 SHALL 提供一组初始化问卷，采集用户的人格基础信息，包括 MBTI 类型、情绪表达偏好、陪伴风格偏好、建议力度偏好、边界感偏好。

#### Scenario: First-time user completes questionnaire
- **WHEN** 用户首次访问初始化端点并提供问卷回答
- **THEN** 系统生成 user_profile.md 文件，包含结构化的用户画像信息

#### Scenario: Questionnaire with partial answers
- **WHEN** 用户提交问卷但部分字段为空
- **THEN** 系统 SHALL 为缺失字段使用默认值，并成功生成 user_profile.md

### Requirement: Initial personality weights
系统 SHALL 根据用户 MBTI 类型生成初始八维人格权重向量（Ti/Te/Fi/Fe/Si/Se/Ni/Ne），每个权重为 0.0~1.0 的浮点数。

#### Scenario: INFP user initialization
- **WHEN** 用户 MBTI 为 INFP
- **THEN** 系统生成 Fi 主导（≥0.7）、Ne 辅助（≥0.5）的权重向量，其余维度较低（≤0.4）

#### Scenario: MBTI not provided
- **WHEN** 用户未提供 MBTI 类型
- **THEN** 系统使用均匀分布的默认权重（每个维度 0.5）

### Requirement: User profile file generation
系统 SHALL 将用户画像持久化为 user_profile.md 文件，存储在 data/{user_id}/ 目录下，内容为纯文本格式，不超过 1200 字符。

#### Scenario: Profile file created
- **WHEN** 初始化完成
- **THEN** data/{user_id}/user_profile.md 文件存在且内容包含用户的关键画像信息

#### Scenario: Profile exceeds character limit
- **WHEN** 生成的画像超过 1200 字符
- **THEN** 系统 SHALL 压缩内容至 1200 字符以内再保存
