# Growth Companion Tauri 桌面应用迁移方案

> 文档版本: v1.2（2026-05-30）
> 基于 floral-notepaper（花笺）项目改造
> v1.1: 砍掉 Phase 1 剥离 + Phase 4 嫁接，改为就地改造增量开发
> v1.2: Phase 0-6 全部完成，release exe 已生成，NSIS 安装包待解决

---

## 1. 项目背景

### 1.1 现状

Growth Companion 是一个 AI 智能日记系统，当前架构：

```
Python FastAPI 后端（~3000 行）  +  单 HTML 前端
         ↓                              ↓
   LLM API 调用                    浏览器打开
   SQLite + 文件系统                fetch 请求后端
   情感分析/记忆/日记生成            SSE 流式聊天
```

### 1.2 为什么要迁移

| 问题 | 说明 |
|------|------|
| 体验差 | 浏览器打开，没有桌面应用的感觉 |
| 无系统集成 | 没有托盘图标、全局快捷键、离线能力 |
| 两个进程 | Python 后端 + 浏览器，启动复杂 |
| 界面粗糙 | 单 HTML 文件，无组件化 |

### 1.3 为什么选花笺

花笺（floral-notepaper）是一个成熟的 Tauri 桌面便签应用，用 Tauri 2 + React 19 + Rust 构建。它已经具备了 Growth Companion 需要的大部分基础设施：

- MD 文件管理系统
- MD 编辑器和渲染预览
- 便签快速记录 + 磁贴钉屏
- 设置面板、系统托盘、全局快捷键
- 暗色模式、多语言
- 精美的宣纸/墨色/竹青设计系统

**核心思路：在花笺的基础上加 AI 能力，而不是从零搭建。**

---

## 2. 技术架构

### 2.1 花笺的「三明治」架构

```
┌─────────────────────────────────────────────┐
│            用户界面（React + TypeScript）       │
│   按钮点击 → invoke("命令名") → 等结果返回     │
├─────────────────────────────────────────────┤
│            Tauri 桥梁（Rust）                 │
│   创建窗口 / 系统托盘 / 全局快捷键 / 文件对话框  │
├─────────────────────────────────────────────┤
│            数据处理（Rust）                    │
│   读写 MD 文件 / 管理分类 / 保存设置            │
└─────────────────────────────────────────────┘
```

所有代码在一个进程里运行，前端通过 `invoke()` 直接调用 Rust 函数，不需要 HTTP 请求。

### 2.2 迁移后的架构

```
┌───────────────────────────────────────────────────────┐
│                 用户界面（React）                       │
│                                                       │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│  │ 对话面板  │ │ 日记面板  │ │ 记忆面板  │ │ 成长面板  │ │
│  └─────┬────┘ └─────┬────┘ └─────┬────┘ └─────┬────┘ │
│        │            │            │            │       │
├────────┼────────────┼────────────┼────────────┼───────┤
│        │  invoke()  │            │            │       │
│  ┌─────▼────────────▼────────────▼────────────▼────┐  │
│  │           Tauri 命令（Rust）                      │  │
│  │                                                  │  │
│  │  ┌────────────┐  ┌────────────┐  ┌───────────┐  │  │
│  │  │ 花笺原有    │  │ 新增 AI    │  │ 新增数据库 │  │  │
│  │  │            │  │            │  │           │  │  │
│  │  │ MD 文件管理 │  │ LLM API   │  │ SQLite    │  │  │
│  │  │ 分类管理    │  │ 流式对话   │  │ 事件记忆   │  │  │
│  │  │ 设置读写    │  │ 记忆注入   │  │ 观察存储   │  │  │
│  │  │ 导入导出    │  │ 事件抽取   │  │ 主题管理   │  │  │
│  │  │ 窗口管理    │  │ 遗忘曲线   │  │ 项目档案   │  │  │
│  │  └────────────┘  └────────────┘  └───────────┘  │  │
│  └──────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────┘
```

---

## 3. 花笺已有能力 → 复用方式

### 3.1 直接复用（不改或小改）

