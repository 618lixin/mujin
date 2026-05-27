# AI 智能日记系统 — 架构设计文档

## 1. 产品定位

基于「降低输入成本 + 提高输出价值」的 AI 辅助日记管理产品。不是情感陪伴 AI，不是聊天机器人，而是一个**对话即记录、AI 帮你串联分析**的智能日记系统。

核心假设：**成长无法精确量化，但可以被诚实地描述和串联。**

---

## 2. 设计哲学

### 2.1 沉淀金字塔

数据从原始对话到成长观察，经过多层沉淀和压缩：

```
            成长观察（周尺度，定性描述）
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

**关键原则**：每一层只能读取下一层的数据，不能跨层。Reflection 只看沉淀后的周报和事件，不看原始对话。

### 2.2 有界记忆

来自 Hermes Agent 的设计启发：记忆不是数据库查询，而是**一直在脑子里的东西**。

- 核心记忆文件有硬字符上限（1200 + 800），逼迫 AI 只留最重要的
- 不用向量数据库，结构化查询足够 MVP
- 记忆注入 system prompt 时冻结（利用 prefix caching）

### 2.3 定性观察而非量化

成长无法用 ±0.02 的数字精确描述。所有对用户变化的描述使用自然语言，不用数字量化。

---

## 3. 系统架构

系统分为五层：

```
┌────────────────────┐
│ 对话层 Chat Layer │  ← SSE 流式 + prompt 组装（自然朋友风格 + 记忆注入）
└────────┬───────────┘
         ↓ 后处理（异步）
┌────────────────────┐
│ 事件与信息抽取层 │  ← 事件抽取 + 主题识别 + importance
└────────┬───────────┘
         ↓
┌────────────────────┐
│ 长期记忆系统 Memory │  ← 三层：Core(注入prompt) + Event(SQLite+遗忘曲线) + 叙事(文件)
└────────┬───────────┘
         ↓ 周触发
┌────────────────────┐
│ Reflection 引擎 │  ← 每周触发，生成定性观察 + 事件归并 + 成长线更新
└────────┬───────────┘
         ↓
┌────────────────────┐
│ 沉淀生成器 │  ← 日记/周报/章节 + 项目档案/成长线
└────────────────────┘
```

```
用户消息
  │
  ▼
┌─────────────────────────────────────────────────────┐
│ Chat Layer                                          │
│                                                     │
│  Prompt 组装：                                       │
│    system prompt（角色定义，自然朋友风格）               │
│    + 核心记忆（冻结注入，≤2000 字符）                  │
│    + 最近 20 轮对话历史                               │
│    + 用户当前消息                                    │
│                                                     │
│  LLM 调用（流式 SSE）                                │
│                                                     │
│  对话后管线（异步，不阻塞回复）：                       │
│    情绪识别 → 事件写入 + 主题关联                      │
│    日记数据累积                                      │
│    记录活跃时间                                      │
│    Reflection 检查（每周一次）                        │
└─────────────────────────────────────────────────────┘
       │           │           │           │
       ▼           ▼           ▼           ▼
  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
  │ 对话历史  │ │ 三层记忆  │ │ 主题系统  │ │ 生成物   │
  │         │ │         │ │         │ │         │
  │ history │ │ 核心    │ │ topics  │ │ 日记    │
  │ .json   │ │ 记忆文件 │ │ 主题关联 │ │ 周报    │
  │ (20轮)  │ │ (注入   │ │ topic_  │ │ 章节叙事 │
  │         │ │  prompt)│ │ links   │ │ 项目档案 │
  │         │ │         │ │         │ │ 成长线  │
  │         │ │ SQLite  │ │         │ │         │
  │         │ │ 事件表   │ │         │ │ 心跳循环 │
  │         │ │ (含遗忘  │ │         │ │ 主动消息 │
  │         │ │  曲线)   │ │         │ │         │
  │         │ │         │ │         │ │         │
  │         │ │ 观察表   │ │         │ │         │
  │         │ │ 洞察表   │ │         │ │         │
  └─────────┘ └─────────┘ └─────────┘ └─────────┘
