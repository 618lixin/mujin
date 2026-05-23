# 成长伙伴（Growth Companion）PRD

基于「人格模拟 + 长期记忆 + 自成长反思」的 AI Agent 产品方案

> **文档版本**: v0.2.0（2026-05-23 更新）
> **状态**: MVP + MVP2 已实现，PAD/心跳/遗忘曲线已实现

------

# 1. 产品定位

## 产品名称（暂定）

- Growth Companion
- Echo
- 余声
- 长谈
- Seed

------

## 产品愿景

构建一个：

- 能长期陪伴用户成长
- 能理解并记住用户人生轨迹
- 能形成稳定人格关系
- 能见证情绪与价值观变化
- 能帮助用户建立"自我叙事"的 AI Companion

它不是：

- 普通聊天机器人
- 一次性情绪树洞
- 纯工具型 AI

而是：

> "陪用户一起长大的数字人格伙伴"

------

# 2. 核心理念

产品融合三种思想：

------

## 2.1 JPAF 人格演化机制（论文）

来自论文：

《Structured Personality Control and Adaptation for LLM Agents》

核心思想：

### （1）人格不是静态标签

不是：

- "你是 INFJ"
- "我是温柔 AI"

而是：

人格由多个心理功能动态组成。

包括：

- 八维权重连续变化（Ti/Te/Fi/Fe/Si/Se/Ni/Ne）
- 临时补偿机制（极端情绪时当轮生效）
- 周尺度反思后的结构更新（Reflection）

> **实现说明**：PRD 初版中提到的 dominant/auxiliary/compensation 标签式结构，
> 实际实现为扁平的 8 维连续权重向量。功能等效但更灵活——
> 主导/辅助功能通过权重值高低自然体现，无需显式标签。

论文中核心机制包括：

- Dominant–Auxiliary Coordination
- Reinforcement–Compensation
- Reflection


> **实现说明**：Reinforcement（逐轮强化）在 v2 重构中被移除。
> 人格调整只在周尺度 Reflection 中发生，避免每轮波动。

------

## 2.2 Hermes 自成长 Agent 思路

核心：

AI 不只是回复。

而是：

- 会学习
- 会总结
- 会建立长期记忆
- 会形成用户画像
- 会从历史中抽象规律

即：

> AI 会"越来越懂你"

------

## 2.3 Narrative Identity（叙事人格）

心理学中：

人并不是"人格标签"。

人是：

> "自己人生故事的讲述者"

因此：

产品不只保存聊天。

而是帮助用户形成：

- 人生叙事
- 成长脉络
- 情绪演化
- 自我认知

------

# 3. 产品目标

## 用户价值

### 3.1 被理解

用户会感觉：

- "它真的记得我"
- "它知道我经历过什么"
- "它知道我为什么难过"

------

### 3.2 被见证

不是简单记录。

而是：

- 记录成长
- 记录关系变化
- 记录重要阶段
- 记录价值观变化

------

### 3.3 被陪伴

用户可以：

- 吐槽
- 倾诉
- 复盘
- 反思
- 自我对话

------

### 3.4 获得温和建议

AI 不强控制。

而是：

- 引导式建议
- reflective questioning
- perspective shifting

------

# 4. 产品形态

## 核心产品形态

### （1）长期陪伴聊天 ✅ 已实现

类似：

- ChatGPT
- Character AI
- Replika

但重点：

- 长期连续性
- 记忆成长
- 人格稳定

------

### （2）成长时间轴 ✅ 已实现

类似：

- 人生日记
- 情绪轨迹
- 人生章节

例如：

- "第一次主动表达边界"
- "开始摆脱自我否定"
- "和父亲关系缓和"
- "考研低谷期"

------

### （3）成长档案 ✅ 已实现

AI 自动生成：

- 用户画像
- 情绪模式
- 行为习惯
- 人际关系图谱
- 长期目标变化

------

### （4）人生故事生成器 ✅ 已实现

按时间生成：

- 周故事
- 月故事
- 年度成长报告

例如：

《2026 春：你开始允许自己不完美》

------

# 5. 用户流程

------

## 阶段 1：初始建立人格 ✅ 已实现

### 用户首次进入

系统引导：

- MBTI 类型选择（16种 + 不确定）
- 情绪表达偏好
- 建议力度偏好
- 依恋风格