| 花笺能力 | 实现位置 | Growth Companion 用法 |
|---------|---------|---------------------|
| MD 文件 CRUD | `services/notes.rs` | 日记、周报、章节都是 MD 文件 |
| MD 编辑器 | React 编辑组件 + 自动保存 | 查看/编辑日记内容 |
| MD 渲染预览 | `react-markdown` + GFM + KaTeX | 渲染日记、周报、观察 |
| 分类目录 | Category 分组 | 改默认分类名：日记/周报/章节/观察 |
| 便签小窗 | Notepad + 窗口池 | Ctrl+Space 快速记录 → AI 自动抽取 |
| 磁贴钉屏 | Tile + 窗口置顶 | 钉住当前关注的项目或成长线 |
| 设置面板 | Theme/Locale/快捷键 | 直接用，加 AI 相关配置 |
| 系统托盘 | `desktop.rs` | 应用常驻后台，心跳运行 |
| 全局快捷键 | `global-shortcut` 插件 | 随时弹出记录 |
| 暗色模式 | `data-theme="dark"` CSS | 直接用 |
| 多语言 | i18next zh-CN/en-US/zh-HK | 直接用 |
| 导入导出 | `.md` 文件导入导出 | 直接用 |
| 设计系统 | paper/ink/bamboo 色系 + 噪点 | 直接用 |

### 3.2 增量改造（不删除，在花笺基础上加）

| 花笺功能 | 改造方向 |
|---------|---------|
| MainWindow 侧边栏 | 在现有笔记列表区域加导航 Tab（对话/日记/记忆/成长），日记 Tab 复用现有笔记列表 |
| 笔记编辑区域 | 日记面板下直接复用编辑器；其他面板切换到对应新组件 |
| 分类侧边栏 | 改默认分类为 diary/summaries/chapters/observations，UI 保留 |
| 设置面板 | 保留现有字段，新增 AI/LLM 配置区块 |
| i18n 文案 | "笔记"→"日记"，"花笺"→"Growth Companion"，其余保留 |
| 数据目录 | `%USERPROFILE%/Documents/花笺` → `%USERPROFILE%/Documents/Growth Companion` |

### 3.3 全新开发

| 功能 | 说明 |
|------|------|
| LLM API 客户端 | Rust 调 OpenAI/Claude/DeepSeek API，支持流式 SSE |
| 对话引擎 | prompt 组装、记忆注入、对话历史管理、后处理 |
| 事件抽取 | 从对话中识别情绪波动、重要事件、行为模式 |
| SQLite 数据层 | 事件、观察、主题、项目、成长线等结构化数据 |
| 记忆系统 | 核心记忆文件 + 事件 DB + 遗忘曲线衰减 |
| 定时任务 | 心跳循环、遗忘曲线维护、周报/章节生成触发 |
| 聊天面板 | 流式气泡 UI、记忆展示、主动消息 |
| 成长时间线 | 观察记录的垂直时间线 |
| 记忆管理 | 事件浏览、核心记忆编辑、容量展示 |

---

## 4. 技术选型

### 4.1 Rust 侧新增依赖

| 依赖 | 用途 |
|------|------|
| `reqwest` | HTTP 客户端，调 LLM API |
| `tokio` | 异步运行时（流式 SSE 必需） |
| `serde` / `serde_json` | JSON 序列化/反序列化 |
| `rusqlite` | SQLite 数据库操作 |
| `chrono` | 时间处理（已有） |
| `uuid` | ID 生成（已有） |

### 4.2 React 侧新增依赖

| 依赖 | 用途 |
|------|------|
| 无需新增 | 花笺已有 React 19 + react-markdown + Tailwind，日记/编辑/渲染全部复用 |

### 4.3 移除的依赖

| 依赖 | 原因 |
|------|------|
| 无需移除 | 花笺全部依赖保留，KaTeX 虽然日记用不到但保留不影响 |

---

## 5. 数据存储设计

### 5.1 文件系统（花笺 NoteStore 直接复用）

