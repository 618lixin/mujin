"""LLM 判断记忆检索：在对话前判断是否需要检索历史记忆，按需注入 prompt。"""

import json

from src.config import settings
from src.services.llm import call_cheap_llm
from src.services.event_memory import (
    search_conversations,
    query_events,
    record_recall,
)


_RETRIEVAL_JUDGE_PROMPT = """判断用户消息是否需要检索历史记忆。

用户消息：{user_message}

判断标准：只有当消息明确引用了过去的对话、事件、人物、话题，或表达了需要历史上下文才能回答的问题时，才需要检索。
纯粹的闲聊、新话题、简单问候不需要检索。

严格以 JSON 格式输出（不要输出其他内容）：
{{
  "need_retrieval": true 或 false,
  "keywords": ["提取的搜索关键词，最多3个"]
}}

示例：
- "我之前说的那个项目怎么样了" → {{"need_retrieval": true, "keywords": ["项目"]}}
- "今天天气真好" → {{"need_retrieval": false, "keywords": []}}
- "你还记得我上次哭的原因吗" → {{"need_retrieval": true, "keywords": ["哭", "原因"]}}
- "谢谢你一直陪我" → {{"need_retrieval": false, "keywords": []}}
- "我爸最近身体不太好" → {{"need_retrieval": true, "keywords": ["爸", "身体"]}}
- "我决定辞职了" → {{"need_retrieval": false, "keywords": []}}"""


def _parse_judgment(raw: str) -> dict:
    """解析 LLM 返回的 JSON 判断结果"""
    text = raw.strip()
    if text.startswith("```"):
        text = text.split("\n", 1)[1].rsplit("```", 1)[0].strip()
    try:
        data = json.loads(text)
    except json.JSONDecodeError:
        return {"need_retrieval": False, "keywords": []}
    if not isinstance(data, dict):
        return {"need_retrieval": False, "keywords": []}
    return {
        "need_retrieval": bool(data.get("need_retrieval", False)),
        "keywords": list(data.get("keywords", []))[:3],
    }


async def _judge_retrieval_need(user_message: str) -> dict:
    """用 cheap LLM 判断是否需要检索"""
    prompt = _RETRIEVAL_JUDGE_PROMPT.format(user_message=user_message)
    try:
        response = await call_cheap_llm(
            [{"role": "user", "content": prompt}],
            temperature=0.1,
            max_tokens=128,
        )
        return _parse_judgment(response)
    except Exception:
        return {"need_retrieval": False, "keywords": []}


def _retrieve_and_format(user_id: str, keywords: list[str]) -> str | None:
    """根据关键词检索记忆并格式化为 prompt 块"""
    # 收集对话片段
    convos = []
    seen_ids = set()
    for kw in keywords:
        try:
            results = search_conversations(user_id, kw, limit=settings.memory_retrieval_max_conversations)
        except Exception:
            continue
        for r in results:
            if r["id"] not in seen_ids:
                seen_ids.add(r["id"])
                convos.append(r)

    # 收集事件
    events = query_events(
        user_id,
        limit=settings.memory_retrieval_max_events,
        min_importance=0.4,
        min_strength=0.1,
    )

    if not convos and not events:
        return None

    # 格式化
    lines = ["\n================================================", "RETRIEVED MEMORIES (相关记忆)", "================================================"]

    for e in events:
        date_str = e.created_at.strftime("%m-%d") if e.created_at else "??"
        lines.append(f"[事件] {date_str}: {e.content} (重要度: {e.importance:.1f})")
        # 强化被检索到的记忆
        try:
            record_recall(user_id, e.id)
        except Exception:
            pass

    for c in convos:
        date_str = c.get("created_at", "")[5:10] if c.get("created_at") else "??"
        summary = c.get("summary", "")
        if summary:
            lines.append(f"[对话] {date_str}: {summary}")

    text = "\n".join(lines)

    # 截断到字符上限
    if len(text) > settings.memory_retrieval_max_chars:
        text = text[: settings.memory_retrieval_max_chars - 3] + "..."

    return text


async def run_memory_retrieval(user_id: str, user_message: str) -> str | None:
    """完整的记忆检索流程：判断 → 检索 → 格式化"""
    if not settings.memory_retrieval_enabled:
        return None

    try:
        judgment = await _judge_retrieval_need(user_message)
        if not judgment["need_retrieval"] or not judgment["keywords"]:
            return None
        return _retrieve_and_format(user_id, judgment["keywords"])
    except Exception:
        return None