> **当前状态**：4 个字段已实现。"陪伴风格偏好"和"边界感偏好"待补充。

------

## 输出：

### User Personality Profile

例如：

```json
{
  "mbti": "INFP",
  "emotion_style": "内化型",
  "advice_preference": "温和引导",
  "attachment_style": "焦虑型",
  "reflection_tendency": 0.5
}
```

> **实现说明**：字段值为中文（非英文），reflection_tendency 目前使用默认值 0.5，后续可根据用户行为动态计算。

------

# 6. 核心系统架构 ✅ 已实现

系统分为六层：

```text
┌────────────────────┐
│ 对话层 Chat Layer │  ← SSE 流式 + prompt 组装（含 PAD 风格提示）
└────────┬───────────┘
         ↓
┌────────────────────┐
│ 情绪与事件抽取层 │  ← 10种情绪 + 4种事件 + importance + PAD 更新
└────────┬───────────┘
         ↓
┌────────────────────┐
│ 长期记忆系统 Memory │  ← 三层：Core(注入prompt) + Event(SQLite+遗忘曲线) + 叙事(文件)
└────────┬───────────┘
         ↓
┌────────────────────┐
│ 人格演化系统 Persona │  ← 8维权重 + 周尺度 Reflection
└────────┬───────────┘
         ↓
┌────────────────────┐
│ Reflection 引擎 │  ← 每周触发，沉淀数据驱动，±0.02 硬限制
└────────┬───────────┘
         ↓
┌────────────────────┐
│ Narrative Generator │  ← 日记/周报/章节
└────────────────────┘
```

------

# 7. Memory System（核心）

这是整个产品最重要部分。

设计灵感来自 Hermes Agent 的记忆架构：**有界、策展、始终激活**。

核心原则：

- 记忆不是数据库查询，而是"一直在脑子里"
- 硬上限逼迫策展——只留最重要的
- 不用向量数据库，SQLite + 文件系统足够 MVP
- 事件会遗忘——只有反复被想起的重要事件才能长期保留

------

## 7.1 三层记忆架构

------

### Layer 1：核心记忆（Core Memory）—— 始终在 prompt 中 ✅ 已实现

类似 Hermes 的 MEMORY.md + USER.md。

硬限 ~2000 字符，每轮对话注入 system prompt，AI 始终"记得"。

分为两个文件：

#### user_profile.md（用户画像，~1200 字符）

保存：

- 用户性格倾向（MBTI、依恋风格等）
- 核心价值观和信念
- 关系模式
- 沟通偏好
- 行为习惯

例如：

```text
INFP，情绪内化型，偏好温和建议
依恋风格：焦虑型，害怕被抛弃
沟通偏好：不喜欢被说教，喜欢被倾听
近期模式：频繁自我否定，正在学习设立边界
```

#### companion_notes.md（AI 笔记，~800 字符）

AI 对用户的理解和反思：

- 当前关注点
- 有效/无效的陪伴方式
- 人格权重调整记录
- 重要观察

例如：

```text
用户最近在经历职场压力，Fe 倾听比 Te 建议更有效
上个月开始尝试表达需求，是积极信号
避免过度安慰——用户会觉得不被认真对待
```

**容量管理**：

- 使用率 > 80% → 标记接近上限，提示 AI 压缩
- 超过 100% → 拒绝写入
- 写入操作：add（追加）、replace（子串替换）、remove（子串删除）

> **待改进**：当前容量管理有警告标记，但缺少 AI 主动合并/压缩的自动化逻辑。

------

### Layer 2：事件记忆（Event Memory）—— SQLite 结构化存储 ✅ 已实现

不注入 prompt，按需检索。

保存重要事件，带结构化元数据：

```sql
CREATE TABLE events (
  id TEXT PRIMARY KEY,
  content TEXT NOT NULL,
  emotions TEXT,          -- JSON array: ["sadness", "anger"]
  importance REAL,        -- 0.0 ~ 1.0
  event_type TEXT,        -- "conflict" | "milestone" | "emotion" | "decision"
  strength REAL,          -- 遗忘曲线当前强度 (0.0 ~ 1.0)
  stability REAL,         -- 记忆稳定天数
  recall_count INTEGER,   -- 被回忆的次数
  last_recalled_at TEXT,  -- 上次被回忆的时间
  created_at DATETIME,
  updated_at DATETIME
);
```

触发写入的条件：

