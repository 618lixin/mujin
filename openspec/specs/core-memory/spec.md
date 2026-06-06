## Purpose
Define the local core memory files that are always available to chat and diary generation.

## Requirements

### Requirement: Core memory file management
系统 SHALL 管理两个核心记忆文件：`user_profile.md`（用户画像，默认 ≤1200 字符）和 `companion_notes.md`（AI 笔记，默认 ≤800 字符）。

#### Scenario: Read core memory
- **WHEN** 系统需要读取核心记忆
- **THEN** 返回两个文件的完整内容、字符数、容量上限、使用率和 near-limit 标记

#### Scenario: Missing core memory files
- **WHEN** 核心记忆文件不存在
- **THEN** 系统返回空内容并继续工作

### Requirement: Core memory patching
系统 SHALL 支持对 profile 或 notes 执行 `add`、`replace`、`remove` patch 操作，并保持容量限制。

#### Scenario: Add memory text
- **WHEN** patch action 为 `add`
- **THEN** 系统将内容追加到目标记忆文件，且写入后不得超过该文件容量上限

#### Scenario: Replace memory text
- **WHEN** patch action 为 `replace` 且提供 `old_text`
- **THEN** 系统用新内容替换目标文件中的旧文本

#### Scenario: Remove memory text
- **WHEN** patch action 为 `remove`
- **THEN** 系统从目标文件中移除匹配内容

#### Scenario: Patch exceeds capacity
- **WHEN** patch 会导致目标文件超过容量上限
- **THEN** 系统 SHALL 拒绝写入并返回错误

### Requirement: Core memory prompt injection
系统 SHALL 在每轮对话开始时将核心记忆注入 system prompt。

#### Scenario: Memory injected into prompt
- **WHEN** 构建新的对话请求
- **THEN** system prompt 包含 `USER PROFILE` 和 `COMPANION NOTES` 两个区块及其实际内容

### Requirement: Tauri core memory commands
系统 SHALL 通过 Tauri command 查看和编辑核心记忆。

#### Scenario: Get core memory
- **WHEN** 前端调用 `ai_get_core_memory`
- **THEN** 返回 profile 与 notes 的内容和容量统计

#### Scenario: Patch core memory
- **WHEN** 前端调用 `ai_patch_core_memory`
- **THEN** 系统执行 patch 并返回更新后的核心记忆统计
