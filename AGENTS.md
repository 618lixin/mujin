# AI 智能日记系统

基于「降低输入成本 + 提高输出价值」的 AI 辅助日记管理产品。

## 项目概述

核心目标：对话即记录，AI 帮用户串联、提取、分析、对比日记，变成长期成长档案。

- 技术栈：Tauri 2 + Rust 后端 + React 19 前端
- 架构：单 LLM + 五层沉淀系统（对话层 → 抽取层 → 记忆层 → Reflection → 沉淀生成器）

## 环境管理

桌面应用入口在 `frontend/` 目录，使用 npm 和 cargo 管理依赖。

```
cd frontend
npm install              # 安装前端依赖
npm run dev              # Vite 开发服务器
npm run tauri dev        # Tauri 桌面开发模式
npm run tauri build      # 构建 release exe
cargo test               # Rust 测试（在 frontend/src-tauri/ 下）
npm test                 # 前端测试
```

## 三工具协同工作流

本项目使用 OpenSpec + SuperPowers + GStack 三套工具协同开发。

### 工具职责划分（互不覆盖）

| 工具 | 管理层次 | 状态存储 |
|------|---------|---------|
| **OpenSpec** | 需求规范 | `openspec/` 目录 |
| **SuperPowers** | 编码纪律 | `AGENTS.md` + skill 文件 |
| **GStack** | 全流程管线 | `.gstack/` 目录 |

### 四个自动串联点

1. **OpenSpec 产物 → GStack 评审输入**
   OpenSpec 的 proposal/specs/design/tasks 产物，作为 GStack `/autoplan` 的评审输入
   `/opsx:propose` 完成 → `/autoplan` 读取产物做 CEO/设计/工程评审

2. **SuperPowers HARD-GATE 拦截编码**
   编码前必须先有失败的测试（TDD 铁律）
   例外：一次性原型、生成的代码、配置文件可跳过 TDD

3. **SuperPowers TDD → GStack Review 生效**
   有了测试才有 review 的基线
   `/review` 基于测试通过的代码进行审查

4. **GStack Ship → OpenSpec Archive 归档**
   `/ship` 完成发布后，触发 `/opsx:archive` 归档变更
   归档时将 delta 规范合入主规范

### 完整开发流程（7 步）

```
1. /opsx:propose "功能描述"
   → 生成 proposal.md, specs/, design.md, tasks.md

2. /autoplan
   → 读取 OpenSpec 产物，运行 CEO/设计/工程评审

3. TDD 铁律自动生效（SuperPowers）
   → 先写测试，看它失败，再写最小代码通过

4. /review
   → 代码审查，扫描问题

5. /qa
   → 用 Playwright/Chromium 做真实浏览器验收（如有 Web 界面）

6. /ship
   → 自动执行：VERSION 升级、CHANGELOG 生成、PR 创建推送

7. /opsx:archive
   → 归档变更，delta 规范合入主规范
```

### 避坑指南

- **不重复门禁**：OpenSpec 做了设计审批，就不要再用 GStack 的 plan design review 重复审查同一份设计
- **Specs 是唯一真相源**：需求以 OpenSpec specs 为准，GStack 设计文档只描述实现细节
- **Ship 是唯一发布出口**：OpenSpec 归档只是收尾记录，不是发布
- **TDD 三个例外**：一次性原型、生成的代码、配置文件可跳过，其余不打折扣

## gstack

Use /browse from gstack for all web browsing. Never use mcp__claude-in-chrome__* tools.
Available skills: /office-hours, /plan-ceo-review, /plan-eng-review, /plan-design-review,
/design-consultation, /design-shotgun, /design-html, /review, /ship, /land-and-deploy,
/canary, /benchmark, /browse, /open-gstack-browser, /qa, /qa-only, /design-review,
/setup-browser-cookies, /setup-deploy, /setup-gbrain, /sync-gbrain, /retro, /investigate,
/document-release, /document-generate, /codex, /cso, /autoplan, /pair-agent, /careful, /freeze,
/guard, /unfreeze, /gstack-upgrade, /learn.