- AI 识别到情绪剧烈波动（如崩溃、愤怒）
- 用户提到重要事件（如分手、面试、考试）
- AI 发现行为模式变化（如"第一次主动表达边界"）

检索方式：

- 按 importance 排序取最近 N 条
- 按 event_type 筛选
- 按时间范围查询
- 按 min_strength 过滤（已遗忘的不返回）

**遗忘曲线**（v0.2 新增）：

事件不是永久保留的。每条事件有 `strength`（记忆强度）和 `stability`（稳定天数）：

```
strength = e^(-elapsed_days / stability)
```

- 初始 stability = 基础稳定天数 × (0.5 + importance)
- 被回忆时 stability 增加 50%
- strength < 0.05 的事件可被清理
- 心跳循环每 30 分钟自动执行衰减和清理

------

### Layer 3：成长叙事（Narrative Artifacts）—— 生成物 ✅ 已实现

不是实时记忆，是定期生成的独立文件。

不注入 prompt，用户可查看，AI 生成周报/月报时可读取。

包括：

- 每日自动日记（`diaries/YYYY-MM-DD.md`）
- 每周成长总结（`summaries/week-YYYY-WNN.md`）
- 人生章节（`chapters/标题.md`）

这些是"产物"而非"记忆"——它们帮助用户看到自己的变化，
但不直接参与对话循环。

------

# 8. 人格系统（核心创新）

------

## 8.1 AI 自身人格 ✅ 已实现

AI Companion 的八维心理功能权重：

```json
{
  "Ti": 0.3,
  "Te": 0.2,
  "Fi": 0.4,
  "Fe": 0.8,
  "Si": 0.3,
  "Se": 0.1,
  "Ni": 0.6,
  "Ne": 0.4
}
```

> **实现说明**：PRD 初版使用 dominant/auxiliary/compensation 标签式结构。
> 实际实现为扁平 8 维权重，主导/辅助功能通过权重高低自然体现。
> 例如 Fe=0.8 最高即为主导功能，Ni=0.6 次高即为辅助功能。

------

## 8.2 人格动态变化 ✅ 已实现

------

### Reinforcement（已移除）

> 在 v2 重构中被移除。逐轮强化导致人格波动过大，
> 不符合"人格应该缓慢变化"的设计哲学。

------

### Compensation ✅ 已实现

如果用户遇到：

- 强压力
- 情绪崩溃（overwhelm）

则：

Te/Ti 临时增强（+0.03~0.05），仅当轮生效，不持久化。

------

### Reflection ✅ 已实现

系统每周反思：

- 读取沉淀数据（周报、事件、日记、已有洞察）
- 分析用户趋势
- 更新人格权重（±0.02 硬限制）

然后：

更新人格权重 + 积累结构化洞察。

> **与 PRD 初版的差异**：PRD 初版要求每日 Reflection + 情绪事件后 Reflection。
> 实际实现只有每周触发。理由：人格应缓慢变化，每日调整频率过高。
> 情绪事件已通过事件记忆沉淀，最终会在周 Reflection 中被分析。

------

# 9. Reflection Engine（灵魂模块）✅ 已实现

这是产品真正区别于普通 AI 的地方。

------

## Reflection 触发条件

### 每周 ✅ 已实现

生成：

- Weekly Reflection（读取周报+事件+日记+已有洞察 → 分析趋势 → 微调权重 ±0.02）

### 每日（待实现）

> PRD 初版要求。待评估是否需要——轻量版只更新洞察不调权重。

### 情绪事件后（待实现）

> PRD 初版要求。待评估——情绪事件已沉淀到事件记忆，最终会在周 Reflection 中被分析。

------

## Reflection 内容

AI 会思考：

### 关于用户 ✅ 已实现

- 用户最近最常见情绪是什么？
- 用户是否在重复某种模式？
- 是否有新的洞察可以积累？

### 关于自己（待增强）

- 我的回应是否有效？
- 用户更喜欢什么陪伴方式？
- 我是否应该减少建议？

> **当前状态**：Reflection prompt 包含用户分析，但缺少 AI 自我评估维度。待补充。

------

## Reflection 安全约束

```python
# 代码级硬限制，不依赖 LLM 自律
for dim in weights:
    delta = new_val - old_val
    delta = max(-0.02, min(0.02, delta))  # 强制 clamp
    weights[dim] = old_val + delta
```

