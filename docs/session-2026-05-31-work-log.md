# 会话工作记录：2026-05-31

> 分支：`feat/phase7-diary-ui-improvements`
> 对比 main 分支：11 个文件，+1065 行，-27 行，8 个提交

---

## 概览

本次会话围绕六个主题展开：项目整理 → 日记 UI 改善 → 记忆系统审查 → 文档对齐 → 记忆检索实现。所有修改均在本分支内，测试全绿（104 Rust + 58 前端）。

---

## 一、清理 Python 遗留代码

**问题**：项目已迁移到 Tauri 2 + Rust + React 19，但旧 Python 后端的 30+ 个 `.py` 文件、`requirements.txt`、`static/index.html` 仍留在仓库。

**操作**：
- `git rm -r src/` 删除整个 Python 后端（~30 个 .py 文件）
- 删除 `requirements.txt`、`data/`、两个散落的 .txt 文件
- 更新 `CLAUDE.md` 和 `AGENTS.md` 中的技术栈描述和环境命令
- 清理 `.gitignore` 中的 Python 相关规则（`__pycache__`、`*.py[cod]`、`.venv`）

---

## 二、日记生成 UI 改善

### 2.1 「生成今日日记」按钮（2 轮迭代）

**第一轮**：在日记面板空状态区添加大按钮。但用户反馈看不到——原因是 `bootstrap()` 自动选中第一条笔记，空状态永远不会出现。

**第二轮**：在工具栏添加文字标签按钮「✒ 生成日记」，无论是否选中笔记都可见。点击时显示「生成中…」loading 状态。

涉及文件：
- `MainWindow.tsx`：空状态 CTA + 工具栏按钮 + 生成逻辑
- 三个 locale 文件（zh-CN / zh-HK / en-US）：新增 `emptyGenerateDiary` / `generatingDiary` 翻译

### 2.2 用户笔记作为第四路日记素材

**问题**：日记生成的数据源只有事件、对话、核心记忆三路，缺少用户自己的笔记。

**实现**：
- `diary.rs` 新增 `query_notes_by_date()`：按本地日期过滤 NoteStore 笔记，排除已有日记
- `diary.rs` 新增 `format_notes_for_prompt()`：格式化笔记，800 字/篇截断
- `build_diary_prompt()` 新增 `notes` 参数
- `DiaryGenerateResult` 新增 `source_note_count` 字段
- Rust + TypeScript 类型同步更新

### 2.3 用户反馈系统

**对话事件保存反馈**：
- `ChatPanel.tsx` 新增 `eventCreated?: boolean` 字段
- 仅当 `importance ≥ 0.6 且 eventType 存在` 时，AI 消息下方显示绿色「✅ 已保存到记忆」条
- 普通对话无反馈（用户明确要求："不需要每条对话就有一次反馈"）

**日记生成素材统计**：
- `MainWindow.tsx`：生成完成后标题栏显示素材来源统计条
- 展示 `sourceEventCount` / `sourceTurnCount` / `sourceNoteCount`
- 5 秒后自动消失

### 2.4 主编辑器自动提取事件

**问题**：只有快捷便签 NotePad 在保存时提取事件，主编辑器 MainWindow 保存的笔记不参与记忆系统。

**实现**：`MainWindow.tsx` 的 `saveCurrentNote()` 末尾添加 fire-and-forget `quickExtract()`，不阻塞保存流程，提取失败静默忽略。

---

## 三、记忆系统全面审查

对 7 个核心文件进行了深度审计：

| 文件 | 行数 | 审查要点 |
|------|------|---------|
| `database.rs` | 1571 | 9 张表 DDL、CRUD、FTS5、索引 |
| `memory.rs` | 501 | 核心记忆读写、文件结构 |
| `extractor.rs` | 392 | LLM 提取流水线、事件阈值 |
| `chat.rs` | 436 | post_chat 7 步管线 |
| `scheduler.rs` | 283 | 心跳衰减逻辑 |
| `diary.rs` | 792 | 四路素材聚合 |

### 审计发现

| 类别 | 问题数 | 严重性 |
|------|--------|--------|
| 文档与实现不一致 | 10 处 | P0 |
| Phase 4+ 功能空壳 | 3 项 | P1 |
| 闲置数据表 | 4 张 | P2 |
| 性能问题（缺索引等） | 6 处 | P3 |

---

## 四、P0：架构文档对齐实现

修复 `docs/memory-system-architecture.md` 的 10 处文档-实现差异：