```

---

## 4. 成长观察系统（v0.3 核心）

### 4.1 设计理念

人的成长无法用算法精确量化。用文字诚实地描述变化，比用数字假装精确更好。

### 4.2 数据结构

```sql
CREATE TABLE observations (
  id TEXT PRIMARY KEY,
  date TEXT NOT NULL,
  content TEXT NOT NULL,        -- 观察内容，如"你开始更多表达自己的感受"
  category TEXT,                -- "emotion" | "behavior" | "relationship" | "value" | "growth"
  source TEXT,                  -- "reflection" | "event"
  created_at DATETIME
);
```

### 4.3 观察原则

- 基于事实（有事件/对话支撑），不臆测
- 描述变化趋势，不做评判
- 可以不确定（"你好像开始..."）
- 不夸大（没有明显变化就不写）

### 4.4 与事件记忆的关系

事件是具体的"发生了什么"，观察是抽象的"你在怎么变化"。
两者互补：事件提供证据，观察提炼趋势。

---

## 5. 三层记忆架构

### 5.1 Layer 1：核心记忆（始终在 prompt 中）

两个文件，会话开始时注入 system prompt，会话内不变（冻结快照模式）：

| 文件 | 上限 | 内容 | 用途 |
|------|------|------|------|
| `user_profile.md` | 1200 字符 | 用户基本情况、沟通偏好、当前关注、近期状态 | AI 始终了解用户是谁 |
| `companion_notes.md` | 800 字符 | AI 的日记生成上下文笔记（用户关注点、对话模式、生成偏好） | 帮助生成更贴合用户的日记 |

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

## 6. 事件与信息抽取

### 6.1 算法

每轮对话后，用便宜 LLM 从用户消息 + AI 回复中提取结构化信息。

**输入**：用户消息 + AI 回复

**输出**：

```json
{
  "emotions": ["sadness", "anger"],
  "event_type": "conflict",
  "importance": 0.7,
  "summary": "用户与父亲发生激烈争吵，同时感到愤怒和悲伤。",
  "topics": ["和父亲的关系", "情绪管理"]
}
```

### 6.2 约束

- 情绪标签白名单（10 种）：joy, sadness, anger, anxiety, fear, surprise, disgust, calm, overwhelm, hope
- 事件类型（4 种 + null）：conflict, milestone, emotion, decision
- importance 强制 clamp 到 [0.0, 1.0]
- 日常闲聊返回空情绪、null 事件类型、低 importance
- topics 为 AI 从对话中识别的主题标签（最多 3 个）

### 6.3 下游消费

情绪识别结果不直接影响对话风格。它只是沉淀层的数据采集：

- importance ≥ 0.6 → 写入事件表（供后续 Reflection 和周报使用）
- 情绪标签 → 累积到 day_data（供日记生成使用）
- topics → 自动创建/关联主题（topic_links 表）

---

## 7. 主题系统（v0.3 新增）

### 7.1 数据结构

```sql
CREATE TABLE topics (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  first_mentioned TEXT,
  last_mentioned TEXT,
  mention_count INTEGER,
  created_at DATETIME,
  updated_at DATETIME
);

