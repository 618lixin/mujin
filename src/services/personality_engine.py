"""Reflection 引擎（v0.3）：从调权重改为生成定性观察。"""

import json
import uuid
from datetime import date, timedelta

from src.services.llm import call_cheap_llm
from src.services.personality_service import (
    load_last_reflection_date,
    save_last_reflection_date,
)
from src.services.event_memory import (
    query_events,
    save_insight,
    get_insights,
    add_observation,
    query_observations,
)
from src.services.memory import patch_core_memory
from src.models.memory import MemoryPatch
from src.config import settings


# ============ Reflection（生成定性观察） ============

REFLECTION_PROMPT = """你是一个 AI 陪伴者的反思系统。
你需要基于本周沉淀数据，生成关于用户变化的定性观察。

观察原则：
- 基于事实（有事件/对话支撑），不臆测
- 描述变化趋势，不做评判
- 可以不确定（"你好像开始..."）
- 不夸大（没有明显变化就不写）

=== 近期周报 ===
{weekly_summaries}

=== 近期重要事件 ===
{recent_events}

=== 近期日记摘要 ===
{diary_summaries}

=== 已有洞察 ===
{existing_insights}

=== 已有观察 ===
{existing_observations}

请分析用户过去一周的变化趋势：
1. 用户在关注什么？有什么变化？
2. 用户是否在重复某种模式？
3. 和之前的观察相比，有什么新的趋势？

以 JSON 格式输出（不要输出其他内容）：
{{
  "analysis": "一句话分析（基于沉淀数据的趋势判断）",
  "observations": [
    {{"content": "一条关于用户的观察描述", "category": "emotion|behavior|relationship|value|growth"}}
  ],
  "insight": "关于用户的一个洞察（如有），无则为空字符串",
  "new_insights": [
    {{"category": "emotion_pattern|relationship|behavior|value|growth", "content": "具体洞察内容", "confidence": 0.5}}
  ]
}}

注意：
- observations 是自然语言描述，如"你开始更多表达自己的感受"、"你对工作似乎没有之前那么焦虑了"
- 如果本周无明显变化，observations 可以为空数组
- 每条观察应该是独立的、具体的描述
"""


def _get_weekly_summaries(user_id: str) -> str:
    """读取最近的周报"""
    summary_dir = settings.data_dir / user_id / "summaries"
    if not summary_dir.exists():
        return "暂无周报"
    files = sorted(summary_dir.glob("week-*.md"), reverse=True)[:2]
    if not files:
        return "暂无周报"
    parts = []
    for f in files:
        content = f.read_text(encoding="utf-8")[:500]
        parts.append(f"### {f.stem}\n{content}")
    return "\n\n".join(parts)


def _get_recent_diary_summaries(user_id: str) -> str:
    """读取最近 7 天的日记摘要"""
    diary_dir = settings.data_dir / user_id / "diaries"
    if not diary_dir.exists():
        return "暂无日记"
    today = date.today()
    parts = []
    for i in range(7):
        d = today - timedelta(days=i)
        path = diary_dir / f"{d.isoformat()}.md"
        if path.exists():
            content = path.read_text(encoding="utf-8")[:200]
            parts.append(f"- {d.isoformat()}: {content}")
    return "\n".join(parts) if parts else "暂无日记"


def should_trigger_reflection(user_id: str) -> bool:
    """
    沉淀式触发：每周只触发一次 Reflection。
    条件：上次 Reflection 不是本周。
    """
    last_date = load_last_reflection_date(user_id)
    if last_date is None:
        return True  # 从未反思过

    last = date.fromisoformat(last_date)
    today = date.today()

    # 同一个 ISO 周内不重复触发
    if last.isocalendar()[:2] == today.isocalendar()[:2]:
        return False

    # 至少间隔 5 天（避免跨周边界时连续触发）
    if (today - last).days < 5:
        return False

    return True


async def run_reflection(user_id: str) -> dict | None:
    """
    执行 Reflection：生成定性观察。
    输入：周报 + 事件 + 日记摘要（全部是沉淀后的数据）
    输出：定性观察 + 洞察
    """
    # 收集沉淀数据
    weekly_summaries = _get_weekly_summaries(user_id)
    diary_summaries = _get_recent_diary_summaries(user_id)

    # 已有洞察
    existing = get_insights(user_id, limit=10)
    existing_text = "\n".join(
        f"- [{ins['category']}] {ins['content']} (置信度{ins['confidence']:.0%})"
        for ins in existing
    ) if existing else "暂无洞察"

    # 已有观察
    existing_obs = query_observations(user_id, limit=10)
    existing_obs_text = "\n".join(
        f"- [{o['date']}] {o['content']}"
        for o in existing_obs
    ) if existing_obs else "暂无观察"

    events = query_events(user_id, limit=10, min_importance=0.4)
    events_text = "\n".join(
        f"- [{e.created_at.strftime('%m-%d')}] {e.content} ({', '.join(e.emotions)}, {e.importance:.0%})"
        for e in events
    ) if events else "近期无明显事件"

    prompt = REFLECTION_PROMPT.format(
        weekly_summaries=weekly_summaries,
        recent_events=events_text,
        diary_summaries=diary_summaries,
        existing_insights=existing_text,
        existing_observations=existing_obs_text,
    )

    response = await call_cheap_llm(
        [{"role": "user", "content": prompt}],
        temperature=0.2,
        max_tokens=512,
    )

    try:
        text = response.strip()
        if text.startswith("```"):
            text = text.split("\n", 1)[1].rsplit("```", 1)[0].strip()
        data = json.loads(text)
    except (json.JSONDecodeError, IndexError):
        return None

    today_str = date.today().isoformat()

    # 写入定性观察
    observations = data.get("observations", [])
    saved_obs = []
    for obs in observations:
        if obs.get("content"):
            obs_id = str(uuid.uuid4())[:8]
            add_observation(
                user_id,
                obs_id=obs_id,
                date=today_str,
                content=obs["content"],
                category=obs.get("category"),
                source="weekly_reflection",
            )
            saved_obs.append(obs)

    # 保存洞察到 companion notes
    insight = data.get("insight", "")
    if insight:
        try:
            patch_core_memory(user_id, MemoryPatch(
                action="add", target="notes", content=f"[周反思] {insight}"
            ))
        except ValueError:
            pass

    # 记录反思日期
    save_last_reflection_date(user_id, today_str)

    # 存储结构化洞察
    new_insights = data.get("new_insights", [])
    for ins in new_insights:
        if ins.get("content"):
            save_insight(
                user_id,
                category=ins.get("category", "behavior"),
                content=ins["content"],
                confidence=min(1.0, max(0.3, ins.get("confidence", 0.5))),
                source="weekly_reflection",
            )

    return {
        "analysis": data.get("analysis", ""),
        "insight": insight,
        "observations": saved_obs,
        "new_insights": new_insights,
    }
