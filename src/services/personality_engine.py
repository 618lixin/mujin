import json
from datetime import date

from src.models.personality import PersonalityWeights
from src.services.llm import call_cheap_llm
from src.services.personality_service import (
    load_weights,
    save_weights,
    load_turn_counter,
    save_turn_counter,
    load_last_reflection_date,
    save_last_reflection_date,
)
from src.services.event_memory import query_events
from src.services.memory import patch_core_memory
from src.models.memory import MemoryPatch
from src.config import settings


# 情绪 → 增强维度的映射
EMOTION_REINFORCEMENT = {
    "sadness": {"Fe": 0.04},
    "anxiety": {"Fe": 0.03, "Ni": 0.02},
    "anger": {"Te": 0.03},
    "fear": {"Ni": 0.04},
    "overwhelm": {"Fe": 0.03, "Ti": 0.02},
    "joy": {"Fe": 0.02, "Ne": 0.02},
    "hope": {"Ne": 0.03, "Ni": 0.02},
    "calm": {"Si": 0.02},
    "surprise": {"Ne": 0.03},
    "disgust": {"Ti": 0.03},
}

# 高压力场景 → 临时补偿
STRESS_COMPENSATION = {
    "overwhelm": {"Te": 0.1, "Ti": 0.05},
    "anxiety": {"Ni": 0.08, "Te": 0.05},
}

# 需要分析/决策 → 增强 Te/Ti
ANALYTICAL_KEYWORDS = {"分析", "帮我决定", "怎么选", "建议", "决策", "利弊", "比较", "选择"}

REFLECTION_PROMPT = """你是一个 AI 陪伴者的人格反思系统。请根据以下信息评估并调整人格权重。

近期事件：
{recent_events}

当前人格权重：
{current_weights}

请思考：
1. 最近用户更需要什么？（共情/分析/实际建议/探索）
2. 当前权重是否匹配用户需求？
3. 是否需要调整？调整幅度不要超过 ±0.1。

以 JSON 格式输出（不要输出其他内容）：
{{
  "analysis": "一句话分析",
  "new_weights": {{"Ti": 0.0, "Te": 0.0, "Fi": 0.0, "Fe": 0.0, "Si": 0.0, "Se": 0.0, "Ni": 0.0, "Ne": 0.0}},
  "insight": "关于用户的一个洞察（如有），无则为空字符串"
}}
"""


def apply_reinforcement(weights: PersonalityWeights, emotions: list[str]) -> PersonalityWeights:
    """Reinforcement：根据情绪调整基础权重"""
    for emotion in emotions:
        adjustments = EMOTION_REINFORCEMENT.get(emotion, {})
        for dim, delta in adjustments.items():
            weights.adjust(dim, delta)
    return weights


def get_compensation(weights: PersonalityWeights, emotions: list[str]) -> PersonalityWeights:
    """Compensation：生成带临时补偿的权重副本（不修改原始）"""
    compensation = {}
    for emotion in emotions:
        compensation.update(STRESS_COMPENSATION.get(emotion, {}))
    if compensation:
        return weights.apply_compensation(compensation)
    return weights


def check_analytical_boost(user_message: str) -> dict[str, float]:
    """检查是否需要增强分析维度"""
    for kw in ANALYTICAL_KEYWORDS:
        if kw in user_message:
            return {"Ti": 0.03, "Te": 0.03}
    return {}


async def run_reflection(user_id: str) -> dict | None:
    """执行 Reflection：回顾近期事件，调整权重"""
    weights = load_weights(user_id)

    # 获取近期重要事件
    events = query_events(user_id, limit=5, min_importance=0.5)
    events_text = "\n".join(
        f"- [{e.created_at.strftime('%m-%d')}] {e.summary or e.content} ({', '.join(e.emotions)})"
        for e in events
    ) if events else "近期无明显事件"

    prompt = REFLECTION_PROMPT.format(
        recent_events=events_text,
        current_weights=json.dumps(weights.weights, ensure_ascii=False),
    )

    response = await call_cheap_llm(
        [{"role": "user", "content": prompt}],
        temperature=0.3,
        max_tokens=512,
    )

    try:
        text = response.strip()
        if text.startswith("```"):
            text = text.split("\n", 1)[1].rsplit("```", 1)[0].strip()
        data = json.loads(text)
    except (json.JSONDecodeError, IndexError):
        return None

    # 更新权重
    new_weights_data = data.get("new_weights", {})
    if new_weights_data:
        for dim in PersonalityWeights.__module__:
            pass
        weights.weights = {
            k: max(0.0, min(1.0, v))
            for k, v in new_weights_data.items()
            if k in weights.weights
        }
        save_weights(user_id, weights)

    # 保存洞察到 companion notes
    insight = data.get("insight", "")
    if insight:
        try:
            patch_core_memory(user_id, MemoryPatch(
                action="add", target="notes", content=f"[反思] {insight}"
            ))
        except ValueError:
            pass  # 容量不足，忽略

    # 记录反思日期
    save_last_reflection_date(user_id, date.today().isoformat())

    return {"analysis": data.get("analysis", ""), "insight": insight, "new_weights": weights.weights}


def should_trigger_reflection(user_id: str) -> bool:
    """检查是否应该触发 Reflection"""
    turn_count = load_turn_counter(user_id)
    last_date = load_last_reflection_date(user_id)
    today = date.today().isoformat()

    # 每 N 轮触发
    if turn_count > 0 and turn_count % settings.reflection_turn_interval == 0:
        return True

    # 每日首次对话触发
    if last_date != today:
        return True

    return False