CREATE TABLE topic_links (
  topic_id TEXT NOT NULL,
  item_id TEXT NOT NULL,
  item_type TEXT NOT NULL,      -- "event" | "observation"
  PRIMARY KEY (topic_id, item_id, item_type)
);
```

### 7.2 主题生命周期

1. 事件抽取时 AI 识别 topics → 自动创建不存在的主题 + 建立关联
2. 同一主题积累了 ≥2 条跨时间段的记录后，可生成主题对比
3. 主题对比通过 LLM 串联不同时间点的同类内容，生成变化描述

---

## 8. 对话流程

### 8.1 Prompt 组装

按以下固定顺序拼接：

```
1. 角色定义（固定模板，自然朋友风格）
2. 核心记忆冻结块（≤ 2000 字符）
3. 最近 20 轮对话历史
4. 用户当前消息
```

### 8.2 对话后管线

```
AI 回复完成
  │
  ├→ 保存对话历史（history.json，保留最近 20 轮）
  │
  ├→ 更新对话计数器
  │
  ├→ 情绪识别（便宜 LLM，~200ms）
  │    ├→ importance ≥ 0.6 → 写入 events 表
  │    ├→ topics → 创建/关联主题
  │    └→ 情绪数据 → 累积到 day_data
  │
  ├→ 记录最后活跃时间（last_activity.json，供心跳使用）
  │
  └→ Reflection 检查（每周一次）
       └→ 如果触发：读取周报+事件+日记+已有观察 → LLM 反思 → 生成定性观察
          → 事件归并到项目 → 更新成长线 → 更新 companion_notes
