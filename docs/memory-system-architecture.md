# 记忆系统架构文档

> 版本: v1.0（2026-05-31）
> 覆盖: Tauri/Rust 端记忆系统的完整设计与实现

---

## 1. 概述

记忆系统是 Growth Companion 的核心基础设施，负责将用户的对话和笔记**结构化存储**、**选择性遗忘**、并在需要时**检索聚合**输出为日记或对话上下文。

### 设计哲学

- **低成本提取**：用 cheap LLM 做情绪和事件提取，不阻塞主流程
- **结构化存储**：SQLite 存事件/对话/主题/观察，Markdown 存用户画像
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
│  │   notes.md    │    │ topics / projects   │  │
│  └──────────────┘    │ observations        │  │
│                       │ growth_lines        │  │
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
  ├── history/              # 对话历史
  │   └── sessions/
  ├── counters/
  │   └── turn_counter.json
  ├── activity/
  │   └── last_active.json
  └── events.db             # SQLite 事件数据库
```

### 3.2 文件说明

| 文件 | 内容 | 管理方式 | 参与场景 |
|------|------|---------|---------|
| `user_profile.md` | 用户画像：性格、偏好、背景 | Memory 面板手动编辑 | 对话 prompt、日记 prompt |
| `companion_notes.md` | AI 对用户的观察和笔记 | Phase 4+ 自动更新 | 对话 prompt、日记 prompt |

### 3.3 接口

```rust
// 加载核心记忆（同时读取 profile + notes）
load_core_memory(base_dir, user_id, config) -> CoreMemory

// 读取/更新 user_profile
get_core_memory(base_dir, user_id) -> CoreMemoryResponse
patch_core_memory(base_dir, user_id, patch) -> CoreMemoryResponse

// 格式化记忆用于 prompt 注入
format_memory_for_prompt(core_memory) -> String
```

---

## 4. 事件记忆（SQLite 数据库）

### 4.1 数据表结构

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

-- 对话轮次表：全文搜索
CREATE TABLE IF NOT EXISTS conversation_turns (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_msg    TEXT NOT NULL,          -- 用户原始消息
    ai_reply    TEXT NOT NULL,          -- AI 回复
    summary     TEXT DEFAULT '',        -- 情绪摘要
    emotions    TEXT DEFAULT '[]',      -- JSON 数组
    created_at  TEXT NOT NULL
);

-- 主题表
CREATE TABLE IF NOT EXISTS topics (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,
    description     TEXT DEFAULT '',
    first_mentioned TEXT,
    last_mentioned  TEXT,
    mention_count   INTEGER DEFAULT 0
);

-- 主题关联表
CREATE TABLE IF NOT EXISTS topic_links (
    topic_id    TEXT NOT NULL,
    target_id   TEXT NOT NULL,          -- event_id 或 project_id
    target_type TEXT NOT NULL,          -- 'event' | 'project'
    PRIMARY KEY (topic_id, target_id)
);

-- 项目档案表
CREATE TABLE IF NOT EXISTS projects (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT DEFAULT '',
    status      TEXT DEFAULT 'active',
    started_at  TEXT,
    ended_at    TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

-- 定性观察表
CREATE TABLE IF NOT EXISTS observations (
    id          TEXT PRIMARY KEY,
    category    TEXT NOT NULL,          -- 观察分类
    content     TEXT NOT NULL,          -- 观察内容
    confidence  REAL DEFAULT 0.5,
    source      TEXT,                   -- 来源（event_id / turn_id）
    created_at  TEXT NOT NULL
);

-- 成长线表
CREATE TABLE IF NOT EXISTS growth_lines (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT DEFAULT '',
    milestones  TEXT DEFAULT '[]',      -- JSON 数组
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
```

### 4.2 关键接口

