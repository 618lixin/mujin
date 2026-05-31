# AI 智能日记系统

基于“降低输入成本 + 提高输出价值”的 AI 辅助日记管理产品。项目目标是让用户通过自然对话完成记录，并由 AI 持续沉淀为日记、记忆、观察和长期成长档案。

## 目录结构

```text
.
├── src/              # 原 Python 后端与服务实现
├── frontend/         # Tauri 桌面端：React 前端 + Rust 本地后端
├── static/           # Python 后端的静态调试页面
├── docs/             # 架构、迁移计划和阶段开发说明
├── openspec/         # OpenSpec 需求规范与变更记录
├── .agents/          # 项目使用的本地 agent skills
├── AGENTS.md         # 协作规则与项目工作流
├── prd.md            # 产品需求文档
└── requirements.txt  # Python 后端依赖
```

## 开发环境

Python 后端依赖使用 conda 环境 `growth-companion`：

```powershell
conda run -n growth-companion python -m uvicorn src.main:app --reload
```

桌面端在 `frontend/` 目录下运行：

```powershell
cd frontend
npm install
npm run tauri dev
```

常用前端验证命令：

```powershell
cd frontend
npm run lint
npm test -- --run
npm run build
```

## 版本管理约定

根仓库是唯一版本管理入口。`frontend/` 是本项目的桌面端子目录，不再保留独立 GitHub workflow、独立 README 或独立发布说明。

以下内容不进入版本控制：

- `.env` 和其他本地密钥
- `data/` 运行时数据库与用户数据
- `frontend/node_modules/`
- `frontend/dist/`
- `frontend/src-tauri/target/`
- 本地浏览器调试日志和 Python 缓存

第三方来源与许可说明见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。
