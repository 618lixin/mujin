## 1. 后端：周成长总结

- [x] 1.1 实现 src/services/weekly_summary.py：周报生成逻辑（查询该周事件 + 日记，调用 LLM 生成）
- [x] 1.2 实现 src/api/summary.py：周报 API（POST /api/summary/weekly, GET /api/summary/weekly/{year}/{week}, GET /api/summary/weekly）
- [x] 1.3 注册 summary 路由到 main.py

## 2. 后端：人生章节生成

- [x] 2.1 实现 src/services/chapter.py：章节生成逻辑（查询时间段内事件 + 日记 + 周报，调用 LLM 生成叙事）
- [x] 2.2 实现 src/api/chapters.py：章节 API（POST /api/chapters/generate, GET /api/chapters, GET /api/chapters/{filename}）
- [x] 2.3 注册 chapters 路由到 main.py

## 3. 后端：日记管理增强

- [x] 3.1 扩展 src/api/diary.py：新增 GET /api/diary（列表查询，支持日期范围）
- [x] 3.2 新增 POST /api/diary/{date}/regenerate（重新生成，覆盖已有日记）
- [x] 3.3 新增 POST /api/diary/batch-generate（批量补生成）

## 4. 后端：人格权重历史

- [x] 4.1 在 src/services/event_memory.py 中新增 personality_snapshots 表建表语句
- [x] 4.2 在 src/services/personality_engine.py 的 run_reflection 中添加权重快照保存
- [x] 4.3 在 src/api/personality.py 中新增 GET /api/personality/history 端点

## 5. 前端：日记管理页面

- [x] 5.1 侧边栏新增"📖 日记"入口，创建日记面板
- [x] 5.2 实现日记列表视图（卡片形式，显示日期和摘要）
- [x] 5.3 实现日记详情展开（点击卡片查看完整内容）
- [x] 5.4 实现重新生成按钮（每条日记旁）
- [x] 5.5 实现批量补生成按钮
- [x] 5.6 实现周成长总结区域（查看已生成周报 + 生成本周总结按钮）

## 6. 前端：记忆管理页面

- [x] 6.1 重构侧边栏"🧠 记忆"为记忆管理面板
- [x] 6.2 实现核心记忆编辑器（textarea 编辑 user_profile 和 companion_notes，保存按钮）
- [x] 6.3 实现事件记忆列表（按类型筛选 + 删除按钮）
- [x] 6.4 实现人格权重历史时间线（每条记录显示时间、柱状图、反思摘要）
- [x] 6.5 实现人生章节区域（生成按钮 + 日期范围选择 + 已生成章节列表）

## 7. 集成验证

- [ ] 7.1 测试周报生成（有事件 vs 事件不足 3 条）
- [ ] 7.2 测试人生章节生成（有数据 vs 无数据）
- [ ] 7.3 测试日记列表、补生成、重新生成
- [ ] 7.4 测试人格权重历史记录和查询
- [ ] 7.5 前端全部页面功能走查