```
%USERPROFILE%/Documents/Growth Companion/
  ├── config.json               # 花笺 AppConfig（扩展 AI 配置）
  ├── metadata.json             # 花笺笔记索引（复用）
  ├── notes/                    # 花笺 NoteStore 管理的 MD 文件
  │   ├── diary/                # 分类：日记（花笺 category 机制）
  │   │   ├── 2026-05-29.md
  │   │   └── 2026-05-28.md
  │   ├── summaries/            # 分类：周报
  │   │   └── week-2026-W22.md
  │   ├── chapters/             # 分类：人生章节
  │   │   └── 你开始允许自己脆弱.md
  │   └── observations/         # 分类：成长观察
  │       └── 2026-05-29.md
  ├── user_profile.md           # 核心记忆：用户画像
  └── companion_notes.md        # 核心记忆：AI 笔记
```

花笺的 `NoteStore` 已经支持：按分类目录管理 MD 文件、JSON 索引、自动修复、回收站删除。日记/周报/章节/观察都复用这套机制，不需要重写。

### 5.2 SQLite（新增）

```sql
-- 事件记忆
CREATE TABLE events (
  id TEXT PRIMARY KEY,
  content TEXT NOT NULL,
  emotions TEXT,            -- JSON: ["sadness", "anger"]
  importance REAL,          -- 0.0 ~ 1.0
  event_type TEXT,          -- conflict | milestone | emotion | decision
  strength REAL,            -- 遗忘曲线强度
  stability REAL,           -- 记忆稳定天数
  recall_count INTEGER,
  last_recalled_at TEXT,
  created_at TEXT,
  updated_at TEXT
);

-- 成长观察
CREATE TABLE observations (
  id TEXT PRIMARY KEY,
  date TEXT NOT NULL,
  content TEXT NOT NULL,
  category TEXT,            -- emotion | behavior | relationship | value | growth
  source TEXT,              -- reflection | event
  created_at TEXT
);

-- 主题
CREATE TABLE topics (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  first_mentioned TEXT,
  last_mentioned TEXT,
  mention_count INTEGER,
  created_at TEXT,
  updated_at TEXT
);

-- 主题关联
CREATE TABLE topic_links (
  topic_id TEXT NOT NULL,
  item_id TEXT NOT NULL,
  item_type TEXT NOT NULL,  -- event | observation
  PRIMARY KEY (topic_id, item_id, item_type)
);

-- 项目档案
CREATE TABLE projects (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT,
  status TEXT DEFAULT 'active',
  start_date TEXT,
  end_date TEXT,
  event_ids TEXT,           -- JSON array
  summary TEXT,
  created_at TEXT,
  updated_at TEXT
);

-- 成长线
CREATE TABLE growth_lines (
  id TEXT PRIMARY KEY,
  dimension TEXT NOT NULL,
  records TEXT,             -- JSON: [{date, note}]
  created_at TEXT,
  updated_at TEXT
);

-- 对话轮次（全文搜索）
CREATE VIRTUAL TABLE conversation_turns USING fts5(
  content,
  role,
  created_at
);

-- 结构化洞察
CREATE TABLE insights (
  id TEXT PRIMARY KEY,
  category TEXT NOT NULL,
  content TEXT NOT NULL,
  source_event_ids TEXT,
  created_at TEXT
);
```

### 5.3 核心记忆文件（复用花笺 MD 管理）

**user_profile.md**（≤1200 字符）：
```markdown
用户基本情况：互联网产品经理，在考虑转独立开发
沟通偏好：随和，不喜欢被说教
当前关注：职业方向、个人项目
近期状态：在纠结是否辞职
```

**companion_notes.md**（≤800 字符）：
```markdown
用户最近在纠结要不要辞职做独立开发
之前聊过两次这个话题，态度从犹豫到倾向行动
聊技术细节时很兴奋，可以多追问
```

---

## 6. 实施计划

### Phase 0: 脚手架 ✅（已完成）

```
✓ git clone 花笺到 frontend/
✓ 删除 .git，解除与原仓库关联
✓ 改名：package.json / Cargo.toml / tauri.conf.json / index.html / main.rs
✓ npm install && npm run tauri dev 验证能跑
✓ 确认花笺 UI 正常显示
```

### Phase 1: 就地改名 ✅（已完成）