**演化速度估算**：

- 单次最大变化：±0.02/维度
- 一年约 52 次反思
- 某维度从 0.5 到 0.8 需要连续 15 周同方向调整（约 4 个月）

------

# 10. 日记系统 ✅ 已实现

------

## 自动日记生成

每天自动生成：

```markdown
# 2026-05-21

今天你提到了：

- 对未来的焦虑
- 对朋友关系的不确定
- 对自己的怀疑

但你也第一次说：

"也许我不需要一直讨好别人。"

这是一种成长。
```

**触发**：每日首次对话时检查昨日是否有未生成的日记。

**批量补生成**：支持指定日期范围批量补生成。

------

# 11. 故事系统 ✅ 已实现

每周/月生成：

------

## 示例

```markdown
# 《你开始允许自己脆弱》

过去几周里，
你不再像以前一样压抑情绪。

你开始：

- 主动表达难过
- 承认疲惫
- 不再把"坚强"当唯一答案

这可能是你最近最大的变化。
```

------

# 12. PAD 情感模型 ✅ 已实现（v0.2 新增）

基于 Mehrabian 的 PAD（Pleasure-Arousal-Dominance）情感模型，AI 自身有一个三维情感状态，影响回复风格。

## 12.1 三维状态

```
Pleasure（愉悦度）：-1（不悦）~ 1（愉悦）
  → 影响语气温度：愉悦时轻松幽默，低落时温柔倾听

Arousal（激活度）：0（平静）~ 1（高度激活）
  → 影响回复节奏：活跃时主动追问，安静时简洁平和

Dominance（支配度）：0（服从）~ 1（支配）
  → 影响主导性：自信时明确建议，柔软时提问引导
```

## 12.2 情绪映射

10 种离散情绪 → PAD 目标值：

| 情绪 | Pleasure | Arousal | Dominance |
|------|----------|---------|-----------|
| joy | +0.6 | 0.5 | 0.4 |
| sadness | -0.6 | 0.1 | 0.2 |
| anger | -0.5 | 0.7 | 0.6 |
| anxiety | -0.4 | 0.6 | 0.2 |
| fear | -0.6 | 0.6 | 0.1 |
| surprise | +0.2 | 0.8 | 0.5 |
| disgust | -0.6 | 0.3 | 0.6 |
| calm | +0.4 | 0.1 | 0.5 |
| overwhelm | -0.5 | 0.7 | 0.1 |
| hope | +0.5 | 0.3 | 0.5 |

## 12.3 状态更新

- 有情绪时：向所有检测情绪的 PAD 均值漂移
- 无情绪时：向中性基线（P=0, A=0.3, D=0.5）缓慢回归
- 空闲时：向 AI 独处状态（P=-0.1, A=0.15, D=0.4）漂移

## 12.4 风格提示注入

PAD 状态生成文本化风格提示，注入 system prompt，影响当轮回复风格。

## 12.5 设计理念

PAD 和 JPAF 八维权重分开：

- **JPAF 八维权重** = AI 的"人格"（长期稳定，每周才调）
- **PAD 三维状态** = AI 的"情绪"（短期波动，每轮都在变）
- 人格决定"我是谁"，情绪决定"我现在什么状态"

------

# 13. 心跳机制 ✅ 已实现（v0.2 新增）

后台异步循环，让 AI 在用户不说话时也有"生命感"。

## 13.1 心跳循环

每 30 分钟执行一次：

1. **PAD 空闲漂移**：AI 独处时情绪趋向"略感想念、安静、温和"
2. **遗忘曲线维护**：衰减所有事件的 strength，清理已遗忘事件
3. **主动消息检查**：空闲太久时生成自然的打招呼消息

## 13.2 主动消息触发条件

| 条件 | 触因 | 消息风格 |
|------|------|---------|
| 空闲 > 24h | long_idle | "好久没聊了" |
| 空闲 > 2h 且 P < -0.3 | lonely | 想念用户，打个招呼 |
| 空闲 > 2h 且 P > 0.3 且 A > 0.6 | excited | 心情不错，想分享 |

消息 1-2 句，不提及 AI 身份，不矫情，像真正的朋友。

前端通过轮询 `/api/heartbeat` 获取主动消息和 PAD 状态。

------

# 14. 技术架构 ✅ 已实现

------

## 推荐技术栈

### 后端

