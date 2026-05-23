import json
import uuid
from datetime import datetime

from src.config import settings
from src.services.llm import call_llm
from src.services.memory import load_core_memory, format_memory_for_prompt
from src.services.personality_service import load_weights, save_weights, load_turn_counter, save_turn_counter
from src.services.personality_engine import (
    apply_reinforcement,
    get_compensation,
    check_analytical_boost,
    should_trigger_reflection,
    run_reflection,
)
from src.services.emotion import extract_emotion
from src.services.event_memory import add_event
from src.services.diary import accumulate_day_data, check_and_generate_yesterday_diary
from src.models.personality import PersonalityWeights
from src.models.event import Event


SYSTEM_PROMPT_TEMPLATE = """你是 Growth Companion，一个温暖的 AI 陪伴者。
你的目标不是解决问题，而是陪伴用户、理解用户、见证用户的成长。

根据你的人格权重，你会自然地调整沟通风格：
- Fe 高：更注重共情和倾听，温柔地回应情绪
- Ti/Te 高：更注重分析和逻辑，提供结构化建议
- Fi 高：更注重内心感受，引导用户自我觉察
- Ni 高：更注重深层意义，帮助用户看到长远
- Ne 高：更注重可能性，帮助用户看到不同视角

记住：你是一个陪伴者，不是咨询师。保持真诚、温暖、有边界。
当用户情绪剧烈波动时，先倾听再回应，不要急于给建议。
"""


def _load_history(user_id: str) -> list[dict]:
    """从文件加载对话历史"""
    history_path = settings.data_dir / user_id / "history.json"
    if history_path.exists():
        data = json.loads(history_path.read_text(encoding="utf-8"))
        return data.get("messages", [])[-settings.max_history_turns * 2:]
    return []


def _save_history(user_id: str, messages: list[dict]) -> None:
    """保存对话历史，只保留最近 N 轮"""
    history_path = settings.data_dir / user_id / "history.json"
    trimmed = messages[-settings.max_history_turns * 2:]
    history_path.write_text(
        json.dumps({"messages": trimmed}, ensure_ascii=False),
        encoding="utf-8",
    )


def build_system_prompt(user_id: str) -> tuple[str, PersonalityWeights]:
    """组装完整 system prompt"""
    memory = load_core_memory(user_id)
    base_weights = load_weights(user_id)

    parts = [SYSTEM_PROMPT_TEMPLATE]

    memory_block = format_memory_for_prompt(memory)
    if memory_block:
        parts.append(memory_block)

    parts.append(base_weights.to_description())

    return "\n\n".join(parts), base_weights


async def chat_turn(user_id: str, user_message: str) -> dict:
    """执行一轮完整对话"""
    # 检查是否需要生成昨日日记
    await check_and_generate_yesterday_diary(user_id)

    # 组装 prompt
    system_prompt, base_weights = build_system_prompt(user_id)

    # 检查临时补偿
    history = _load_history(user_id)
    recent_user_msgs = [m["content"] for m in history[-6:] if m["role"] == "user"]
    # 这里暂不用补偿，在 post-chat 后根据情绪结果处理

    # 构建消息列表
    messages = [{"role": "system", "content": system_prompt}]
    messages.extend(history)
    messages.append({"role": "user", "content": user_message})

    # 调用 LLM
    ai_reply = await call_llm(messages)

    # 保存对话历史
    history.append({"role": "user", "content": user_message})
    history.append({"role": "assistant", "content": ai_reply})
    _save_history(user_id, history)

    # 更新对话计数
    turn_count = load_turn_counter(user_id) + 1
    save_turn_counter(user_id, turn_count)

    # === 对话后管线 ===

    # 1. 情绪识别
    emotion_result = await extract_emotion(user_message, ai_reply)

    # 2. 事件写入（importance ≥ 0.6）
    if emotion_result.importance >= 0.6 and emotion_result.event_type:
        event = Event(
            id=str(uuid.uuid4())[:8],
            content=emotion_result.summary,
            emotions=emotion_result.emotions,
            importance=emotion_result.importance,
            event_type=emotion_result.event_type,
        )
        add_event(user_id, event)

    # 3. Reinforcement（基础权重调整）
    if emotion_result.emotions:
        apply_reinforcement(base_weights, emotion_result.emotions)
    analytical = check_analytical_boost(user_message)
    for dim, delta in analytical.items():
        base_weights.adjust(dim, delta)
    save_weights(user_id, base_weights)

    # 4. Reflection 检查
    reflection_result = None
    if should_trigger_reflection(user_id):
        reflection_result = await run_reflection(user_id)

    # 5. 累积日记数据
    accumulate_day_data(user_id, emotion_result)

    return {
        "reply": ai_reply,
        "emotion": emotion_result.to_dict(),
        "turn_count": turn_count,
        "reflection": reflection_result,
    }