```
1.1 i18n 文案替换
    └── "花笺" → "Growth Companion"
    └── "笔记" → "日记"（UI 显示层）
    └── 文件: src/locales/zh-CN/*.json, en-US/*.json, zh-HK/*.json

1.2 默认分类名
    └── 改 AppConfig 默认分类为 diary / summaries / chapters / observations
    └── 文件: src-tauri/src/services/notes.rs (AppConfig defaults)

1.3 数据目录改名
    └── Windows: Documents/花笺 → Documents/Growth Companion
    └── 环境变量: FLORAL_NOTEPAPER_DATA_DIR → GROWTH_COMPANION_DATA_DIR
    └── 文件: src-tauri/src/services/notes.rs (base_dir logic)

1.4 窗口标题 + 托盘
    └── 确认所有窗口标题、托盘菜单显示 "Growth Companion"
    └── 文件: src-tauri/src/locales.rs, src-tauri/src/desktop.rs
```

验证：启动后 UI 显示"日记"而非"笔记"，数据存在新目录。

### Phase 2: MainWindow 增量改造 ✅（已完成）

在现有 MainWindow 上加侧边栏导航，日记面板直接复用花笺笔记 UI：

```
2.1 侧边栏导航
    └── 在现有笔记列表区域顶部加导航 Tab
    └── 四个 Tab: 对话 | 日记 | 记忆 | 成长
    └── 文件: src/components/MainWindow.tsx

2.2 日记面板（复用）
    └── 选中"日记"Tab → 显示现有笔记列表 + 编辑器，完全不动
    └── 分类侧边栏保留，显示 diary/summaries/chapters/observations

2.3 空白面板占位
    └── 对话/记忆/成长 Tab → 先渲染占位文字，Phase 4 填充
    └── 文件: src/components/panels/ChatPanel.tsx (placeholder)
    └── 文件: src/components/panels/MemoryPanel.tsx (placeholder)
    └── 文件: src/components/panels/GrowthPanel.tsx (placeholder)

2.4 面板切换状态
    └── useState 管理 activePanel，切换时隐藏/显示对应区域
    └── 日记面板保持花笺原有的全部状态（选中笔记、编辑内容等）
```

验证：侧边栏切换正常，日记 Tab 显示花笺原有笔记功能，其他 Tab 显示占位。

### Phase 3: Rust AI 核心模块 ✅（已完成）

按依赖顺序开发，每完成一个模块同步注册 Tauri command：

```
3.1 SQLite 数据层
    └── 建表（events, observations, topics, topic_links, projects, growth_lines, conversation_turns FTS5, insights）
    └── CRUD 操作 + 事务支持
    └── 文件: src-tauri/src/services/database.rs
    └── 新增依赖: rusqlite

3.2 LLM API 客户端
    └── reqwest HTTP 客户端 + tokio 异步运行时
    └── SSE 流式解析（chat_send 用）
    └── 支持多提供商（OpenAI / Claude / DeepSeek / 自定义）
    └── 文件: src-tauri/src/services/llm.rs
    └── 新增依赖: reqwest, tokio

3.3 记忆系统
    └── 核心记忆文件读写（user_profile.md, companion_notes.md）
    └── 事件记忆 CRUD + 遗忘曲线衰减
    └── 容量计算 + near_limit 警告
    └── 全文搜索（FTS5）
    └── 文件: src-tauri/src/services/memory.rs

3.4 对话引擎
    └── prompt 组装：system prompt + 核心记忆 + 检索记忆 + 对话历史
    └── LLM 调用 + SSE 流式返回
    └── 后处理管线：情绪抽取 → 事件创建 → 主题关联 → 日期数据累积
    └── 文件: src-tauri/src/services/chat.rs

3.5 事件抽取
    └── 从对话中识别情绪波动、重要事件、行为模式
    └── LLM 结构化 JSON 输出解析
    └── 文件: src-tauri/src/services/extractor.rs

3.6 定时任务
    └── 心跳循环：遗忘曲线维护 + 空闲检测 + 主动消息
    └── 周报触发：每周一次 reflection
    └── 日记自动生成检查：每日首次对话时补昨日日记
    └── 文件: src-tauri/src/services/scheduler.rs
```

验证：每个模块完成后用 `cargo test` 验证，command 可通过 tauri invoke 测试。

### Phase 4: React 面板开发 ✅（已完成）

