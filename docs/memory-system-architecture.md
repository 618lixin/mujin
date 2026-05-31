# 记忆系统架构文档

> 版本: v1.1（2026-05-31）
> 覆盖: Tauri/Rust 端记忆系统的完整设计与实现
> 上一版本: v1.0 — 修复多处文档与实现不一致

---

## 1. 概述

记忆系统是 Growth Companion 的核心基础设施，负责将用户的对话和笔记**结构化存储**、**选择性遗忘**、并在需要时**检索聚合**输出为日记或对话上下文。

### 设计哲学

- **低成本提取**：用 cheap LLM 做情绪和事件提取，不阻塞主流程
- **结构化存储**：SQLite 存事件/对话/主题/观察/洞察/项目/成长线，Markdown 存用户画像
- **选择性遗忘**：遗忘曲线自动衰减事件强度，模拟真实记忆
- **聚合输出**：日记生成和对话上下文都从记忆系统中聚合素材

---

## 2. 分层架构

```
┌─────────────────────────────────────────────┐
│                 记忆系统                      │
│                                               │
│  ┌──────────────┐    ┌────────────────────┐  │
│  │  核心记忆     │    │   事件记忆          │  │
│  │  (Markdown)   │    │   (SQLite)          │  │
│  │               │    │                     │  │
│  │ user_profile  │    │ events 表           │  │
│  │ companion_    │    │ conversation_turns  │  │
│  │   notes.md    │    │ turn_search (FTS5)  │  │
│  │               │    │ topics / topic_links│  │
│  │ history.json  │    │ projects            │  │
│  │ turn_counter  │    │ observations        │  │
│  │ last_activity │    │ insights            │  │
│  └──────────────┘    │ growth_lines        │  │
│                       └────────────────────┘  │
│                                               │
│  ┌──────────────────────────────────────────┐ │
│  │           维护机制                        │ │
│  │  遗忘曲线衰减 · 记忆清理 · 心跳触发       │ │
│  └──────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

---

## 3. 核心记忆（Markdown 文件）

### 3.1 存储位置

```
data/{user_id}/
  ├── user_profile.md       # 用户画像
  ├── companion_notes.md    # AI 观察笔记
  ├── history.json          # 对话历史
  ├── turn_counter.json     # 对话轮次计数
  ├── last_activity.json    # 最后活跃时间
  └── events.db             # SQLite 事件数据库
```

### 3.2 文件说明

| 文件 | 内容 | 管理方式 | 参与场景 |
|------|------|---------|---------|
| `user_profile.md` | 用户画像：性格、偏好、背景 | Memory 面板手动编辑 | 对话 prompt、日记 prompt |
| `companion_notes.md` | AI 对用户的观察和笔记 | Phase 4+ 自动更新（当前为 TODO） | 对话 prompt、日记 prompt |
| `history.json` | 最近 N 轮对话 JSON 数组 | `save_history()` 自动维护 | 对话上下文 |
| `turn_counter.json` | 累计对话轮次计数 | `save_turn_counter()` 自动递增 | 统计 |
| `last_activity.json` | 最后活跃 UTC 时间戳 | `save_last_activity()` 更新 | 心跳空闲检测 |

### 3.3 接口

```rust
// 加载核心记忆（同时读取 profile + notes）
load_core_memory(base_dir, user_id, config) -> CoreMemory

// 读取/更新 user_profile
get_core_memory(base_dir, user_id) -> CoreMemoryResponse
patch_core_memory(base_dir, user_id, patch) -> CoreMemoryResponse

// 格式化记忆用于 prompt 注入
format_memory_for_prompt(core_memory) -> String

// 对话历史管理
load_history(base_dir, user_id, max_turns) -> Vec<ChatMessage>
save_history(base_dir, user_id, messages, max_turns) -> ()

