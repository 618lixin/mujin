import json
from datetime import date, timedelta
from pathlib import Path

from src.models.personality import PersonalityWeights
from src.services.llm import call_cheap_llm
from src.services.personality_service import (
    load_weights,
    save_weights,
    load_last_reflection_date,
    save_last_reflection_date,
)
from src.services.event_memory import query_events, save_personality_snapshot, save_insight, get_insights
from src.services.memory import patch_core_memory
from src.models.memory import MemoryPatch
from src.config import settings


# ============ Compensation（临时，当轮有效） ============
# 仅极端情绪场景，不持久化
STRESS_COMPENSATION = {
    "overwhelm": {"Te": 0.05, "Ti": 0.03},
}


def get_compensation(weights: PersonalityWeights, emotions: list[str]) -> PersonalityWeights:
    """Compensation：仅在极端情绪下生成临时权重增量（不持久化）"""
    compensation = {}
    for emotion in emotions:
        compensation.update(STRESS_COMPENSATION.get(emotion, {}))
    if compensation:
        return weights.apply_compensation(compensation)
    return weights


# ============ Reflection（人格唯一调整入口） ============

REFLECTION_PROMPT = """你是一个 AI 陪伴者的人格反思系统。
人格权重只能在反思时微调，调整幅度严格不超过 ±0.02。如果没有明显变化趋势，保持原权重。

=== 近期周报 ===
{weekly_summaries}

=== 近期重要事件 ===
{recent_events}

=== 近期日记摘要 ===
{diary_summaries}

=== 已有洞察 ===
{existing_insights}

=== 当前人格权重 ===
{current_weights}

请基于以上沉淀数据（周报、事件、日记），判断用户过去一周的需求趋势：
1. 用户最常表达的情绪是什么？
2. 用户更需要共情还是分析？
3. 是否有持续性模式（不是单次波动）？
4. 是否有新的洞察可以积累？（关于情绪模式、行为习惯、价值观变化等）

以 JSON 格式输出（不要输出其他内容）：
{{
  "analysis": "一句话分析（基于沉淀数据的趋势判断）",
  "new_weights": {{"Ti": 0.0, "Te": 0.0, "Fi": 0.0, "Fe": 0.0, "Si": 0.0, "Se": 0.0, "Ni": 0.0, "Ne": 0.0}},
  "insight": "关于用户的一个洞察（如有），无则为空字符串",
  "new_insights": [
    {{"category": "emotion_pattern|relationship|behavior|value|growth", "content": "具体洞察内容", "confidence": 0.5}}
  ]
}}
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
    执行 Reflection：唯一修改持久化权重的地方。
    输入：周报 + 事件 + 日记摘要（全部是沉淀后的数据）
    输出：微调权重（±0.02）+ 洞察
    """
    weights = load_weights(user_id)

    # 收集沉淀数据
    weekly_summaries = _get_weekly_summaries(user_id)
    diary_summaries = _get_recent_diary_summaries(user_id)

    # 已有洞察（让 LLM 能看到之前的认知，避免重复）
    existing = get_insights(user_id, limit=10)
    existing_text = "\n".join(f"- [{ins['category']}] {ins['content']} (置信度{ins['confidence']:.0%})"
                              for ins in existing) if existing else "暂无洞察"

    events = query_events(user_id, limit=10, min_importance=0.4)
    events_text = "\n".join(
        f"- [{e.created_at.strftime('%m-%d')}] {e.summary or e.content} ({', '.join(e.emotions)}, {e.importance:.0%})"
        for e in events
    ) if events else "近期无明显事件"

    prompt = REFLECTION_PROMPT.format(
        weekly_summaries=weekly_summaries,
        recent_events=events_text,
        diary_summaries=diary_summaries,
        existing_insights=existing_text,
        current_weights=json.dumps(weights.weights, ensure_ascii=False),
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

    # 更新权重（clamp 每个维度与之前的差值 ≤ 0.02）
    new_weights_data = data.get("new_weights", {})
    if new_weights_data:
        for dim in weights.weights:
            if dim in new_weights_data:
                old_val = weights.weights[dim]
                new_val = max(0.0, min(1.0, new_weights_data[dim]))
                # 强制限制单次调整幅度
                delta = new_val - old_val
                delta = max(-0.02, min(0.02, delta))
                weights.weights[dim] = old_val + delta
        save_weights(user_id, weights)

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
    save_last_reflection_date(user_id, date.today().isoformat())

    # 保存权重快照
    save_personality_snapshot(user_id, weights.weights, summary=data.get("analysis", ""))

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

    return {"analysis": data.get("analysis", ""), "insight": insight, "new_weights": weights.weights, "new_insights": new_insights}
