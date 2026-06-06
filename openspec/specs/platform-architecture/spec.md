## Purpose
Define the current desktop-first architecture after the Tauri/Rust refactor.

## Requirements

### Requirement: Desktop-first architecture
系统 SHALL 以 Tauri 2 桌面应用作为当前主运行形态。

#### Scenario: Application entry
- **WHEN** 开发者启动应用
- **THEN** 前端入口位于 `frontend/`，通过 Vite/React 渲染 UI，通过 Tauri command 调用本地 Rust 服务

#### Scenario: No active Python backend
- **WHEN** 查阅当前实现
- **THEN** 系统 SHALL NOT 依赖旧 Python/FastAPI 服务作为运行时后端

### Requirement: Rust local service layer
系统 SHALL 在 `frontend/src-tauri/src/services/` 中实现 AI、记忆、日记、笔记和调度服务。

#### Scenario: Chat service
- **WHEN** 用户聊天
- **THEN** `services/chat.rs` 负责编排 prompt、LLM 调用、流式事件和 post-chat pipeline

#### Scenario: Memory database service
- **WHEN** 系统读写事件、对话轮次、主题、观察、项目或成长线
- **THEN** `services/database.rs` 负责 SQLite schema 和 CRUD

#### Scenario: Diary service
- **WHEN** 用户生成或重新生成日记
- **THEN** `services/diary.rs` 聚合素材并写入 NoteStore

### Requirement: Frontend API boundary
前端 SHALL 通过 `frontend/src/features/api/` 封装 Tauri command 调用，避免业务组件直接散落调用底层命令。

#### Scenario: Chat API wrapper
- **WHEN** Chat UI 需要发送消息
- **THEN** 使用 `features/api/chat.ts` 中的封装

#### Scenario: Memory API wrapper
- **WHEN** Memory UI 需要查询或编辑记忆
- **THEN** 使用 `features/api/memory.ts` 中的封装

### Requirement: Local-first data storage
系统 SHALL 使用本地文件系统和 SQLite 保存用户数据。

#### Scenario: Markdown notes
- **WHEN** 用户创建、导入、编辑或导出笔记
- **THEN** NoteStore 以 Markdown 文件和元数据管理内容

#### Scenario: Structured memory
- **WHEN** 系统保存结构化记忆
- **THEN** 使用用户级 SQLite 数据库表保存 events、conversation_turns、observations、topics、projects、growth_lines 等数据

### Requirement: Validation commands
当前项目 SHALL 使用 npm 与 cargo 验证桌面应用。

#### Scenario: Frontend tests
- **WHEN** 验证前端
- **THEN** 在 `frontend/` 运行 `npm.cmd test`

#### Scenario: Rust tests
- **WHEN** 验证 Tauri/Rust 后端
- **THEN** 在 `frontend/src-tauri/` 运行 `cargo test`

#### Scenario: Production frontend build
- **WHEN** 验证生产前端构建
- **THEN** 在 `frontend/` 运行 `npm.cmd run build`