```rust
// 事件 CRUD
add_event(user_id, event, base_stability)
query_events(user_id, params) -> Vec<Event>
delete_event(user_id, id)
query_events_by_date(user_id, utc_start, utc_end, limit) -> Vec<Event>

// 对话轮次
save_conversation_turn(user_id, user_msg, ai_reply, summary, emotions)
query_conversation_turns_by_date(user_id, utc_start, utc_end, limit)
    -> Vec<ConversationTurn>
search_conversations(user_id, query, limit)  // FTS5 全文搜索

// 维护操作
decay_all_events(user_id, min_strength) -> usize    // 衰减
cleanup_forgotten_events(user_id, min_strength)     // 清理
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
              │ 创建/更新 topics       │  (不入事件，不创建主题)
              │ 写入 topic_links       │
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
| AI 对话 | 每次对话完成 | ❌ 异步 |
| 快捷便签 NotePad | Ctrl+S 保存 | ❌ 异步 |
| 主编辑器 MainWindow | Ctrl+S 保存 | ❌ fire-and-forget |

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

### 6.3 清理

每次心跳执行：
```rust
decay_all_events(user_id, forget_min_strength)
cleanup_forgotten_events(user_id, forget_min_strength)
```

`strength < forget_min_strength` 的事件被永久删除。

---

## 7. 记忆检索

### 7.1 对话上下文检索

每次用户发消息时，`chat_stream_start` → `prepare_chat` 会检索：

```
1. query_events_by_date()       → 当天事件（带数量上限）
2. FTS5 搜索 conversation_turns  → 语义相关历史对话
3. load_core_memory()           → user_profile + companion_notes
4. load_history()               → 最近 N 轮对话历史

        ↓ 拼装为 system prompt ↓

  你是一个温暖的 AI 朋友...
  
  --- 用户背景 ---
  {user_profile + companion_notes}
  
  --- 今天的记忆 ---
  {当天事件列表}
  
  --- 相关历史 ---
  {FTS5 搜索结果}
```

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
  ├─ 1. decay_all_events()     → 所有事件强度衰减
  ├─ 2. cleanup_forgotten()    → 删除过期事件
  ├─ 3. 检查用户空闲时长
  │      ├─ 空闲 < min_idle   → 跳过
  │      ├─ min_idle ≤ 空闲 < max_idle → 温和问候
  │      └─ 空闲 ≥ max_idle   → 关切问候
  │
  └─ 4. 如果有待发消息 → 推送到 Chat 面板
```

可通过设置面板配置：
- `heartbeatProactiveEnabled`：是否启用主动问候
- `heartbeatIntervalMinutes`：心跳间隔

---

## 9. 前端展示

### MemoryPanel

| Tab | 数据源 | 操作 |
|-----|-------|------|
| 核心记忆 | `user_profile.md` | 编辑 + 保存 |
| 事件列表 | `events` 表 | 查看、删除 |
| 主题 | `topics` 表 | 查看关联事件 |

### GrowthPanel

| 区块 | 数据源 |
|------|-------|
| 定性观察 | `observations` 表 |
| 主题概览 | `topics` 表 |
| 项目档案 | `projects` 表 |
| 成长线 | `growth_lines` 表 |

---

## 10. 相关文件索引

| 文件 | 职责 |
|------|------|
| `frontend/src-tauri/src/services/database.rs` | SQLite 数据库层（~1100 行） |
| `frontend/src-tauri/src/services/memory.rs` | 核心记忆读写 + 格式化 |
| `frontend/src-tauri/src/services/extractor.rs` | 情绪/事件提取 + quickExtract |
| `frontend/src-tauri/src/services/chat.rs` | 对话流程编排（含 post_chat） |
| `frontend/src-tauri/src/services/scheduler.rs` | 心跳调度器 |
| `frontend/src-tauri/src/services/diary.rs` | 日记生成（聚合记忆素材） |
| `frontend/src/components/panels/MemoryPanel.tsx` | 记忆面板 UI |
| `frontend/src/components/panels/GrowthPanel.tsx` | 成长面板 UI |
| `frontend/src/features/api/memory.ts` | 前端记忆 API 封装 |
