# Growth Companion 架构设计文档

## 1. 产品定位

一个能长期陪伴用户成长的 AI 伙伴。不是聊天机器人，不是心理咨询师，而是一个**会记住你、理解你、见证你变化**的数字人格。

核心假设：**人格应该缓慢演化，不是每轮对话都在变。**

---

## 2. 设计哲学

### 2.1 沉淀金字塔

数据从原始对话到人格变化，经过多层沉淀和压缩：

```
            人格微调（周尺度）
           ┌───────────┐
           │ Reflection │  ← 只从沉淀后的数据来
           └─────┬─────┘
          ┌──────┴──────┐
          │  周成长总结   │   ← 从事件和日记中提炼
          └──────┬──────┘
         ┌───────┴───────┐
         │  事件记忆/日记   │   ← 从对话中提取和压缩
         └───────┬───────┘
        ┌────────┴────────┐
        │  零散对话记录      │   ← 原始数据，每轮产生
        └─────────────────┘
```

**关键原则**：每一层只能读取下一层的数据，不能跨层。人格调整只看沉淀后的周报和事件，不看原始对话。

### 2.2 有界记忆

来自 Hermes Agent 的设计启发：记忆不是数据库查询，而是**一直在脑子里的东西**。

- 核心记忆文件有硬字符上限（1200 + 800），逼迫 AI 只留最重要的
- 不用向量数据库，结构化查询足够
- 记忆注入 system prompt 时冻结（利用 prefix caching）

### 2.3 单模型 + 状态机

不用多 Agent。单个 LLM 调用 + 一个八维权重状态向量，通过 prompt 注入控制人格。

---

## 3. 系统架构

```
用户消息
  │
  ▼
┌─────────────────────────────────────────────────────┐
│ Chat Layer                                          │
│                                                     │
│  Prompt 组装：                                       │
│    system prompt（角色定义）                          │
│    + 核心记忆（冻结注入，≤2000 字符）                  │
│    + 八维权重描述（可视化文本）                         │
│    + PAD 情感风格提示（实时情感状态）                    │
│    + 最近 20 轮对话历史                               │
│    + 用户当前消息                                    │
│                                                     │
│  LLM 调用（流式 SSE）                                │
│                                                     │
│  对话后管线（不改权重，只沉淀）：                       │
│    情绪识别 → 事件写入                                │
│    PAD 情感状态更新                                   │
│    日记数据累积                                      │
│    记录活跃时间                                      │
│    Reflection 检查（每周一次）                        │
└─────────────────────────────────────────────────────┘
       │           │           │           │           │
       ▼           ▼           ▼           ▼           ▼
  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
  │ 对话历史  │ │ 三层记忆  │ │ 人格引擎  │ │ PAD情感  │ │ 生成物   │
  │         │ │         │ │         │ │         │ │         │
  │ history │ │ 核心    │ │ 八维权重  │ │ P/A/D   │ │ 日记    │
  │ .json   │ │ 记忆文件 │ │ 状态向量  │ │ 三维状态 │ │ 周报    │
  │ (20轮)  │ │ (注入   │ │         │ │         │ │ 章节叙事 │
  │         │ │  prompt)│ │ 每周    │ │ 情绪驱动 │ │         │
  │         │ │         │ │ 反思调整 │ │ 漂移更新 │ │         │
  │         │ │ SQLite  │ │         │ │ 风格提示 │ │         │
  │         │ │ 事件表   │ │ 快照历史 │ │         │ │         │
  │         │ │ (含遗忘  │ │         │ │ 心跳循环 │ │         │
  │         │ │  曲线)   │ │         │ │ 主动消息 │ │         │
  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘
```

---

## 4. JPAF 八维人格状态机

### 4.1 状态表示

8 个浮点数，每个 ∈ [0.0, 1.0]：

```
Ti (内倾思考)  — 逻辑分析、内在框架
Te (外倾思考)  — 效率、结构化建议、执行
Fi (内倾情感)  — 内在价值观、真实感受
Fe (外倾情感)  — 共情、倾听、回应他人情绪
Si (内倾感觉)  — 经验回忆、细节关注
Se (外倾感觉)  — 当下体验、感官感知
Ni (内倾直觉)  — 深层意义、长远洞察
Ne (外倾直觉)  — 可能性探索、多角度思考
```