```
4.1 API 客户端封装
    └── invoke() 调用封装 + TypeScript 类型定义
    └── src/features/api/ (types.ts, chat.ts, diary.ts, memory.ts, observations.ts, heartbeat.ts)

4.2 ChatPanel（最复杂）
    └── 流式气泡 UI：接收 Tauri 事件流，逐 token 渲染
    └── 记忆展示：对话中显示被注入的核心记忆片段
    └── 输入框：支持多行、Shift+Enter 换行、发送触发 chat_send
    └── src/components/panels/ChatPanel.tsx
    └── src/components/chat/ (MessageList.tsx, MessageBubble.tsx, ChatInput.tsx)

4.3 MemoryPanel
    └── 核心记忆卡片：显示 profile + notes，支持编辑（patch_core_memory）
    └── 事件列表：按时间/重要性筛选，显示遗忘曲线强度
    └── 容量展示：进度条显示 profile/notes 使用百分比
    └── src/components/panels/MemoryPanel.tsx
    └── src/components/memory/ (CoreMemoryCard.tsx, EventList.tsx, ChapterList.tsx)

4.4 GrowthPanel
    └── 观察时间线：垂直时间线展示定性观察
    └── 主题卡片：显示主题 + 关联事件数 + 提及频率
    └── 成长线图表：维度维度展示长期变化
    └── src/components/panels/GrowthPanel.tsx
    └── src/components/growth/ (ObservationTimeline.tsx, TimelineItem.tsx)

4.5 InitPanel
    └── 用户初始化表单：姓名、基本介绍、沟通偏好
    └── 调用 init_user command
    └── src/components/panels/InitPanel.tsx
```

验证：各面板可渲染 mock 数据，invoke 调用正常。

### Phase 5: 花笺功能 AI 化 ✅（已完成）

```
5.1 便签 → 快速记录
    └── Ctrl+Space 弹出便笺（保留花笺原有逻辑）
    └── 写入后新增后处理：调用 extractor 从内容中抽取事件
    └── 文件: src-tauri/src/services/extractor.rs (新增 quick_extract command)
    └── 文件: src/components/NotePad.tsx (onSave 时触发 AI 抽取)

5.2 磁贴 → 置顶关注
    └── 保留花笺磁贴钉屏功能
    └── 可钉住项目档案或成长线（内容来自 Phase 3 数据）

5.3 设置面板改造
    └── 保留现有设置区块（主题/语言/快捷键等）
    └── 新增 AI 配置区块：
        - LLM 提供商（openai/claude/deepseek/custom）
        - API Key（加密存储）
        - API Base URL
        - 模型名称
        - 聊天风格 / 称呼方式 / 建议力度
    └── 文件: src/components/SettingsPanel.tsx
    └── 文件: src-tauri/src/services/notes.rs (AppConfig 新增字段)
```

验证：Ctrl+Space 便签 → 写入 → 事件被抽取。设置面板可配置 AI 参数。

### Phase 6: 集成测试 + 构建 ✅（已完成，NSIS 安装包待解决）

```
6.1 自动化测试
    ✓ 前端测试：19 files / 53 tests 全部通过
    ✓ Rust 测试：77 tests 全部通过
    ✓ npm run build 通过
    ✓ npm run lint 通过

6.2 构建
    ✓ npm run tauri build 成功生成 release exe
    ✓ 路径：frontend/src-tauri/target/release/growth-companion.exe
    ✗ NSIS 安装包：Tauri 下载 NSIS 失败（socket 10013 → 提权后 global timeout）
      代码和 exe 均编译成功，卡点在外部 NSIS 下载，后续手动安装 NSIS 或切换网络解决

6.3 杂项
    ✓ 根 .gitignore 补全 frontend 忽略规则（node_modules/dist/target/gen）
    ✓ 迁移计划文档更新至 v1.2
```

---

## 7. 目录结构预览

