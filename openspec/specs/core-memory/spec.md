## ADDED Requirements

### Requirement: Core memory file management
系统 SHALL 管理两个核心记忆文件：user_profile.md（用户画像，≤1200 字符）和 companion_notes.md（AI 笔记，≤800 字符），存储在 data/{user_id}/ 目录下。

#### Scenario: Read core memory
- **WHEN** 系统需要读取核心记忆
- **THEN** 返回两个文件的完整内容，若文件不存在则返回空内容

#### Scenario: Update user profile
- **WHEN** AI 调用记忆写入工具更新 user_profile.md
- **THEN** 新内容替换旧内容，总字符数不超过 1200

#### Scenario: Update companion notes
- **WHEN** AI 调用记忆写入工具更新 companion_notes.md
- **THEN** 新内容替换旧内容，总字符数不超过 800

### Requirement: Core memory capacity management
当核心记忆文件使用超过 80% 容量时，系统 SHALL 触发压缩流程：AI 合并相关条目、删除过时信息、压缩表述。

#### Scenario: User profile near capacity
- **WHEN** user_profile.md 内容超过 960 字符（80%）
- **THEN** 系统在下一次对话中提示 AI 执行压缩，将内容精简至 80% 以下

#### Scenario: Capacity management on write
- **WHEN** 写入操作会导致文件超过字符上限
- **THEN** 系统 SHALL 拒绝写入并返回当前内容和容量信息，要求 AI 先压缩再写入

### Requirement: Core memory prompt injection
系统 SHALL 在每轮对话开始时将核心记忆内容注入 system prompt，格式为带标题和容量百分比的冻结块。

#### Scenario: Memory injected into prompt
- **WHEN** 构建新的对话请求
- **THEN** system prompt 包含 USER PROFILE 和 COMPANION NOTES 两个区块，显示当前字符数/上限和实际内容

### Requirement: Core memory CRUD API
系统 SHALL 提供 REST API 端点用于查看和编辑核心记忆文件。

#### Scenario: GET core memory
- **WHEN** 调用 GET /api/memory/core
- **THEN** 返回 user_profile.md 和 companion_notes.md 的内容及容量信息

#### Scenario: PATCH core memory
- **WHEN** 调用 PATCH /api/memory/core 并提供 action（add/replace/remove）和 content
- **THEN** 系统执行对应操作并返回更新后的内容