### 4.2 初始化

16 种 MBTI 类型映射到预定义权重表：

```python
# 示例
"INFP": {"Ti": 0.3, "Te": 0.2, "Fi": 0.9, "Fe": 0.4,
         "Si": 0.3, "Se": 0.1, "Ni": 0.5, "Ne": 0.7}
```

未提供 MBTI 则均匀分布（每个维度 0.5）。

### 4.3 演化机制

**只有一个机制会修改持久化权重：Reflection。**

| 机制 | 频率 | 是否持久化 | 触发条件 | 幅度 |
|------|------|-----------|---------|------|
| Compensation | 每轮 | 否（仅当轮） | overwhelm 极端情绪 | ±0.03~0.05 |
| Reflection | 每周一次 | 是 | 上次反思不在本周 + 间隔 ≥5 天 | ±0.02（硬限制） |

**已移除的机制**（v2 重构）：
- ~~Reinforcement（逐轮强化）~~ — 被砍掉，不再每轮调整
- ~~Analytical Boost（分析性增量）~~ — 被砍掉
- ~~轮数触发 Reflection~~ — 改为纯周触发

### 4.4 Reflection 详解

**输入**（全部是沉淀后的数据）：

```
1. 近期周报（最近 2 篇，各取前 500 字符）
2. 近期重要事件（最近 10 条，importance ≥ 0.4）
3. 最近 7 天的日记摘要（各取前 200 字符）
4. 当前八维权重
```

**输出**：

```json
{
  "analysis": "基于沉淀数据的趋势判断",
  "new_weights": {"Ti": 0.31, "Te": 0.21, ...},
  "insight": "关于用户的一个洞察"
}
```

**安全约束**：

```python
# 代码级硬限制，不依赖 LLM 自律
for dim in weights:
    delta = new_val - old_val
    delta = max(-0.02, min(0.02, delta))  # 强制 clamp
    weights[dim] = old_val + delta
```

**演化速度估算**：

- 单次最大变化：±0.02/维度
- 一年约 52 次反思
- 某维度从 0.5 到 0.8 需要连续 15 周同方向调整（约 4 个月）

---

## 5. 三层记忆架构

### 5.1 Layer 1：核心记忆（始终在 prompt 中）

两个文件，会话开始时注入 system prompt，会话内不变（冻结快照模式）：

| 文件 | 上限 | 内容 | 用途 |
|------|------|------|------|
| `user_profile.md` | 1200 字符 | MBTI、情绪风格、依恋风格、行为模式 | AI 始终"记得"用户是谁 |
| `companion_notes.md` | 800 字符 | AI 对用户的理解、有效陪伴方式、反思洞察 | AI 的"工作笔记" |

**容量管理**：

- 使用率 > 80% → 提示 AI 压缩
- 超过 100% → 拒绝写入，返回当前内容和容量信息
- 写入操作：add（追加）、replace（子串替换）、remove（子串删除）

**冻结模式**：会话开始时加载，会话内不更新 prompt 中的记忆块。修改立即持久化到磁盘，但只在下次会话生效。好处：
- 利用 prefix caching（系统 prompt 不变，模型不用每轮重算）
- 避免 AI 追着自己的记忆更新跑

### 5.2 Layer 2：事件记忆（SQLite，按需检索，含遗忘曲线）

不注入 prompt，当 AI 需要回顾历史时查询。

```sql
CREATE TABLE events (
    id TEXT PRIMARY KEY,
    content TEXT,
    emotions TEXT,          -- JSON: ["sadness", "anger"]
    importance REAL,        -- 0.0 ~ 1.0
    event_type TEXT,        -- conflict | milestone | emotion | decision
    strength REAL,          -- 遗忘曲线当前强度 (0.0 ~ 1.0)
    stability REAL,         -- 记忆稳定天数 (越高衰减越慢)
    recall_count INTEGER,   -- 被回忆的次数
    last_recalled_at TEXT,  -- 上次被回忆的时间
    created_at DATETIME,
    updated_at DATETIME
);
```

**写入条件**：情绪识别的 importance ≥ 0.6 且 event_type 不为空。

