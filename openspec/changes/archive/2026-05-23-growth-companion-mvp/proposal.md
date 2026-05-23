## Why

用户需要一个能长期陪伴、记住人生轨迹、形成稳定人格关系的 AI 伙伴。现有 AI 聊天产品（ChatGPT、Character AI）要么没有真正的长期记忆，要么人格是静态标签无法演化。基于 JPAF 论文的八维人格框架 + Hermes 启发的有界记忆架构，可以构建一个"陪用户一起长大的数字人格伙伴"。

## What Changes

- 新增人格初始化系统：通过问卷建立用户画像（MBTI、情绪风格、依恋风格、陪伴偏好）
- 新增三层记忆架构：核心记忆（有界文件注入 prompt）+ 事件记忆（SQLite 结构化存储）+ 日记生成物（独立文件）
- 新增 JPAF 八维人格演化引擎：基于 Reinforcement / Compensation / Reflection 三大机制动态调整 AI 人格权重
- 新增长期陪伴聊天接口：集成记忆检索、人格状态注入的对话 API
- 新增自动日记生成：每轮对话后提取关键事件和情绪，生成每日日记
- 新增情绪识别模块：从对话中提取情绪标签和重要性评分，驱动事件记忆写入

## Capabilities

### New Capabilities

- `personality-init`: 人格初始化——问卷采集 + 生成 user_profile.md + 初始化八维权重向量
- `core-memory`: 核心记忆系统——有界文件（user_profile.md + companion_notes.md）的读写、容量管理、prompt 注入
- `event-memory`: 事件记忆——SQLite 结构化存储，情绪标签、重要性评分、按条件检索
- `personality-engine`: JPAF 人格演化引擎——八维权重状态机，Reinforcement / Compensation / Reflection 三大机制
- `companion-chat`: 陪伴聊天——集成记忆检索 + 人格状态注入的对话 API
- `diary-generator`: 自动日记生成——对话后提取关键事件和情绪，生成每日日记文件
- `emotion-extractor`: 情绪识别——从对话文本中提取情绪标签、事件类型、重要性评分

### Modified Capabilities

（无现有能力需要修改）

## Impact

- **新增依赖**：Python 3.11+, FastAPI, SQLite（标准库）, Pydantic（数据校验）
- **新增 API 端点**：初始化问卷、聊天、记忆管理、日记查看
- **文件系统**：用户数据目录结构（profiles/, memories/, diaries/, events.db）
- **LLM API 调用**：聊天、情绪识别、日记生成、人格反思均需调用 LLM