// 计数器与活跃时间
load_turn_counter(base_dir, user_id) -> u32
save_turn_counter(base_dir, user_id, count) -> ()
save_last_activity(base_dir, user_id) -> ()
load_last_activity(base_dir, user_id) -> Option<String>
```

---

## 4. 事件记忆（SQLite 数据库）

### 4.1 数据表结构（共 9 张表）

```sql
-- 事件表：核心记忆单元
CREATE TABLE IF NOT EXISTS events (
    id          TEXT PRIMARY KEY,
    content     TEXT NOT NULL,          -- 事件内容摘要
    emotions    TEXT DEFAULT '[]',      -- JSON 数组：情绪标签
    importance  REAL DEFAULT 0.5,       -- 重要性 0.0~1.0
    event_type  TEXT,                   -- conflict/milestone/emotion/decision
    strength    REAL DEFAULT 1.0,       -- 当前记忆强度
    stability   REAL DEFAULT 30.0,      -- 遗忘稳定性（越高越抗遗忘）
    recall_count INTEGER DEFAULT 0,    -- 被回忆次数
    last_recalled_at TEXT,             -- 上次被回忆时间
    created_at  TEXT NOT NULL,         -- 创建时间（UTC ISO 8601）
    updated_at  TEXT NOT NULL          -- 更新时间
);

-- 对话轮次表：原始对话全文存储
CREATE TABLE IF NOT EXISTS conversation_turns (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_msg    TEXT NOT NULL,          -- 用户原始消息
    ai_msg      TEXT NOT NULL,          -- AI 回复
    summary     TEXT DEFAULT '',        -- 情绪摘要
    emotions    TEXT DEFAULT '[]',      -- JSON 数组
    created_at  TEXT NOT NULL           -- 创建时间
);

-- FTS5 全文搜索虚拟表：索引对话内容
CREATE VIRTUAL TABLE IF NOT EXISTS turn_search USING fts5(
    summary,
    user_msg
);