**检索方式**：按 importance、event_type、日期范围、min_strength 过滤。不做向量检索。

**遗忘曲线**：

事件不是永久保留的。每条事件有 `strength`（记忆强度）和 `stability`（稳定天数），模拟艾宾浩斯遗忘曲线：

```
strength = e^(-elapsed_days / stability)
```

- 初始 stability = `基础稳定天数 × (0.5 + importance)` — 重要事件衰减更慢
- 被回忆（recall）时 stability 增加 50% — 越常想起的事越不容易忘
- strength < 0.05 的事件可被清理
- 心跳循环每 30 分钟执行一次衰减和清理

**配置项**（`src/config.py`）：

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `forget_min_strength` | 0.05 | 低于此强度的事件可被清理 |
| `forget_base_stability` | 30.0 | 基础记忆稳定天数 |
| `forget_recall_boost` | 0.5 | 回忆时稳定性增幅 (50%) |

### 5.3 Layer 3：生成物（独立文件，不参与对话循环）

| 类型 | 路径 | 触发 | 格式 |
|------|------|------|------|
| 每日日记 | `diaries/YYYY-MM-DD.md` | 每日首次对话时检查昨日 | 结构化 Markdown |
| 周成长总结 | `summaries/week-YYYY-WNN.md` | 用户手动触发 | 结构化 Markdown |
| 人生章节 | `chapters/标题.md` | 用户手动触发 | 叙事散文 |

这些是"产物"而非"记忆"。它们帮助用户看到自己的变化，但不直接注入对话 prompt。

---

## 6. 情绪识别

### 6.1 算法

每轮对话后，用便宜 LLM 从用户消息 + AI 回复中提取结构化信息。

**输入**：用户消息 + AI 回复

**输出**：

```json
{
  "emotions": ["sadness", "anger"],
  "event_type": "conflict",
  "importance": 0.7,
  "summary": "用户与父亲发生激烈争吵，同时感到愤怒和悲伤。"
}
```

### 6.2 约束

- 情绪标签白名单（10 种）：joy, sadness, anger, anxiety, fear, surprise, disgust, calm, overwhelm, hope
- 事件类型（4 种 + null）：conflict, milestone, emotion, decision
- importance 强制 clamp 到 [0.0, 1.0]
- 日常闲聊返回空情绪、null 事件类型、低 importance

### 6.3 下游消费

情绪识别结果**不直接改权重**。它只是沉淀层的数据采集：

- importance ≥ 0.6 → 写入事件表（供后续 Reflection 和周报使用）
- 情绪标签 → 累积到 day_data（供日记生成使用）
- 情绪标签含 overwhelm → 当轮临时 Compensation（不持久化）

---

## 7. 对话流程

### 7.1 Prompt 组装

按以下固定顺序拼接：

```
1. 角色定义（固定模板，约 200 字符）
2. 核心记忆冻结块（≤ 2000 字符）
3. 八维权重可视化描述（约 200 字符）
4. PAD 情感风格提示（根据实时 PAD 状态生成）
5. 最近 20 轮对话历史
6. 用户当前消息
```

### 7.2 对话后管线

```
AI 回复完成
  │
  ├→ 保存对话历史（history.json，保留最近 20 轮）
  │
  ├→ 更新对话计数器
  │
  ├→ 情绪识别（便宜 LLM，~200ms）
  │    ├→ importance ≥ 0.6 → 写入 events 表（沉淀，含遗忘曲线字段）
  │    └→ 情绪数据 → 累积到 day_data（沉淀）
  │
  ├→ PAD 情感状态更新
  │    └→ 根据检测到的情绪 → PAD 向目标漂移（或向中性回归）
  │
  ├→ 记录最后活跃时间（last_activity.json，供心跳使用）
  │
  ├→ Compensation 检查（仅 overwhelm 场景）
  │    └→ 如果触发：当轮临时权重增量（不持久化）
  │
  └→ Reflection 检查（每周一次）
       └→ 如果触发：读取周报+事件+日记 → LLM 反思 → 微调权重 ±0.02
```

### 7.3 流式输出

通过 SSE（Server-Sent Events）实现逐 token 输出：

