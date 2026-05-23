import json
from datetime import date, datetime
from pathlib import Path

from src.config import settings
from src.services.llm import call_cheap_llm
from src.services.event_memory import query_events


def _user_dir(user_id: str) -> Path:
    return settings.data_dir / user_id


def _diary_path(user_id: str, d: date) -> Path:
    return _user_dir(user_id) / "diaries" / f"{d.isoformat()}.md"


def _day_data_path(user_id: str, d: date) -> Path:
    return _user_dir(user_id) / f"day_data_{d.isoformat()}.json"


def accumulate_day_data(user_id: str, emotion_result) -> None:
    """累积当日情绪/事件数据，供日记生成使用"""
    today = date.today()
    path = _day_data_path(user_id, today)
    if path.exists():
        data = json.loads(path.read_text(encoding="utf-8"))
    else:
        data = {"entries": []}

    data["entries"].append({
        "emotions": emotion_result.emotions,
        "event_type": emotion_result.event_type,
        "importance": emotion_result.importance,
        "summary": emotion_result.summary,
    })

    path.write_text(json.dumps(data, ensure_ascii=False), encoding="utf-8")


async def generate_diary(user_id: str, target_date: date) -> str | None:
    """为指定日期生成日记"""
    diary_file = _diary_path(user_id, target_date)
    if diary_file.exists():
        return None  # 已存在，不覆盖

    # 获取当日数据
    day_data_path = _day_data_path(user_id, target_date)
    if not day_data_path.exists():
        return None  # 无数据

    day_data = json.loads(day_data_path.read_text(encoding="utf-8"))
    if not day_data.get("entries"):
        return None

    # 获取当日重要事件
    date_str = target_date.isoformat()
    next_date_str = (target_date.replace(day=target_date.day + 1)).isoformat() if target_date.day < 28 else None
    events = query_events(user_id, limit=5, min_importance=0.5, start_date=date_str)
    if next_date_str:
        events = [e for e in events if e.created_at.strftime("%Y-%m-%d") == date_str]

    events_text = "\n".join(f"- {e.summary or e.content}" for e in events)
    summaries = "\n".join(f"- {e['summary']}" for e in day_data["entries"] if e.get("summary"))

    prompt = f"""根据以下数据生成一篇 {target_date.isoformat()} 的日记。

情绪和事件摘要：
{summaries}

重要事件：
{events_text or "无"}

请用 Markdown 格式生成日记，包含：
1. 日期标题
2. "今天你提到了" 段落
3. 情绪变化描述
4. "成长观察" 段落（发现一个积极的信号或变化）

语气：温暖、客观、有洞察力。不要说教，只是陪伴和见证。"""

    content = await call_cheap_llm(
        [{"role": "user", "content": prompt}],
        temperature=0.7,
        max_tokens=1024,
    )

    # 写入文件
    diary_file.parent.mkdir(parents=True, exist_ok=True)
    diary_file.write_text(content, encoding="utf-8")

    return content


async def check_and_generate_yesterday_diary(user_id: str) -> None:
    """检查并生成昨日日记"""
    yesterday = date.today().replace(day=date.today().day - 1) if date.today().day > 1 else None
    if yesterday is None:
        return
    await generate_diary(user_id, yesterday)


def get_diary(user_id: str, target_date: date) -> str | None:
    """读取指定日期的日记"""
    path = _diary_path(user_id, target_date)
    if path.exists():
        return path.read_text(encoding="utf-8")
    return None
