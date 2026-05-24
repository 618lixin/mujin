import json

from src.services.llm import call_cheap_llm

VALID_EMOTIONS = {"joy", "sadness", "anger", "anxiety", "fear", "surprise", "disgust", "calm", "overwhelm", "hope"}
VALID_EVENT_TYPES = {"conflict", "milestone", "emotion", "decision"}

EMOTION_EXTRACTION_PROMPT = """分析以下对话，提取情绪和事件信息。

用户消息：{user_message}
AI 回复：{ai_reply}

请以 JSON 格式输出（不要输出其他内容）：
{{
  "emotions": ["从 {valid_emotions} 中选择，可多个"],
  "event_type": "conflict/milestone/emotion/decision 或 null",
  "importance": 0.0到1.0之间的浮点数,
  "summary": "一句话摘要",
  "topics": ["从对话中识别的主题标签，如'职业选择'、'和父亲的关系'、'考研'、'健身'等，最多3个，日常闲聊为空数组"]
}}

评分标准：
- importance 0.0~0.3：日常闲聊
- importance 0.3~0.6：有情绪但非重大事件
- importance 0.6~0.8：明确的事件或强烈情绪
- importance 0.8~1.0：人生重大变化或情绪崩溃

主题识别规则：
- 只提取对话中明确涉及的主题，不要推测
- 主题应该是用户生活中持续出现的关注点或经历
- 如果用户换了工作，主题可以是"职业变化"或具体公司名
"""


class EmotionResult:
    def __init__(self, emotions: list[str], event_type: str | None, importance: float, summary: str, topics: list[str] | None = None):
        self.emotions = emotions
        self.event_type = event_type
        self.importance = max(0.0, min(1.0, importance))
        self.summary = summary
        self.topics = topics or []

    def to_dict(self) -> dict:
        return {
            "emotions": self.emotions,
            "event_type": self.event_type,
            "importance": self.importance,
            "summary": self.summary,
            "topics": self.topics,
        }


def _validate_emotions(raw: list[str]) -> list[str]:
    return [e for e in raw if e in VALID_EMOTIONS]


def _validate_event_type(raw: str | None) -> str | None:
    if raw is None:
        return None
    return raw if raw in VALID_EVENT_TYPES else None


async def extract_emotion(user_message: str, ai_reply: str) -> EmotionResult:
    """从对话中提取情绪和事件信息"""
    prompt = EMOTION_EXTRACTION_PROMPT.format(
        user_message=user_message,
        ai_reply=ai_reply,
        valid_emotions="/".join(sorted(VALID_EMOTIONS)),
    )

    response = await call_cheap_llm(
        [{"role": "user", "content": prompt}],
        temperature=0.1,
        max_tokens=256,
    )

    try:
        # 尝试提取 JSON（可能包含在 markdown 代码块中）
        text = response.strip()
        if text.startswith("```"):
            text = text.split("\n", 1)[1].rsplit("```", 1)[0].strip()
        data = json.loads(text)
    except (json.JSONDecodeError, IndexError):
        return EmotionResult(emotions=[], event_type=None, importance=0.0, summary="")

    return EmotionResult(
        emotions=_validate_emotions(data.get("emotions", [])),
        event_type=_validate_event_type(data.get("event_type")),
        importance=data.get("importance", 0.0),
        summary=data.get("summary", ""),
        topics=data.get("topics", []),
    )