```
后端：stream_llm() → 逐 token yield
API：POST /api/chat/stream → StreamingResponse
前端：fetch → ReadableStream → 逐字追加到气泡
```

对话后管线在流式输出完成后异步执行，不阻塞用户看到回复。

---

## 8. PAD 情感模型

### 8.1 概述

基于 Mehrabian 的 PAD（Pleasure-Arousal-Dominance）情感模型，AI 自身有一个三维情感状态，影响回复风格。

三个维度：

```
Pleasure（愉悦度）：-1（不悦）~ 1（愉悦）
  → 影响语气温度：愉悦时更轻松幽默，低落时更温柔倾听

Arousal（激活度）：0（平静）~ 1（高度激活）
  → 影响回复节奏：活跃时更主动追问，安静时更简洁平和

Dominance（支配度）：0（服从）~ 1（支配）
  → 影响主导性：自信时给出更明确建议，柔软时多用提问引导
```

### 8.2 情绪映射

10 种离散情绪 → PAD 目标值：

| 情绪 | Pleasure | Arousal | Dominance |
|------|----------|---------|-----------|
| joy | +0.6 | 0.5 | 0.4 |
| sadness | -0.6 | 0.1 | 0.2 |
| anger | -0.5 | 0.7 | 0.6 |
| anxiety | -0.4 | 0.6 | 0.2 |
| fear | -0.6 | 0.6 | 0.1 |
| surprise | +0.2 | 0.8 | 0.5 |
| disgust | -0.6 | 0.3 | 0.6 |
| calm | +0.4 | 0.1 | 0.5 |
| overwhelm | -0.5 | 0.7 | 0.1 |
| hope | +0.5 | 0.3 | 0.5 |

### 8.3 状态更新机制

```
每轮对话后：
  有情绪 → 计算所有情绪的 PAD 均值作为 target → 向 target 漂移
  无情绪 → 向中性基线 (P=0, A=0.3, D=0.5) 缓慢回归
```

漂移公式：`new = current + rate × (target - current)`

配置项：

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `pad_drift_rate` | 0.2 | 情绪驱动时的漂移速度 |
| `pad_decay_rate` | 0.05 | 无情绪时的回归速度 |

### 8.4 风格提示注入

PAD 状态生成文本化的风格提示，注入 system prompt：

- Pleasure > 0.4 → "回复更轻松愉快、带些幽默感"
- Pleasure < -0.3 → "语气更温柔，先倾听再回应"
- Arousal > 0.6 → "更主动追问、展开话题，回复稍长"
- Arousal < 0.2 → "回复更简洁平和"
- Dominance > 0.6 → "给出更明确的观点和建议"
- Dominance < 0.3 → "更多用提问和引导"

### 8.5 存储

`data/{user_id}/pad_state.json`，每轮对话后持久化。

---

## 9. 心跳机制

### 9.1 概述

后台异步循环，让 AI 在用户不说话时也有"生命感"：情感状态会漂移、事件记忆会衰减、空闲太久会主动打招呼。

### 9.2 心跳循环

```
应用启动 → asyncio.create_task(heartbeat_loop)
  │
  └→ 每 30 分钟执行一次 _heartbeat_tick：
       │
       ├→ PAD 空闲漂移：向 PAD_IDLE_TARGET (P=-0.1, A=0.15, D=0.4) 漂移
       │   → AI 独处时情绪会趋向"略感想念、安静、温和"
       │
       ├→ 遗忘曲线维护：
       │   ├→ decay_all_events：所有事件的 strength 按遗忘公式衰减
       │   └→ cleanup_forgotten_events：清理 strength < 0.05 的事件
       │
       └→ 主动消息检查：
            ├→ 空闲 > 24h → 强制签到（long_idle）
            ├→ 空闲 > 2h 且 P < -0.3 → 想念用户（lonely）
            ├→ 空闲 > 2h 且 P > 0.3 且 A > 0.6 → 想分享（excited）
            └→ 条件满足 → LLM 生成 1-2 句自然消息 → 存入 pending
```

### 9.3 主动消息

生成的消息暂存在内存中，前端通过轮询 `/api/heartbeat` 获取：