```

### 8.3 流式输出

通过 SSE（Server-Sent Events）实现逐 token 输出：

```
后端：stream_llm() → 逐 token yield
API：POST /api/chat/stream → StreamingResponse
前端：fetch → ReadableStream → 逐字追加到气泡
```

对话后管线在流式输出完成后异步执行，不阻塞用户看到回复。

---

## 9. Reflection Engine（成长观察模块）

### 9.1 触发条件

每周一次（间隔 ≥ 5 天），在对话后管线中检查。

### 9.2 输入

```
1. 近期周报（最近 2 篇）
2. 近期重要事件（最近 10 条，importance ≥ 0.4）
3. 最近 7 天的日记摘要
4. 已有洞察和观察
5. 已有成长线
```

### 9.3 输出

```json
{
  "analysis": "基于沉淀数据的趋势判断",
  "observations": [
    {"content": "一条关于用户的观察描述", "category": "emotion"}
  ],
  "insight": "关于用户的一个洞察",
  "new_insights": [
    {"category": "emotion_pattern", "content": "具体洞察内容", "confidence": 0.5}
  ]
}
```

### 9.4 后续动作

Reflection 完成后自动触发：
1. 保存观察到 observations 表
2. 保存新洞察到 insights 表
3. 更新 companion_notes（加入新洞察）
4. 事件归并（检查新事件能否归入已有项目或发现新项目）
5. 更新成长线（识别变化的成长维度）

### 9.5 观察原则

- 基于事实（有事件/对话支撑），不臆测
- 描述变化趋势，不做评判
- 可以不确定（"你好像开始..."）
- 不夸大（没有明显变化就不写）

---

## 10. 项目档案（v0.3 新增）

### 10.1 数据结构

```sql
CREATE TABLE projects (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT,
  status TEXT DEFAULT 'active',
  start_date TEXT,
  end_date TEXT,
  event_ids TEXT,           -- JSON array
  summary TEXT,
  created_at DATETIME,
  updated_at DATETIME
);
```

### 10.2 触发方式

- Reflection 时自动检查：新事件能否归入已有项目，或发现新项目
- 用户可手动标记/调整

### 10.3 流程

1. Reflection 收集本周新事件
2. LLM 判断哪些事件可归入已有项目，哪些构成新项目
3. 自动创建项目或追加事件到已有项目
4. 积累足够事件后可生成项目总结

---

## 11. 成长线（v0.3 新增）

### 11.1 数据结构

```sql
CREATE TABLE growth_lines (
  id TEXT PRIMARY KEY,
  dimension TEXT NOT NULL,
  records TEXT,              -- JSON array: [{date, note}]
  created_at DATETIME,
  updated_at DATETIME
);
```

### 11.2 触发方式

- Reflection 时，LLM 分析事件趋势，识别可观察的成长维度
- 自动新增或更新成长线记录
- 每条记录只保留 `{date, note}`（定性描述，不量化）

---

## 12. 心跳机制

后台异步循环，让 AI 具备时间感知能力。

### 12.1 心跳循环

每 30 分钟执行一次：

1. **遗忘曲线维护**：衰减所有事件的 strength，清理已遗忘事件
2. **时间感知更新**：记录用户最后活跃时间，感知空闲时长
3. **主动消息检查**：空闲太久时自然地打招呼

### 12.2 主动消息触发条件

| 条件 | 触因 | 消息风格 |
|------|------|---------|
| 空闲 > 24h | long_idle | "好久没聊了，最近怎么样？" |
| 空闲 > 2h | idle | 自然打招呼 |

消息 1-2 句，自然随意，不矫情，不提及 AI 身份。

### 12.3 配置项

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `heartbeat_interval_minutes` | 30 | 心跳间隔 |
| `heartbeat_min_idle_minutes` | 120 | 最短空闲才触发主动消息 |
| `heartbeat_max_idle_minutes` | 1440 | 最长空闲强制签到 (24h) |
| `heartbeat_proactive_enabled` | True | 是否启用主动消息 |

### 12.4 生命周期

心跳任务通过 FastAPI lifespan 管理：应用启动时 `create_task`，关闭时 `cancel`。

---

## 13. 日记系统

### 13.1 数据累积

每轮对话后，将情绪识别结果追加到 `day_data_YYYY-MM-DD.json`：

```json
{
  "entries": [
    {"emotions": ["sadness"], "event_type": "conflict", "importance": 0.7, "summary": "..."},
    {"emotions": [], "event_type": null, "importance": 0.1, "summary": ""}
  ]
}
```

### 13.2 日记生成

**触发**：每日首次对话时检查昨日是否有未生成的日记。

**输入**：day_data + 当日事件记忆

**LLM Prompt 要点**：
- 语气温暖、客观、有洞察力
- 包含"今天你提到了"段落
- 包含"成长观察"段落（发现积极信号）
- 不说教

**输出**：`diaries/YYYY-MM-DD.md`，不覆盖已有文件。

---

## 14. 周成长总结

### 14.1 生成逻辑

**输入**：
- 指定周的事件记忆（importance ≥ 0.3）
- 事件不足 3 条时标注"本周对话较少"

**LLM Prompt 要点**：
- 给这周起一个有温度的标题
- 本周概览（2-3 句话）
- 情绪变化描述
- 重要事件列表
- 成长观察

### 14.2 存储

`summaries/week-YYYY-WNN.md`，已存在时不覆盖。

---

## 15. 人生章节

### 15.1 生成逻辑

**输入**：
- 指定日期范围内的事件记忆
- 可选自定义标题

**LLM Prompt 要点**：
- 像写传记的一个章节
- 包含：这段时光、关键时刻、你在变化、未完待续

### 15.2 存储

`chapters/标题.md`。

---

## 16. 数据存储

```
data/{user_id}/
  ├── user_profile.md              # 核心记忆：用户画像（≤1200 字符）
  ├── companion_notes.md           # 核心记忆：AI 笔记（≤800 字符）
  ├── turn_counter.json            # 对话总轮数
  ├── last_reflection.json         # 上次反思日期
  ├── last_activity.json           # 最后活跃时间（心跳使用）
  ├── history.json                 # 最近 20 轮对话历史
  ├── events.db                    # SQLite
  │   ├── events                   # 事件记忆表（含遗忘曲线字段）
  │   ├── observations             # 成长观察表
  │   ├── topics                   # 主题表
  │   ├── topic_links              # 主题关联表（事件/观察）
  │   ├── projects                 # 项目档案表
  │   ├── growth_lines             # 成长线表
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