```
D:/人格画像/
  ├── src/                          # Python 后端（迁移完成后可归档）
  ├── data/                         # 用户数据（迁移到 Tauri 管理）
  ├── docs/
  │   └── tauri-migration-plan.md   # 本文档
  ├── prd.md
  ├── CLAUDE.md
  ├── requirements.txt
  │
  └── frontend/                     # Tauri 桌面应用
      ├── package.json
      ├── vite.config.ts
      ├── tsconfig.json
      ├── index.html
      │
      ├── src/
      │   ├── main.tsx              # 入口（花笺保留）
      │   ├── App.tsx               # 根组件（花笺保留）
      │   ├── App.css               # 设计系统（花笺保留）
      │   │
      │   ├── assets/
      │   │   └── fonts/            # HarmonyOS Sans SC 字体（花笺保留）
      │   │
      │   ├── components/
      │   │   ├── MainWindow.tsx    # 主窗口（增量改造：加导航 Tab + 面板切换）
      │   │   ├── NotePad.tsx       # 便签窗口（花笺保留，Phase 5 加 AI 抽取）
      │   │   ├── Tile.tsx          # 磁贴组件（花笺保留）
      │   │   ├── TileShowcase.tsx  # 磁贴展示（花笺保留）
      │   │   ├── SettingsPanel.tsx # 设置面板（花笺保留，加 AI 配置区块）
      │   │   ├── ContextMenu.tsx   # 右键菜单（花笺保留）
      │   │   ├── SlidingButtonGroup.tsx  # 切换按钮组（花笺保留）
      │   │   ├── BackgroundLayer.tsx    # 背景层（花笺保留）
      │   │   │
      │   │   ├── panels/           # 新增：各面板
      │   │   │   ├── ChatPanel.tsx
      │   │   │   ├── MemoryPanel.tsx
      │   │   │   ├── GrowthPanel.tsx
      │   │   │   └── InitPanel.tsx
      │   │   │
      │   │   ├── chat/             # 新增：聊天子组件
      │   │   │   ├── MessageList.tsx
      │   │   │   ├── MessageBubble.tsx
      │   │   │   └── ChatInput.tsx
      │   │   │
      │   │   ├── memory/           # 新增：记忆子组件
      │   │   │   ├── CoreMemoryCard.tsx
      │   │   │   ├── EventList.tsx
      │   │   │   └── ChapterList.tsx
      │   │   │
      │   │   └── growth/           # 新增：成长子组件
      │   │       ├── ObservationTimeline.tsx
      │   │       └── TimelineItem.tsx
      │   │
      │   ├── features/
      │   │   ├── notes/            # 花笺保留（日记复用）
      │   │   │   ├── api.ts
      │   │   │   ├── types.ts
      │   │   │   └── useNotes.ts
      │   │   │
      │   │   ├── api/              # 新增：AI 相关 Tauri invoke 封装
      │   │   │   ├── types.ts
      │   │   │   ├── chat.ts
      │   │   │   ├── diary.ts
      │   │   │   ├── memory.ts
      │   │   │   ├── observations.ts
      │   │   │   └── heartbeat.ts
      │   │   │
      │   │   ├── settings/         # 花笺保留，加 AI 字段
      │   │   │   ├── api.ts
      │   │   │   ├── theme.ts
      │   │   │   ├── types.ts      # AppConfig 加 AI 字段
      │   │   │   └── tileColor.ts
      │   │   │
      │   │   ├── markdown/         # 花笺保留（日记渲染）
      │   │   │   └── ...
      │   │   │
      │   │   └── windows/          # 花笺保留
      │   │       ├── windowRoutes.ts
      │   │       └── controls.ts
      │   │
      │   └── locales/              # 花笺保留，更新翻译
      │       ├── zh-CN/
      │       ├── en-US/
      │       └── zh-HK/
      │
      └── src-tauri/
          ├── Cargo.toml
          ├── tauri.conf.json
          ├── capabilities/
          │   └── default.json
          │
          └── src/
              ├── main.rs           # 入口
              ├── lib.rs             # 命令注册（花笺保留 + 新增 AI commands）
              ├── desktop.rs         # 托盘/快捷键/窗口（花笺保留）
              ├── locales.rs         # 国际化（花笺保留）
              │
              └── services/
                  ├── mod.rs         # 模块声明（花笺保留 + 新增）
                  ├── notes.rs       # 花笺保留（日记复用 NoteStore + AppConfig）
                  │
                  ├── database.rs    # 新增：SQLite 数据层
                  ├── llm.rs         # 新增：LLM API 客户端
                  ├── memory.rs      # 新增：记忆系统
                  ├── chat.rs        # 新增：对话引擎
                  ├── extractor.rs   # 新增：事件抽取
                  └── scheduler.rs   # 新增：定时任务
```

