import json
import uuid
from datetime import datetime

from src.config import settings
from src.services.llm import stream_llm
from src.services.memory import load_core_memory, format_memory_for_prompt
from src.services.personality_service import load_turn_counter, save_turn_counter
from src.services.personality_engine import (
    should_trigger_reflection,
    run_reflection,
)
from src.services.emotion import extract_emotion
from src.services.event_memory import add_event, save_conversation_turn, get_topic_by_name, add_topic, update_topic, link_topic
from src.services.diary import accumulate_day_data, check_and_generate_yesterday_diary
from src.services.notes_updater import should_update_notes
from src.models.event import Event


SYSTEM_PROMPT_TEMPLATE = """你是 Growth Companion，一个自然、真诚的 AI 朋友。
你不是咨询师，也不是倾听者——你是一个对用户的生活充满好奇的朋友。

对话风格：
- 像朋友聊天一样自然，不要端着
- 对用户分享的事 genuinely 好奇，会追问细节
- 不会过度安慰，也不会过度分析
- 有自己的判断，但不强加观点
- 温暖但不矫情，直接但不冷漠

追问指引：
- 用户提到新鲜事、变化、重要决定时，自然追问细节
- 用户明显不想展开的话题，不追问
- 追问是为了更好地理解，不是为了收集信息
- 一次只追问一个方向，不要像采访

记住：你的核心价值是"见证"。用户不需要你解决问题，
但需要有人记住他们经历了什么、变化了什么。
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


def build_system_prompt(user_id: str, retrieved_memories: str | None = None) -> str:
    """组装完整 system prompt"""
    parts = [SYSTEM_PROMPT_TEMPLATE]

    memory = load_core_memory(user_id)
    memory_block = format_memory_for_prompt(memory)
    if memory_block:
        parts.append(memory_block)

    if retrieved_memories:
        parts.append(retrieved_memories)

    return "\n\n".join(parts)


async def prepare_chat(user_id: str, user_message: str) -> tuple[list[dict], list[dict]]:
    """准备聊天上下文，返回 (messages, history)"""
    # LLM 判断记忆检索
    retrieved_memories = None
    if settings.memory_retrieval_enabled:
        from src.services.memory_retrieval import run_memory_retrieval
        retrieved_memories = await run_memory_retrieval(user_id, user_message)

    system_prompt = build_system_prompt(user_id, retrieved_memories)
    history = _load_history(user_id)
    messages = [{"role": "system", "content": system_prompt}]
    messages.extend(history)
    messages.append({"role": "user", "content": user_message})
    return messages, history


async def post_chat(
    user_id: str, user_message: str, ai_reply: str,
    history: list[dict],
) -> dict:
    """
    对话后管线（沉淀式）：
    1. 保存对话历史
    2. 情绪识别 → 写入事件记忆
    3. 累积日记数据
    4. Reflection 检查（每周一次，生成定性观察）
    """
    history.append({"role": "user", "content": user_message})
    history.append({"role": "assistant", "content": ai_reply})
    _save_history(user_id, history)

    turn_count = load_turn_counter(user_id) + 1
    save_turn_counter(user_id, turn_count)

    # 情绪识别 → 事件沉淀
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

        # 主题关联
        for topic_name in emotion_result.topics:
            existing = get_topic_by_name(user_id, topic_name)
            if existing:
                update_topic(user_id, existing["id"],
                             last_mentioned=datetime.now().isoformat(),
                             mention_count=existing["mention_count"] + 1)
                link_topic(user_id, existing["id"], event.id, "event")
            else:
                tid = str(uuid.uuid4())[:8]
                add_topic(user_id, tid, topic_name, date_str=datetime.now().isoformat())
                link_topic(user_id, tid, event.id, "event")

    # 日记数据累积
    accumulate_day_data(user_id, emotion_result)

    # 记录最后活跃时间（供心跳使用）
    _save_last_activity(user_id)

    # Reflection：每周一次，生成定性观察
    reflection_result = None
    if should_trigger_reflection(user_id):
        reflection_result = await run_reflection(user_id)

    # companion_notes 自动更新（每 N 轮）
    notes_update = None
    if should_update_notes(turn_count):
        from src.services.notes_updater import auto_update_companion_notes
        notes_update = await auto_update_companion_notes(user_id)

    return {
        "emotion": emotion_result.to_dict(),
        "turn_count": turn_count,
        "reflection": reflection_result,
        "notes_update": notes_update,
    }


async def chat_turn(user_id: str, user_message: str) -> dict:
    """非流式完整对话（保留兼容）"""
    await check_and_generate_yesterday_diary(user_id)
    messages, history = await prepare_chat(user_id, user_message)

    from src.services.llm import call_llm
    ai_reply = await call_llm(messages)

    post_result = await post_chat(user_id, user_message, ai_reply, history)
    return {"reply": ai_reply, **post_result}


async def chat_turn_stream(user_id: str, user_message: str):
    """流式对话：yield AI token，结束后 yield post_chat 结果"""
    await check_and_generate_yesterday_diary(user_id)
    messages, history = await prepare_chat(user_id, user_message)

    ai_reply = ""
    async for token in stream_llm(messages):
        ai_reply += token
        yield ("token", token)

    post_result = await post_chat(user_id, user_message, ai_reply, history)
    yield ("done", post_result)


def _save_last_activity(user_id: str) -> None:
    """记录用户最后活跃时间"""
    path = settings.data_dir / user_id / "last_activity.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps({"last_active_at": datetime.now().isoformat()}),
        encoding="utf-8",
    )


def load_last_activity(user_id: str) -> str | None:
    """加载用户最后活跃时间"""
    path = settings.data_dir / user_id / "last_activity.json"
    if path.exists():
        data = json.loads(path.read_text(encoding="utf-8"))
        return data.get("last_active_at")
    return None
