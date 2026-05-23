"""companion_notes 自动更新：定期分析近期事件，更新 AI 关于用户的陪伴笔记。"""

import json

from src.config import settings
from src.services.llm import call_cheap_llm
from src.services.memory import load_core_memory, patch_core_memory
from src.services.event_memory import query_events
from src.models.memory import MemoryPatch


_NOTES_UPDATE_PROMPT = """你是一个 AI 陪伴者的自我反思系统。
你需要根据最近的对话情况，更新你关于"如何与这位用户相处"的笔记。

=== 当前陪伴笔记 ===
{current_notes}

=== 最近重要事件和对话 ===
{recent_events}

请分析最近的事件，判断是否有新的行为模式或互动偏好需要记录。

注意：
- 只记录稳定的模式，不记录偶发事件
- 笔记内容是关于"如何更好地陪伴这个用户"的指导
- 保持简洁，每条不超过50字
- 如果笔记已接近容量上限，优先用 replace 更新旧内容

如果无需更新，返回 action 为 "none"。

严格以 JSON 格式输出（不要输出其他内容）：
{{
  "action": "add" 或 "replace" 或 "none",
  "old_text": "需要替换的旧文本（replace 操作必填）",
  "content": "新内容（add/replace 操作必填）"
}}"""


def should_update_notes(turn_count: int) -> bool:
    """判断当前轮数是否应触发笔记更新"""
    if not settings.notes_auto_update_enabled:
        return False
    if turn_count < settings.notes_auto_update_interval:
        return False
    return turn_count % settings.notes_auto_update_interval == 0


async def auto_update_companion_notes(user_id: str) -> str | None:
    """分析近期事件，按需更新 companion_notes"""
    try:
        memory = load_core_memory(user_id)
        current_notes = memory.notes_content or "(空)"

        events = query_events(user_id, limit=10, min_importance=0.3)
        events_text = "\n".join(
            f"- [{e.created_at.strftime('%m-%d')}] {e.content}"
            for e in events
        ) if events else "无近期事件"

        prompt = _NOTES_UPDATE_PROMPT.format(
            current_notes=current_notes,
            recent_events=events_text,
        )

        response = await call_cheap_llm(
            [{"role": "user", "content": prompt}],
            temperature=0.2,
            max_tokens=256,
        )

        text = response.strip()
        if text.startswith("```"):
            text = text.split("\n", 1)[1].rsplit("```", 1)[0].strip()

        data = json.loads(text)
        action = data.get("action", "none")

        if action == "none":
            return None

        if action == "add" and data.get("content"):
            patch_core_memory(user_id, MemoryPatch(
                action="add", target="notes", content=data["content"],
            ))
            return data["content"]

        if action == "replace" and data.get("old_text") and data.get("content"):
            patch_core_memory(user_id, MemoryPatch(
                action="replace", target="notes",
                old_text=data["old_text"], content=data["content"],
            ))
            return data["content"]

        return None

    except (json.JSONDecodeError, ValueError, IndexError):
        return None
    except Exception:
        return None