## 17. API 端点

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/init` | 用户初始化（偏好 → 画像） |
| POST | `/api/chat` | 非流式聊天 |
| POST | `/api/chat/stream` | 流式聊天（SSE） |
| GET | `/api/heartbeat` | 前端轮询：主动消息 |
| GET | `/api/memory/core` | 读取核心记忆 |
| PATCH | `/api/memory/core` | 编辑核心记忆（add/replace/remove） |
| GET | `/api/memory/events` | 查询事件记忆（支持 min_strength） |
| DELETE | `/api/memory/events/{id}` | 删除事件 |
| POST | `/api/memory/events/maintain` | 手动触发遗忘曲线衰减和清理 |
| GET | `/api/memory/search` | 全文搜索对话记录（FTS5） |
| GET | `/api/memory/insights` | 查询结构化洞察 |
| GET | `/api/observations` | 成长观察列表 |
| GET | `/api/observations/{category}` | 按类别查看观察 |
| GET | `/api/topics` | 主题列表 |
| GET | `/api/topics/{id}` | 主题详情 + 关联事件/观察 |
| GET | `/api/topics/{id}/compare` | 主题对比 |
| GET | `/api/projects` | 项目档案列表 |
| GET | `/api/projects/{id}` | 查看项目详情 |
| POST | `/api/projects/generate` | 手动触发项目归并 |
| PATCH | `/api/projects/{id}` | 编辑项目 |
| GET | `/api/growth-lines` | 成长线列表 |
| GET | `/api/growth-lines/{dimension}` | 查看某维度成长线 |
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

## 18. 技术栈

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

## 19. 设计权衡

### 为什么用文件存核心记忆而不是数据库？

核心记忆需要注入 system prompt，文件可以直接读取拼接，零延迟。硬上限（1200 + 800 字符）逼迫 AI 只保留最重要的信息，避免无限膨胀。

### 为什么不用向量数据库？

单用户 MVP 场景，事件量小（每天几条到几十条）。结构化查询（按 importance、日期、类型过滤）已经足够。如果后续需要语义搜索，再加 pgvector。

### 为什么不用多 Agent？

MVP 阶段增加调度复杂度且拉高 token 消耗。单模型 + 状态机足以验证核心假设。

### 为什么成长线不用数字量化？

人的成长无法用算法精确量化。数字看起来严谨，但实际上只是 LLM 的主观猜测。用自然语言描述变化更诚实。

### 为什么用 topic_links 关联表而不是 events.topic_id？

一个事件可能涉及多个主题（如"因为考研压力和女朋友吵架"涉及"考研"和"恋爱关系"）。多对多关联表更灵活。

### 为什么加遗忘曲线？

无限积累的事件记忆会导致：查询变慢、噪音事件淹没重要事件、不符合人类记忆的衰减特性。遗忘曲线让重要事件（高 importance + 被反复回忆）保留更久，日常琐事自然消退。

### 为什么心跳用轮询而不是 WebSocket？

MVP 阶段前端是单 HTML，用 fetch 轮询最简单。心跳间隔 30 分钟，轮询频率远低于间隔，几乎不影响性能。

---

## 20. 已移除的系统

以下系统在 v0.3 中被移除，此处仅记录历史决策：

### 八维权重系统（v0.3 移除）

用 8 个浮点数（Ti/Te/Fi/Fe/Si/Se/Ni/Ne）表示 AI 人格，每周 Reflection 调整 ±0.02。
移除原因：量化成长是虚假的精确，改为定性观察。

### PAD 情感模型（v0.3 移除）

基于 Mehrabian 的三维情感状态（Pleasure/Arousal/Dominance），影响 AI 回复风格。
移除原因：产品定位为日记工具而非情感陪伴，AI 不需要自身的情感状态。

### Compensation 临时补偿（v0.3 移除）

用户情绪崩溃时临时调整八维权重（仅当轮生效）。
移除原因：八维权重移除后，临时补偿机制不再适用。

### Reinforcement 逐轮强化（v2 移除）

每轮对话后微调八维权重。
移除原因：逐轮强化导致人格波动过大。