```json
{
  "proactive_message": {
    "message": "好久没聊了，最近怎么样？",
    "reason": "long_idle",
    "pad": {"pleasure": -0.08, "arousal": 0.18, "dominance": 0.42},
    "created_at": "2026-05-23T15:30:00"
  },
  "pad": {"pleasure": -0.08, "arousal": 0.18, "dominance": 0.42}
}
```

**消息生成约束**：
- 不超过 1-2 句话
- 不提及自己是 AI 或 PAD 数值
- 不矫情、不戏剧化
- 像真正的朋友那样自然

### 9.4 配置项

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `heartbeat_interval_minutes` | 30 | 心跳间隔 |
| `heartbeat_min_idle_minutes` | 120 | 最短空闲才触发主动消息 |
| `heartbeat_max_idle_minutes` | 1440 | 最长空闲强制签到 (24h) |
| `heartbeat_idle_drift_rate` | 0.05 | 空闲 PAD 漂移速度 |
| `heartbeat_proactive_enabled` | True | 是否启用主动消息 |

### 9.5 生命周期

心跳任务通过 FastAPI lifespan 管理：应用启动时 `create_task`，关闭时 `cancel`。

---

## 10. 日记系统

### 8.1 数据累积

每轮对话后，将情绪识别结果追加到 `day_data_YYYY-MM-DD.json`：

```json
{
  "entries": [
    {"emotions": ["sadness"], "event_type": "conflict", "importance": 0.7, "summary": "..."},
    {"emotions": [], "event_type": null, "importance": 0.1, "summary": ""}
  ]
}
```

### 8.2 日记生成

**触发**：每日首次对话时检查昨日是否有未生成的日记。

**输入**：day_data + 当日事件记忆

**LLM Prompt 要点**：
- 语气温暖、客观、有洞察力
- 包含"今天你提到了"段落
- 包含"成长观察"段落（发现积极信号）
- 不说教

**输出**：`diaries/YYYY-MM-DD.md`，不覆盖已有文件。

---

## 11. 周成长总结

### 11.1 生成逻辑

**输入**：
- 指定周的事件记忆（importance ≥ 0.3）
- 事件不足 3 条时标注"本周对话较少"

**LLM Prompt 要点**：
- 给这周起一个有温度的标题
- 本周概览（2-3 句话）
- 情绪变化描述
- 重要事件列表
- 成长观察

### 11.2 存储

`summaries/week-YYYY-WNN.md`，已存在时不覆盖。

---

## 12. 人生章节

### 12.1 生成逻辑

**输入**：
- 指定日期范围内的事件记忆
- 可选自定义标题

**LLM Prompt 要点**：
- 像写传记的一个章节
- 包含：这段时光、关键时刻、你在变化、未完待续

### 12.2 存储

`chapters/标题.md`。

---

## 13. 数据存储

```
data/{user_id}/
  ├── user_profile.md              # 核心记忆：用户画像（≤1200 字符）
  ├── companion_notes.md           # 核心记忆：AI 笔记（≤800 字符）
  ├── personality_weights.json     # 八维权重当前值
  ├── turn_counter.json            # 对话总轮数
  ├── last_reflection.json         # 上次反思日期
  ├── history.json                 # 最近 20 轮对话历史
  ├── pad_state.json               # PAD 情感状态
  ├── last_activity.json           # 最后活跃时间（心跳使用）
  ├── events.db                    # SQLite
  │   ├── events                   # 事件记忆表（含遗忘曲线字段）
  │   ├── personality_snapshots    # 人格权重快照表
  │   ├── conversation_turns       # 对话轮次（FTS5 全文搜索）
  │   └── insights                 # 结构化洞察表
  ├── day_data_YYYY-MM-DD.json     # 日记原始数据
  ├── diaries/
  │   └── YYYY-MM-DD.md            # 每日日记
  ├── summaries/
  │   └── week-YYYY-WNN.md         # 周成长总结
  └── chapters/
      └── 标题.md                   # 人生章节
```

---

