## Context

Growth Companion 是一个长期陪伴型 AI 产品。当前项目从零开始，没有任何已有代码。技术栈锁定 Python + FastAPI + SQLite。理论基础是 JPAF 论文（荣格八维人格演化）和 Hermes Agent 的有界记忆架构。MVP 阶段不做前端，只提供 API。

## Goals / Non-Goals

**Goals:**
- 实现三层记忆架构（核心记忆文件 + SQLite 事件记忆 + 日记生成物）
- 实现 JPAF 八维人格演化引擎（Reinforcement / Compensation / Reflection）
- 提供可工作的聊天 API，能感知用户情绪、记忆历史、动态调整人格
- 自动生成每日日记
- 单用户本地部署即可运行

**Non-Goals:**
- 不做前端 UI（MVP2）
- 不做多用户/认证系统
- 不做向量数据库 / embedding 检索
- 不做周总结和成长叙事生成器（MVP2）
- 不做 Agent 框架集成（LangGraph 等）
- 不做实时推送 / WebSocket

## Decisions

### D1: 单 LLM + 状态机，不用多 Agent

**选择**：单模型调用 + 八维权重状态向量，通过 prompt 注入控制人格。

**替代方案**：多 Agent（一个负责聊天，一个负责记忆，一个负责反思）。

**理由**：MVP 阶段多 Agent 增加调度复杂度且拉高 token 消耗。单模型 + 状态机足以验证核心假设（人格演化 + 长期记忆是否有用）。

### D2: 核心记忆用文件，不用数据库

**选择**：user_profile.md 和 companion_notes.md 存为文件，有字符硬限（1200 + 800）。

**替代方案**：存数据库，按需检索注入。

**理由**：参考 Hermes 的设计——有界文件逼迫策展，避免无限膨胀。文件可直接注入 system prompt，零检索延迟。SQLite 专用于事件记忆。

### D3: 事件记忆用 SQLite，不用向量数据库

**选择**：SQLite + importance/event_type/emotions 元数据过滤。

**替代方案**：pgvector / Chroma 做 embedding 检索。

**理由**：MVP 事件量小（单用户），结构化查询足够。向量检索在 MVP 阶段是过度工程。如果 MVP2 需要语义搜索，再加 pgvector。

### D4: 情绪识别用 LLM，不用专用模型

**选择**：每次对话后用 LLM 提取情绪标签、事件类型、重要性评分。

**替代方案**：用专门的 emotion detection 模型（如 GoEmotions）。

**理由**：避免引入额外模型依赖。LLM 已经理解对话上下文，零成本额外提取情绪信息。

### D5: 人格权重调整频率

**选择**：
- 每轮对话：轻量 Reinforcement / Compensation（调权重 ±0.05）
- 每 10 轮对话或每日首次对话：Reflection（回顾近况，更新人格结构）

**替代方案**：每轮都跑完整 Reflection。

**理由**：Reflection 需要回顾历史，token 消耗高。高频 Reflection 会拖慢响应且浪费 token。

### D6: 日记生成策略

**选择**：每轮对话结束后，累积情绪/事件到内存。当日首次空闲或用户触发时，LLM 生成日记文件。

**替代方案**：定时任务（cron）生成。

**理由**：MVP 没有后台调度基础设施。在对话流程内触发更简单。

### D7: 项目目录结构

```
src/
  main.py              # FastAPI 入口
  config.py            # 配置管理
  models/
    personality.py     # 八维权重状态机
    memory.py          # 记忆数据模型
    event.py           # 事件数据模型
    diary.py           # 日记数据模型
  services/
    chat.py            # 聊天服务（组装 prompt + 调 LLM）
    memory.py          # 记忆服务（核心记忆 CRUD + 事件记忆）
    personality.py     # 人格演化引擎
    emotion.py         # 情绪识别
    diary.py           # 日记生成
    llm.py             # LLM API 调用封装
  api/
    init.py            # 初始化问卷 API
    chat.py            # 聊天 API
    memory.py          # 记忆管理 API
    diary.py           # 日记 API
data/
  {user_id}/
    user_profile.md
    companion_notes.md
    events.db
    diaries/
```

## Risks / Trade-offs

**[核心记忆硬限丢失信息]** → AI 主动合并压缩。若用户觉得丢失重要信息，可通过 API 手动编辑核心记忆文件。

**[单 LLM 调用延迟高]** → 情绪识别和日记生成可异步执行。聊天主流程只做 prompt 组装 + LLM 调用 + 轻量权重调整。

**[LLM 成本]** → MVP 单用户场景可控。日记生成和 Reflection 可选用便宜模型（如 GPT-4o-mini）。

**[情绪识别不准]** → 这是已知局限。MVP 阶段接受精度换取简单性，后续可加用户反馈校正。

**[无前端]** → MVP 通过 API 测试。可能影响演示效果，但降低开发范围。