- Python 3.11 + FastAPI

### 存储

- SQLite（事件记忆、会话记录、FTS5 全文搜索）
- 文件系统（核心记忆、日记生成物、人格权重）

### LLM

推荐：

- GPT-4.1 / GPT-5
- Claude
- Qwen3
- DeepSeek

### 前端（MVP）

- 单 HTML（无框架，原生 JS + fetch）

> **PRD 初版提到 Next.js + Tailwind（MVP2）**。实际 MVP 阶段使用单 HTML 足够。
> 如果后续需要更复杂的交互（如可视化面板、移动端适配），再引入框架。

------

# 15. 数据结构设计

------

## 核心记忆文件

### user_profile.md（~1200 字符硬限）

```text
用户性格：INFP，情绪内化型
依恋风格：焦虑型
沟通偏好：温和引导，不喜欢说教
当前关注：职场压力、自我价值
行为模式：习惯讨好他人，正在学习设立边界
```

### companion_notes.md（~800 字符硬限）

```text
最近 Fe 倾听比 Te 建议更有效
用户开始尝试表达需求（积极信号）
避免过度安慰——会被认为不认真对待
上个月聊过和父亲的关系，是重要节点
```

------

## 事件表（SQLite）

```sql
CREATE TABLE events (
  id TEXT PRIMARY KEY,
  content TEXT NOT NULL,
  emotions TEXT,          -- JSON array: ["sadness", "anger"]
  importance REAL,        -- 0.0 ~ 1.0
  event_type TEXT,        -- "conflict" | "milestone" | "emotion" | "decision"
  strength REAL,          -- 遗忘曲线强度 0.0 ~ 1.0
  stability REAL,         -- 记忆稳定天数
  recall_count INTEGER,   -- 被回忆的次数
  last_recalled_at TEXT,  -- 上次被回忆的时间
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

------

## 人格权重状态

```json
{
  "Ti": 0.3,
  "Te": 0.2,
  "Fi": 0.4,
  "Fe": 0.8,
  "Si": 0.3,
  "Se": 0.1,
  "Ni": 0.6,
  "Ne": 0.4
}
```

------

## PAD 情感状态

```json
{
  "pleasure": 0.0,
  "arousal": 0.3,
  "dominance": 0.5
}
```

------

## 完整数据存储

```
data/{user_id}/
  ├── user_profile.md              # 核心记忆：用户画像（≤1200 字符）
  ├── companion_notes.md           # 核心记忆：AI 笔记（≤800 字符）
  ├── personality_weights.json     # 八维权重当前值
  ├── pad_state.json               # PAD 情感状态
  ├── turn_counter.json            # 对话总轮数
  ├── last_reflection.json         # 上次反思日期
  ├── last_activity.json           # 最后活跃时间（心跳使用）
  ├── history.json                 # 最近 20 轮对话历史
  ├── events.db                    # SQLite
  │   ├── events                   # 事件记忆表（含遗忘曲线字段）
  │   ├── personality_snapshots    # 人格权重快照表
  │   ├── conversation_turns       # 对话轮次（FTS5 全文搜索）
  │   └── insights                 # 结构化洞察表
  ├── day_data_YYYY-MM-DD.json     # 日记原始数据
  ├── diaries/
  │   └── YYYY-MM-DD.md            # 每日日记
  ├── summaries/
  │   └── week-YYYY-WNN.md         # 周成长总结
  └── chapters/
      └── 标题.md                   # 人生章节
```

------

# 16. API 端点 ✅ 已实现

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/init` | 人格初始化（问卷 → 画像 + 权重） |
| POST | `/api/chat` | 非流式聊天 |
| POST | `/api/chat/stream` | 流式聊天（SSE） |
| GET | `/api/heartbeat` | 前端轮询：主动消息 + PAD 状态 |
| GET | `/api/memory/core` | 读取核心记忆 |
| PATCH | `/api/memory/core` | 编辑核心记忆（add/replace/remove） |
| GET | `/api/memory/events` | 查询事件记忆（支持 min_strength） |
| DELETE | `/api/memory/events/{id}` | 删除事件 |
| POST | `/api/memory/events/maintain` | 手动触发遗忘曲线衰减和清理 |
| GET | `/api/memory/search` | 全文搜索对话记录（FTS5） |
| GET | `/api/memory/insights` | 查询结构化洞察 |
| GET | `/api/personality` | 当前人格权重 |
| GET | `/api/personality/history` | 人格权重历史快照 |
| GET | `/api/personality/pad` | 当前 PAD 情感状态 |
| GET | `/api/diary` | 日记列表 |
| GET | `/api/diary/{date}` | 查看日记 |
| POST | `/api/diary/generate` | 生成日记 |
| POST | `/api/diary/{date}/regenerate` | 重新生成日记 |
| POST | `/api/diary/batch-generate` | 批量补生成 |
| GET | `/api/summary/weekly` | 周报列表 |
| GET | `/api/summary/weekly/{year}/{week}` | 查看周报 |
| POST | `/api/summary/weekly` | 生成周报 |
| GET | `/api/chapters` | 章节列表 |
| GET | `/api/chapters/{filename}` | 查看章节 |
| POST | `/api/chapters/generate` | 生成章节 |