-- 洞察表：LLM 推导的心理洞察
CREATE TABLE IF NOT EXISTS insights (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    category    TEXT NOT NULL,          -- 洞察分类
    content     TEXT NOT NULL,          -- 洞察内容
    confidence  REAL DEFAULT 0.5,       -- 置信度
    source      TEXT,                   -- 来源说明
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

-- 定性观察表
CREATE TABLE IF NOT EXISTS observations (
    id          TEXT PRIMARY KEY,
    date        TEXT NOT NULL,          -- 观察日期
    content     TEXT NOT NULL,          -- 观察内容
    category    TEXT,                   -- 观察分类
    source      TEXT,                   -- 来源说明
    created_at  TEXT NOT NULL
);

-- 主题表
CREATE TABLE IF NOT EXISTS topics (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,      -- 注意：无 UNIQUE 约束，应用层保证唯一
    description     TEXT DEFAULT '',
    first_mentioned TEXT,
    last_mentioned  TEXT,
    mention_count   INTEGER DEFAULT 1,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

-- 主题关联表：多对多连接
CREATE TABLE IF NOT EXISTS topic_links (
    topic_id    TEXT NOT NULL,
    item_id     TEXT NOT NULL,          -- event_id 或 project_id
    item_type   TEXT NOT NULL,          -- 'event' | 'project'
    PRIMARY KEY (topic_id, item_id, item_type)
);

-- 项目档案表
CREATE TABLE IF NOT EXISTS projects (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    description TEXT DEFAULT '',
    status      TEXT DEFAULT 'active',  -- active / completed / paused
    start_date  TEXT,
    end_date    TEXT,
    event_ids   TEXT DEFAULT '[]',      -- JSON 数组：关联事件 ID
    summary     TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

-- 成长线表
CREATE TABLE IF NOT EXISTS growth_lines (
    id          TEXT PRIMARY KEY,
    dimension   TEXT NOT NULL,          -- 成长维度名称
    records     TEXT DEFAULT '[]',      -- JSON 数组：里程碑记录
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
```

### 4.2 为什么 topics.name 没有 UNIQUE 约束？

`get_topic_by_name()` 在应用层保证唯一性：查询时取第一条匹配记录，创建前先检查是否存在。DDL 层面不使用 UNIQUE 是为了避免并发写入时的 `UNIQUE constraint failed` 错误中断整个 post_chat 流程。

### 4.3 关键接口

```rust
// 事件 CRUD
add_event(user_id, event, base_stability) -> ()
query_events(user_id, params) -> Vec<Event>
delete_event(user_id, id) -> bool
record_recall(user_id, event_id, recall_boost) -> ()   // 回忆增强
query_events_by_date(user_id, utc_start, utc_end, limit) -> Vec<Event>

// 对话轮次
save_conversation_turn(user_id, user_msg, ai_msg, summary, emotions) -> ()
query_conversation_turns_by_date(user_id, utc_start, utc_end, limit)
    -> Vec<ConversationTurn>
search_conversations(user_id, query, limit)  // FTS5 MATCH 全文搜索

// 洞察 (insights)
save_insight(user_id, category, content, confidence, source) -> ()
get_insights(user_id, category?, limit) -> Vec<Insight>

// 观察 (observations)
add_observation(user_id, obs) -> ()
query_observations(user_id, category?, limit) -> Vec<Observation>

// 主题 (topics)
add_topic(user_id, topic) -> ()
query_topics(user_id, limit) -> Vec<Topic>
get_topic_by_name(user_id, name) -> Option<Topic>
update_topic(user_id, topic_id, last_mentioned?, mention_count?, description?) -> ()
link_topic(user_id, topic_id, item_id, item_type) -> ()
get_topic_links(user_id, topic_id) -> Vec<TopicLink>

// 项目 (projects)
add_project(user_id, project) -> ()
query_projects(user_id, status?, limit) -> Vec<Project>
get_project(user_id, project_id) -> Option<Project>
update_project(user_id, project_id, ...) -> ()

// 成长线 (growth_lines)
add_growth_line(user_id, id, dimension) -> ()
query_growth_lines(user_id, limit) -> Vec<GrowthLine>
get_growth_line(user_id, dimension) -> Option<GrowthLine>
update_growth_line_records(user_id, gl_id, records) -> ()
delete_growth_line(user_id, gl_id) -> bool

// 维护操作
decay_all_events(user_id, min_strength) -> usize    // 返回低于阈值的数量
cleanup_forgotten_events(user_id, min_strength)     // 删除过期事件
```

---

## 5. 数据入站通道

### 5.1 对话提取

```
用户发消息 → AI 回复 → post_chat()
                          │
                    extract_emotion()
                          │
                    cheap LLM 分析
                          │
                    EmotionResult {
                      emotions: ["anxiety", "hope"],
                      event_type: "milestone",
                      importance: 0.75,
                      summary: "用户今天参加了面试",
                      topics: ["职业选择"]
                    }
                          │
                    ┌─────┴─────┐
                    │ importance │
                    │   ≥ 0.6   │
                    │  + eventType│
                    └─────┬─────┘
                      YES  │  NO
              ┌────────────┴──────────┐
              │ 写入 events 表         │  仅写 conversation_turns
              │ 创建/更新 topics       │  + FTS5 turn_search
              │ 写入 topic_links       │  (不入事件，不创建主题)
              └───────────────────────┘
```

### 5.2 笔记提取

```
快捷便签(NotePad) / 主编辑器(MainWindow)
          │
    保存笔记时触发
          │
    quickExtract(note_id, title, content)
          │
    extract_emotion() → 同上流程
```

三个入口：
| 入口 | 触发时机 | 阻塞 UI |
|------|---------|:---:|
| AI 对话 | 每次对话完成（post_chat） | ❌ 异步 |
| 快捷便签 NotePad | Ctrl+S 保存 | ❌ 异步 |
| 主编辑器 MainWindow | Ctrl+S 保存 | ❌ fire-and-forget |

### 5.3 post_chat 完整流水线

```
post_chat() 执行 7 步：
  1. save_history()           → 追加到 history.json
  2. increment turn_counter   → turn_counter.json +1
  3. extract_emotion()        → cheap LLM 分析情绪
  4. save_conversation_turn() → conversation_turns + turn_search (FTS5)
  5. 如果 important: add_event()     → events 表
  6. 如果 important: link_topics()   → topics + topic_links
  7. save_last_activity()     → last_activity.json
```

---

## 6. 遗忘曲线

### 6.1 公式

```
stability = forget_base_stability × (0.5 + importance)

Δt = 自上次衰减到现在的时间间隔（心跳周期）

decay_rate = e^(-Δt / stability)

new_strength = old_strength × decay_rate
```

### 6.2 设计意图

| importance | stability | 半衰期（约） | 事件类型 |
|-----------|-----------|------------|---------|
| 0.9 | 42h | ~29h | 人生重大变化 |
| 0.75 | 37.5h | ~26h | 重要事件（面试/离职） |
| 0.5 | 30h | ~21h | 有情绪但非重大 |
| 0.3 | 24h | ~17h | 日常记录 |

越不重要的事件遗忘越快，模拟真实记忆规律。

### 6.3 回忆增强

当事件被检索并用于对话上下文时，`record_recall()` 提升其稳定性：

```
new_stability = stability × (1 + recall_boost)
recall_count += 1
last_recalled_at = now
```

### 6.4 清理

每次心跳执行：
```rust
decay_all_events(user_id, forget_min_strength)
cleanup_forgotten_events(user_id, forget_min_strength)
```

`strength < forget_min_strength`（默认 0.05）的事件被永久删除。

---

## 7. 记忆检索

### 7.1 对话上下文检索

每次用户发消息时，`chat_stream_start` → `prepare_chat` 构建 system prompt：

```
1. query_events_by_date()       → 当天事件（带数量上限）
2. load_core_memory()           → user_profile + companion_notes
3. load_history()               → 最近 N 轮对话历史

        ↓ 拼装为 system prompt ↓

  你是一个温暖的 AI 朋友...

  --- 用户背景 ---
  {user_profile + companion_notes}

  --- 今天的记忆 ---
  {当天事件列表}
```

> **注意**：Phase 4+ 计划中的语义记忆检索（FTS5 搜索历史对话 + 根据用户当前消息检索相关事件）尚未实现。
> `prepare_chat` 中 `retrieved_memories` 当前为 `None`，待后续开发。

### 7.2 日记生成素材检索

日记生成时聚合四路素材（详见 `diary.rs`）：

| 数据源 | 获取方式 | 上限 |
|--------|---------|------|
| 当天事件 | `query_events_by_date()` | 100 条 |
| 当天对话 | `query_conversation_turns_by_date()` | 200 条 |
| 当天笔记 | `query_notes_by_date()` (NoteStore) | 全部（含 800 字/篇截断） |
| 核心记忆 | `load_core_memory()` | 全量 |

---

## 8. 心跳维护周期

```rust
start_heartbeat(app, interval_minutes)
```

每 `interval_minutes` 分钟执行一次：

```
heartbeat_tick()
  │
  ├─ 遍历所有用户目录（通过检测 user_profile.md 发现）
  │
  ├─ 1. decay_all_events()     → 所有事件强度衰减
  ├─ 2. cleanup_forgotten()    → 删除 strength < forget_min_strength 的事件
  ├─ 3. 检查用户空闲时长
  │      ├─ 空闲 < min_idle   → 跳过
  │      ├─ min_idle ≤ 空闲 < max_idle → 温和问候
  │      └─ 空闲 ≥ max_idle   → 关切问候
  │
  └─ 4. 如果有待发消息 → 推送到 Chat 面板（proactive-message 事件）
```

可通过设置面板配置：
- `heartbeatProactiveEnabled`：是否启用主动问候
- `heartbeatIntervalMinutes`：心跳间隔

---

## 9. 前端展示

### MemoryPanel

| Tab | 数据源 | 操作 |
|-----|-------|------|
| 核心记忆 | `user_profile.md` + `companion_notes.md` | 编辑 + 保存 |
| 事件列表 | `events` 表 | 查看、删除 |
| 主题 | `topics` 表 | 查看关联事件 |

### GrowthPanel

| 区块 | 数据源 |
|------|-------|
| 定性观察 | `observations` 表 |
| 主题概览 | `topics` 表 |
| 项目档案 | `projects` 表 |
| 成长线 | `growth_lines` 表 |

### ChatPanel

| 交互反馈 | 数据来源 |
|---------|---------|
| 事件已保存 (✅) | `post_chat` 返回的 `EmotionResult`（importance ≥ 0.6 且有 eventType） |
| 主动问候消息 | 心跳生成的 `proactive-message` 事件 |

### 日记生成反馈

MainWindow 工具栏中「生成日记」按钮执行后，显示四路素材统计：
| 指标 | 来源 |
|------|------|
| sourceEventCount | `query_events_by_date()` 返回条数 |
| sourceTurnCount | `query_conversation_turns_by_date()` 返回条数 |
| sourceNoteCount | `query_notes_by_date()` 返回条数 |

---

## 10. 相关文件索引

| 文件 | 职责 |
|------|------|
| `frontend/src-tauri/src/services/database.rs` | SQLite 数据库层（~1570 行）— 9 张表的 DDL、CRUD、FTS5 |
| `frontend/src-tauri/src/services/types.rs` | Rust 类型定义（~440 行）— Event, Topic, Project 等 struct |
| `frontend/src-tauri/src/services/memory.rs` | 核心记忆读写 + 格式化 + history 管理 |
| `frontend/src-tauri/src/services/extractor.rs` | 情绪/事件提取 + quickExtract |
| `frontend/src-tauri/src/services/chat.rs` | 对话流程编排（含 post_chat 7 步流水线） |
| `frontend/src-tauri/src/services/scheduler.rs` | 心跳调度器（衰减 + 清理 + 主动问候） |
| `frontend/src-tauri/src/services/diary.rs` | 日记生成（聚合四路记忆素材） |
| `frontend/src-tauri/src/services/notes.rs` | 笔记存储（NoteStore）— 日记的持久化载体 |
| `frontend/src/components/panels/MemoryPanel.tsx` | 记忆面板 UI |
| `frontend/src/components/panels/GrowthPanel.tsx` | 成长面板 UI |
| `frontend/src/components/panels/ChatPanel.tsx` | 对话面板 UI（含事件保存反馈） |
| `frontend/src/components/MainWindow.tsx` | 主窗口（含日记生成按钮 + 笔记事件提取） |
| `frontend/src/features/api/memory.ts` | 前端记忆 API 封装 |
| `frontend/src/features/api/types.ts` | 前端类型定义 |

---

## 11. 已知限制与待实现功能

### 11.1 Phase 4+ 待实现

| 功能 | 当前状态 | 位置 |
|------|---------|------|
| 语义记忆检索 | ❌ `retrieved_memories = None` | `chat.rs:prepare_chat` |
| Reflection 检查 | ❌ `// TODO: Reflection check` | `chat.rs:post_chat` |
| Notes 自动更新 | ❌ `// TODO: Notes auto-update` | `chat.rs:post_chat` |

### 11.2 闲置数据表

以下表有完整 CRUD 但尚无自动写入 pipeline：

| 表 | 手动 CRUD | 自动写入 |
|-----|:---:|:---:|
| `insights` | ✅ | ❌ — 从未在 pipeline 中使用 |
| `observations` | ✅ | ❌ — 需设计自动观察生成 |
| `projects` | ✅ | ❌ — 需设计项目自动识别 |
| `growth_lines` | ✅ | ❌ — 需设计成长里程碑提取 |

### 11.3 性能注意事项

- 6 个关键列缺少索引（`events.created_at`、`conversation_turns.created_at` 等），数据量小 (<10 万条) 时无影响
- 心跳周期中 `compute_strength` 被调用两次（`decay_all_events` + `cleanup_forgotten_events`）
- `find_existing_diary()` 使用 O(n) 遍历而非定向查询
- `conn_with_schema()` 每次查询都执行 schema DDL（idempotent 但多余）
