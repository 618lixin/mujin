import json
import uuid
from datetime import datetime

from src.config import settings
from src.services.llm import stream_llm
from src.services.memory import load_core_memory, format_memory_for_prompt
from src.services.personality_service import load_weights, load_turn_counter, save_turn_counter
from src.services.personality_engine import (
    should_trigger_reflection,
    run_reflection,
)
from src.services.emotion import extract_emotion
from src.services.event_memory import add_event, save_conversation_turn
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


def prepare_chat(user_id: str, user_message: str) -> tuple[list[dict], list[dict], PersonalityWeights]:
    """准备聊天上下文，返回 (messages, history, base_weights)"""
    system_prompt, base_weights = build_system_prompt(user_id)
    history = _load_history(user_id)
    messages = [{"role": "system", "content": system_prompt}]
    messages.extend(history)
    messages.append({"role": "user", "content": user_message})
    return messages, history, base_weights


async def post_chat(
    user_id: str, user_message: str, ai_reply: str,
    history: list[dict], base_weights: PersonalityWeights,
) -> dict:
    """
    对话后管线（沉淀式，不直接改权重）：
    1. 保存对话历史
    2. 情绪识别 → 写入事件记忆（沉淀层）
    3. 累积日记数据（沉淀层）
    4. Reflection 检查（每周一次，唯一改权重的地方）
    """
    history.append({"role": "user", "content": user_message})
    history.append({"role": "assistant", "content": ai_reply})
    _save_history(user_id, history)

    turn_count = load_turn_counter(user_id) + 1
    save_turn_counter(user_id, turn_count)

    # 情绪识别 → 事件沉淀（不改权重）
    emotion_result = await extract_emotion(user_message, ai_reply)

    # 存储对话摘要（供跨会话搜索）
    save_conversation_turn(
        user_id, user_message, ai_reply,
        summary=emotion_result.summary or "",
        emotions=emotion_result.emotions,
    )

    if emotion_result.importance >= 0.6 and emotion_result.event_type:
        event = Event(
            id=str(uuid.uuid4())[:8],
            content=emotion_result.summary,
            emotions=emotion_result.emotions,
            importance=emotion_result.importance,
            event_type=emotion_result.event_type,
        )
        add_event(user_id, event)

    # 日记数据累积
    accumulate_day_data(user_id, emotion_result)

    # Reflection：每周一次，唯一修改权重的地方
    reflection_result = None
    if should_trigger_reflection(user_id):
        reflection_result = await run_reflection(user_id)

    return {
        "emotion": emotion_result.to_dict(),
        "turn_count": turn_count,
        "reflection": reflection_result,
    }


async def chat_turn(user_id: str, user_message: str) -> dict:
    """非流式完整对话（保留兼容）"""
    await check_and_generate_yesterday_diary(user_id)
    messages, history, base_weights = prepare_chat(user_id, user_message)

    from src.services.llm import call_llm
    ai_reply = await call_llm(messages)

    post_result = await post_chat(user_id, user_message, ai_reply, history, base_weights)
    return {"reply": ai_reply, **post_result}


async def chat_turn_stream(user_id: str, user_message: str):
    """流式对话：yield AI token，结束后 yield post_chat 结果"""
    await check_and_generate_yesterday_diary(user_id)
    messages, history, base_weights = prepare_chat(user_id, user_message)

    ai_reply = ""
    async for token in stream_llm(messages):
        ai_reply += token
        yield ("token", token)

    post_result = await post_chat(user_id, user_message, ai_reply, history, base_weights)
    yield ("done", post_result)