| 位置 | 旧文档 | 实际实现 |
|------|--------|---------|
| 目录结构 | 嵌套子目录 `history/sessions/` | 扁平文件 `history.json` 等 |
| `conversation_turns` | `ai_reply` 列 | `ai_msg` 列 |
| 缺失表 | 无 | 补充 `turn_search`（FTS5）、`insights` |
| `observations` | 有 `confidence`，无 `date` | 无 `confidence`，有 `date` |
| `topics` | `name UNIQUE` | 无 UNIQUE（应用层保证） |
| `topic_links` | `target_id` / `target_type` | `item_id` / `item_type` |
| `projects` | `name` / `started_at` | `title` / `start_date` / `status` 等 |
| `growth_lines` | `name` / `description` / `milestones` | `dimension` / `records` |
| 记忆检索 | 描述为已实现 | 标注为 Phase 4+ TODO |
| 新增 §11 | 无 | 已知限制 + 闲置表 + 性能注意事项 |

同时补充了完整的 Rust 接口列表和文件索引。

---

## 五、P1：实现记忆检索（Phase 4+）

### 5.1 问题

`prepare_chat()` 中 `retrieved_memories` 一直是 `None`。聊天 system prompt 只有用户画像 + 当天事件，无法利用历史记忆。

### 5.2 实现

双路检索 + system prompt 注入：

```
用户发消息 → prepare_chat()
              │
              └─ retrieve_memories()
                   │
                   ├─ LIKE 搜索 conversation_turns
                   │   (取消息前 20 字，匹配 summary / user_msg，上限 5 条)
                   │
                   ├─ 查询近 30 天重要事件
                   │   (importance ≥ 0.5，上限 5 条)
                   │
                   └─ 格式化为 prompt 块:
                      --- 相关历史记忆 ---
                      过往相关事件：
                      - [2026-05-25] 🏔 用户参加了面试 [anxiety, hope]
                      
                      相关过往对话：
                      - [2026-05-20] 工作压力讨论
                      ---
```

### 5.3 为什么用 LIKE 而不是 FTS5

FTS5 的 `unicode61` 分词器将连续 CJK 字符视为一个 token。「最近工作压力很大」→ 一整块。搜索「工作压力」找不到。

`LIKE '%工作压力%'` 对所有语言都能做子串匹配。已通过集成测试验证。

### 5.4 新增代码

| 文件 | 新增 |
|------|------|
| `database.rs` | `search_conversations_like()` — 43 行 |
| `chat.rs` | `escape_fts5_query()` / `event_type_emoji()` / `format_retrieved_memories()` / `truncate_for_display()` / `retrieve_memories()` + 9 个测试 — 313 行 |

### 5.5 接口变更

`prepare_chat()` 签名新增 `db: &DbState` 参数。调用方 `chat_turn()` 和 `chat_stream_start()` 同步更新。

---

## 六、测试覆盖

| 层级 | 测试数 | 状态 |
|------|--------|:--:|
| Rust unit tests | 104 | ✅ |
| Frontend (vitest) | 58 | ✅ |

新增测试：
- `test_escape_fts5_query_removes_special_chars`
- `test_escape_fts5_query_truncates_long_input`
- `test_event_type_emoji_variants`
- `test_truncate_for_display_short_text`
- `test_truncate_for_display_long_text`
- `test_format_retrieved_memories_empty`
- `test_format_retrieved_memories_with_events_and_turns`
- `test_format_retrieved_memories_only_events`
- `test_format_retrieved_memories_only_turns`
- `test_retrieve_memories_integration`

---

## 七、完整提交历史

```
7b5ca0a feat: implement Phase 4+ memory retrieval for chat context
2fdd2d3 docs: align memory system architecture with actual implementation (v1.1)
2b9bc2c docs: add memory system architecture document
0d61dc9 feat: auto-extract events from notes saved in main editor
a77cfa1 fix: only show chat feedback when event is actually saved to memory
1bf8553 feat: add user feedback for diary generation and chat event extraction
6f749ad feat: include user's own notes as diary generation source material
09bef7b fix: make diary generate button visible in toolbar with text label
b5cb8dd feat: add prominent "Generate Today's Diary" button in diary empty state
```

---

## 八、待办事项（未纳入本次）

| 优先级 | 事项 | 状态 |
|--------|------|:--:|
| P2 | Reflection 检查（周期性审视记忆库） | 待实现 |
| P2 | Notes 自动更新（LLM 自动写入 companion_notes） | 待实现 |
| P3 | 激活 observations / projects / growth_lines 自动生成 pipeline | 待设计 |
| P3 | 补数据库索引 | 待优化 |
| — | 合入 main 分支 | 待决定 |
