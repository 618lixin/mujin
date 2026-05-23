## 1. 项目骨架与基础设施

- [x] 1.1 创建项目目录结构（src/main.py, src/config.py, src/models/, src/services/, src/api/, data/）
- [x] 1.2 初始化 FastAPI 应用，配置 CORS 和基础路由
- [x] 1.3 添加依赖：fastapi, uvicorn, pydantic, httpx（LLM 调用）
- [x] 1.4 实现配置管理（src/config.py）：LLM API key、模型名、数据目录路径、记忆容量参数

## 2. LLM 调用封装

- [x] 2.1 实现 src/services/llm.py：统一的 LLM API 调用接口（支持 OpenAI 兼容格式）
- [x] 2.2 支持配置多个模型（主聊天模型 vs 便宜模型用于情绪识别/日记生成）

## 3. 数据模型

- [x] 3.1 实现 src/models/personality.py：八维权重向量数据模型（Ti/Te/Fi/Fe/Si/Se/Ni/Ne），含权重边界校验
- [x] 3.2 实现 src/models/event.py：事件数据模型（id, content, emotions, importance, event_type, timestamps）
- [x] 3.3 实现 src/models/diary.py：日记数据模型（date, content, generated_at）
- [x] 3.4 实现 src/models/memory.py：核心记忆数据模型（profile_content, notes_content, 容量信息）

## 4. 核心记忆系统

- [x] 4.1 实现 src/services/memory.py：核心记忆文件读写（user_profile.md + companion_notes.md）
- [x] 4.2 实现容量管理：字符计数、80% 阈值警告、超限拒绝写入
- [x] 4.3 实现 CRUD 操作：add（追加条目）、replace（替换条目）、remove（删除条目）
- [x] 4.4 实现 prompt 注入格式：带标题、容量百分比、内容的冻结块格式
- [x] 4.5 实现 src/api/memory.py：核心记忆 REST API（GET /api/memory/core, PATCH /api/memory/core）

## 5. 事件记忆系统

- [x] 5.1 实现 SQLite 初始化：events 表建表语句、数据目录下 events.db 自动创建
- [x] 5.2 实现事件写入：插入新事件记录
- [x] 5.3 实现事件检索：按 importance、event_type、时间范围过滤，支持 limit 参数
- [x] 5.4 实现事件删除：按 id 删除单条记录
- [x] 5.5 实现 src/api/memory.py 中的事件 API（GET /api/memory/events, DELETE /api/memory/events/{id}）

## 6. 人格初始化

- [x] 6.1 定义 MBTI 到八维权重的映射表（16 种 MBTI 类型的默认权重）
- [x] 6.2 实现初始化问卷处理逻辑：接收问卷回答，生成 user_profile.md
- [x] 6.3 实现初始权重计算：根据 MBTI 生成初始八维向量并持久化
- [x] 6.4 创建数据目录结构：data/{user_id}/、子目录（diaries/）、初始化 events.db
- [x] 6.5 实现 src/api/init.py：初始化 API（POST /api/init）

## 7. 情绪识别模块

- [x] 7.1 设计情绪识别 prompt：从对话中提取 emotions、event_type、importance、summary
- [x] 7.2 实现 src/services/emotion.py：调用 LLM 执行情绪识别，解析结构化 JSON 输出
- [x] 7.3 实现重要性评分校验：确保 importance 在 [0.0, 1.0] 范围内
- [x] 7.4 实现情绪标签白名单校验：仅接受预定义的 10 种情绪标签

## 8. 人格演化引擎

- [x] 8.1 实现 Reinforcement 机制：根据情绪识别结果调整基础权重（±0.03~0.05）
- [x] 8.2 实现 Compensation 机制：检测高压力场景，生成临时权重增量（仅当轮有效）
- [x] 8.3 设计 Reflection prompt：回顾近期事件 + 当前权重 → 输出新权重建议
- [x] 8.4 实现 Reflection 触发逻辑：每 10 轮或每日首次对话触发
- [x] 8.5 实现 Reflection 执行：调用 LLM、解析权重建议、持久化新基础权重
- [x] 8.6 实现 src/api/personality.py：人格状态 API（GET /api/personality）

## 9. 陪伴聊天服务

- [x] 9.1 设计 system prompt 模板：角色定义 + 记忆注入 + 人格权重描述
- [x] 9.2 实现 prompt 组装：按顺序拼接 system prompt + 核心记忆 + 权重 + 历史 + 用户消息
- [x] 9.3 实现对话历史管理：保留最近 20 轮，存储在内存/文件中
- [x] 9.4 实现对话后管线：情绪识别 → 事件写入 → 权重调整 → Reflection 检查 → 日记数据累积
- [x] 9.5 实现 src/api/chat.py：聊天 API（POST /api/chat）

## 10. 日记生成器

- [x] 10.1 设计日记生成 prompt：基于事件记忆和对话摘要生成结构化日记
- [x] 10.2 实现日记生成触发：每日首次对话时检查并生成昨日日记
- [x] 10.3 实现日记文件写入：data/{user_id}/diaries/YYYY-MM-DD.md，不覆盖已有文件
- [x] 10.4 实现 src/api/diary.py：日记 API（GET /api/diary/{date}, POST /api/diary/generate）

## 11. 集成与端到端测试

- [x] 11.1 编写人格初始化 → 聊天 → 记忆写入 → 日记生成的端到端流程测试
- [x] 11.2 验证人格权重在多轮对话中的演化行为
- [x] 11.3 验证核心记忆容量管理（压缩和拒绝写入）
- [x] 11.4 验证事件记忆的 CRUD 和检索功能