## 14. API 端点

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/init` | 人格初始化（问卷 → 画像 + 权重） |
| POST | `/api/chat` | 非流式聊天 |
| POST | `/api/chat/stream` | 流式聊天（SSE） |
| GET | `/api/heartbeat` | 前端轮询：主动消息 + PAD 状态 |
| GET | `/api/memory/core` | 读取核心记忆 |
| PATCH | `/api/memory/core` | 编辑核心记忆（add/replace/remove） |
| GET | `/api/memory/events` | 查询事件记忆（支持 min_strength 过滤） |
| DELETE | `/api/memory/events/{id}` | 删除事件 |
| POST | `/api/memory/events/maintain` | 手动触发遗忘曲线衰减和清理 |
| GET | `/api/memory/search` | 全文搜索对话记录（FTS5） |
| GET | `/api/memory/insights` | 查询结构化洞察 |
| GET | `/api/personality` | 当前人格权重 |
| GET | `/api/personality/history` | 人格权重历史快照 |
| GET | `/api/personality/pad` | 当前 PAD 情感状态 |
| GET | `/api/diary` | 日记列表 |
| GET | `/api/diary/{date}` | 查看日记 |
| POST | `/api/diary/generate` | 生成日记 |
| POST | `/api/diary/{date}/regenerate` | 重新生成日记 |
| POST | `/api/diary/batch-generate` | 批量补生成 |
| GET | `/api/summary/weekly` | 周报列表 |
| GET | `/api/summary/weekly/{year}/{week}` | 查看周报 |
| POST | `/api/summary/weekly` | 生成周报 |
| GET | `/api/chapters` | 章节列表 |
| GET | `/api/chapters/{filename}` | 查看章节 |
| POST | `/api/chapters/generate` | 生成章节 |

---

## 15. 技术栈

| 层 | 技术 |
|---|---|
| 后端 | Python 3.11 + FastAPI |
| 存储 | SQLite + 文件系统 |
| LLM | DeepSeek / OpenAI 兼容 API（流式 + 非流式） |
| 前端 | 单 HTML（无框架，原生 JS + fetch） |
| 环境管理 | Conda |

**不用的**：
- 不用向量数据库（MVP 不需要语义搜索）
- 不用 Agent 框架（单模型 + 状态机足够）
- 不用前端框架（单 HTML 够用）
- 不用 Redis（SQLite 足够）

---

## 16. 设计权衡

### 为什么用文件存核心记忆而不是数据库？

核心记忆需要注入 system prompt，文件可以直接读取拼接，零延迟。硬上限（1200 + 800 字符）逼迫 AI 只保留最重要的信息，避免无限膨胀。

### 为什么不用向量数据库？

单用户 MVP 场景，事件量小（每天几条到几十条）。结构化查询（按 importance、日期、类型过滤）已经足够。如果后续需要语义搜索（"我之前聊过和爸爸的事"），再加 pgvector。

### 为什么不用多 Agent？

MVP 阶段增加调度复杂度且拉高 token 消耗。单模型 + 状态机足以验证核心假设（沉淀式人格演化是否有用）。

### 为什么人格每周才调一次？

人格应该像真实性格一样缓慢变化。一个人不会因为聊了一次天就变一个人格。只有持续的、沉淀后的模式（持续一周的情绪趋势、反复出现的话题）才应该影响人格。单次对话只是数据点，不是趋势。

### 为什么 Compensation 保留？

极端场景（用户情绪崩溃）下，AI 需要立即调整风格（比如从分析模式切换到共情模式）。但这个切换是临时的、仅当轮有效的，不影响基础人格。

### 为什么 PAD 和 JPAF 八维权重分开？

JPAF 八维权重描述的是 AI 的"人格"（长期稳定，每周才调），PAD 描述的是 AI 的"情绪"（短期波动，每轮都在变）。人格决定"我是谁"，情绪决定"我现在什么状态"。两者独立演化，互不干扰，但都影响回复风格。

### 为什么加遗忘曲线？

无限积累的事件记忆会导致：查询变慢、噪音事件淹没重要事件、不符合人类记忆的衰减特性。遗忘曲线让重要事件（高 importance + 被反复回忆）保留更久，日常琐事自然消退。这与沉淀金字塔的设计哲学一致——只留最重要的。

### 为什么心跳用轮询而不是 WebSocket？

MVP 阶段前端是单 HTML，用 fetch 轮询最简单。心跳间隔 30 分钟，轮询频率远低于间隔，几乎不影响性能。如果后续需要实时推送（如打字中状态），再加 WebSocket。