---

## 8. Rust 侧新增命令清单

| 命令 | 功能 | 对应原 Python API |
|------|------|-----------------|
| `chat_send` | 发送消息，返回流式 SSE | POST /api/chat/stream |
| `init_user` | 初始化用户 | POST /api/init |
| `get_core_memory` | 读取核心记忆 | GET /api/memory/core |
| `patch_core_memory` | 编辑核心记忆 | PATCH /api/memory/core |
| `get_events` | 查询事件记忆 | GET /api/memory/events |
| `delete_event` | 删除事件 | DELETE /api/memory/events/{id} |
| `get_observations` | 获取观察列表 | GET /api/observations |
| `get_topics` | 获取主题列表 | GET /api/topics |
| `get_topic_detail` | 主题详情 + 关联 | GET /api/topics/{id} |
| `get_topic_compare` | 主题对比 | GET /api/topics/{id}/compare |
| `get_projects` | 项目列表 | GET /api/projects |
| `generate_projects` | 触发项目归并 | POST /api/projects/generate |
| `get_growth_lines` | 成长线列表 | GET /api/growth-lines |
| `generate_diary` | 生成日记 | POST /api/diary/generate |
| `get_diary_list` | 日记列表 | GET /api/diary |
| `generate_summary` | 生成周报 | POST /api/summary/weekly |
| `generate_chapter` | 生成章节 | POST /api/chapters/generate |
| `heartbeat_check` | 心跳检查 | GET /api/heartbeat |
| `maintain_memory` | 遗忘曲线维护 | POST /api/memory/events/maintain |
| `search_conversations` | 全文搜索 | GET /api/memory/search |

---

## 9. 设置面板配置项

### 9.1 保留自花笺

| 配置项 | 字段 | 说明 |
|--------|------|------|
| 主题 | `theme` | light / dark / system |
| 语言 | `locale` | zh-CN / en-US / zh-HK |
| 编辑器字号 | `font_size` | 8-30px |
| 关闭到托盘 | `close_to_tray` | bool |
| 开机自启 | `autostart` | bool |
| 全局快捷键 | `global_shortcut` | 默认 Ctrl+Space |

### 9.2 新增 AI 配置

| 配置项 | 字段 | 说明 |
|--------|------|------|
| LLM 提供商 | `llm_provider` | openai / claude / deepseek / custom |
| API Key | `llm_api_key` | 加密存储 |
| API Base URL | `llm_base_url` | 默认 https://api.openai.com/v1 |
| 模型名称 | `llm_model` | 默认 gpt-4.1 |
| 聊天风格 | `chat_style` | 随意轻松 / 温和耐心 / 直接坦率 |
| 称呼方式 | `address_preference` | 用"你" / 用昵称 |
| 建议力度 | `advice_preference` | 温和引导 / 适度建议 / 直接建议 |

---

## 10. 风险与注意事项

| 风险 | 应对策略 |
|------|---------|
| Rust 学习曲线 | 优先参考花笺现有代码模式（notes.rs），不造新轮子 |
| LLM SSE 流式解析 | 使用 reqwest 的 stream + tokio 异步处理 |
| SQLite 并发 | 使用 Tauri 的 state 管理，单连接 + Mutex |
| 迁移期间双后端并存 | Python 后端保留，Tauri 独立开发，逐步替换 |
| 数据迁移 | 提供 Python → Tauri 数据迁移脚本 |
| MainWindow 过大（2383 行） | 增量改造而非重写，新面板代码放独立组件文件 |
| 花笺笔记 UI 深度耦合 | 日记面板直接复用，不拆分；新面板完全独立 |

---

## 11. 开发命令速查

```bash
# 安装依赖（仅首次）
cd frontend
npm install

# 开发模式（Tauri 一体化，不需要 Python 后端）
npm run tauri dev

# 构建 release exe
npm run tauri build

# 测试
npm test -- --run                                      # 前端测试（vitest）
cargo test --manifest-path frontend/src-tauri/Cargo.toml  # Rust 测试

# 代码检查
npm run lint                                           # oxlint
cargo fmt --manifest-path frontend/src-tauri/Cargo.toml -- --check  # Rust 格式
```
