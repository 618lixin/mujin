## Purpose
Define lightweight user initialization for the current local-first desktop app.

## Requirements

### Requirement: Lightweight user initialization
系统 SHALL 使用轻量初始化替代旧的 MBTI/人格问卷初始化。

#### Scenario: Initialize user
- **WHEN** 前端调用 `ai_init_user` 并提供 `user_id`
- **THEN** 系统创建用户目录并初始化 SQLite schema

#### Scenario: Re-initialize existing user
- **WHEN** 用户目录和数据库已存在
- **THEN** 初始化 SHALL 保持幂等，不破坏已有数据

### Requirement: No MBTI-derived weights
系统 SHALL NOT 根据 MBTI 生成八维人格权重。

#### Scenario: User has no MBTI
- **WHEN** 初始化用户
- **THEN** 系统不要求 MBTI，也不创建 personality weights

### Requirement: Profile remains editable memory
用户画像 SHALL 作为核心记忆的一部分由 `user_profile.md` 管理，而不是由固定问卷一次性决定。

#### Scenario: Empty profile after initialization
- **WHEN** 用户刚初始化但尚未填写画像
- **THEN** 核心记忆读取可以返回空 profile

#### Scenario: Profile edited later
- **WHEN** 前端调用 `ai_patch_core_memory`
- **THEN** 系统更新 `user_profile.md`