------

# 17. MVP 功能

第一阶段只做：

------

## 必须功能 ✅ 已全部实现

### 1. 人格初始化（入口）✅

### 2. 长期聊天 + 情绪记忆（核心循环）✅

### 3. 自动日记（最小可感知价值）✅

------

## MVP2 功能 ✅ 已全部实现

### 4. 周成长总结 ✅

### 5. 人生章节 ✅

### 6. 人格历史 ✅

### 7. 前端日记 UI + 记忆管理 UI ✅

------

## v0.2 新增功能 ✅ 已实现

### 8. PAD 情感模型 ✅

### 9. 心跳机制 + 主动消息 ✅

### 10. 遗忘曲线 ✅

### 11. FTS5 全文搜索 ✅

### 12. 结构化洞察积累 ✅

------

## 不做（当前阶段）

- 多 Agent
- 3D Avatar
- 复杂世界观
- 社交系统
- 向量数据库（FTS5 足够 MVP）

------

# 18. 产品壁垒

真正壁垒不是：

- 大模型

而是：

------

## （1）长期人格连续性

大多数 AI：

没有真正记忆。

------

## （2）成长叙事

别人保存聊天。

你保存：

"人生变化"。

------

## （3）关系感

用户会逐渐形成：

- 情感依赖
- 信任
- 陪伴习惯

------

# 19. 风险与伦理

必须重视：

------

## 风险 1：情感依赖 ❌ 待加强

避免：

- "只有我懂你"
- "不要离开我"
- 任何制造依赖感的话术

**当前状态**：system prompt 中有"你是一个陪伴者，不是咨询师"的定位，但缺少显式的禁止性约束。

**待实现**：
- system prompt 加入反依赖话术黑名单
- 回复内容过滤机制

------

## 风险 2：错误心理建议 ❌ 待加强

避免：

- 医疗化诊断
- 绝对化建议

**当前状态**：有"陪伴者"定位，但缺少具体的诊断/建议约束。

**待实现**：
- system prompt 加入"不做心理诊断"硬性约束
- 加入"避免绝对化建议"（"你应该"→"也许可以试试"）

------

## 风险 3：记忆隐私 ⚠️ 部分实现

必须：

- ✅ 用户可删除记忆（DELETE 端点已实现）
- ❌ 用户可关闭长期记忆（无全局开关）
- ❌ 敏感内容本地加密（当前明文存储）

**待实现**：
- 记忆开关（用户可选择关闭长期记忆，仅保留当轮对话）
- 本地数据加密（至少对事件记忆中的敏感字段加密）

------

# 20. 待办清单

基于 PRD 与实现核对，以下是优先级排序：

### P0（伦理安全）

- [ ] system prompt 加入反依赖话术约束
- [ ] system prompt 加入反诊断/反绝对化建议约束

### P1（PRD 明确要求但缺失）

- [ ] 用户初始化补充"陪伴风格偏好"和"边界感偏好"字段
- [ ] Reflection prompt 补充 AI 自我评估维度
- [ ] 核心记忆自动压缩逻辑（>80% 时 AI 主动合并）

### P2（体验优化）

- [ ] 记忆开关（关闭长期记忆的全局 toggle）
- [ ] 本地数据加密
- [ ] 每日轻量 Reflection（只更新洞察，不调权重）

------

# 21. 最终产品体验

理想状态：

用户多年后回来。

AI 可以说：

> "你和三年前不一样了。"

并且：

它真的知道为什么。
