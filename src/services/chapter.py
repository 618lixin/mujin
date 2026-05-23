from datetime import date
from pathlib import Path

from src.config import settings
from src.services.llm import call_cheap_llm
from src.services.event_memory import query_events
from src.services.weekly_summary import list_summaries, get_summary


def _user_dir(user_id: str) -> Path:
    return settings.data_dir / user_id


def _chapters_dir(user_id: str) -> Path:
    return _user_dir(user_id) / "chapters"


def list_chapters(user_id: str) -> list[dict]:
    """列出所有已生成的章节"""
    ch_dir = _chapters_dir(user_id)
    if not ch_dir.exists():
        return []
    results = []
    for f in sorted(ch_dir.glob("*.md")):
        first_line = f.read_text(encoding="utf-8").split("\n")[0].lstrip("# ").strip()
        results.append({"filename": f.name, "title": first_line})
    return list(reversed(results))


def get_chapter(user_id: str, filename: str) -> str | None:
    path = _chapters_dir(user_id) / filename
    if path.exists():
        return path.read_text(encoding="utf-8")
    return None


async def generate_chapter(
    user_id: str, start_date: str, end_date: str, title: str | None = None
) -> str | None:
    """为指定时间段生成人生章节"""
    events = query_events(user_id, limit=50, min_importance=0.3, start_date=start_date, end_date=end_date)

    if not events:
        return None

    events_text = "\n".join(
        f"- [{e.created_at.strftime('%Y-%m-%d')}] {e.summary or e.content} ({', '.join(e.emotions)})"
        for e in events
    )

    title_hint = f'标题使用："{title}"' if title else "请起一个有温度的标题"

    prompt = f"""根据以下数据生成一段人生章节叙事。

时间范围：{start_date} ~ {end_date}
{title_hint}

事件记录：
{events_text}

请用 Markdown 格式生成，包含：
1. 标题（# 格式）：{title_hint}
2. "这段时光"：用 3-5 句话讲述这段时间的故事，像在写一本书的一个章节
3. "关键时刻"：列出最重要的 3-5 个事件
4. "你在变化"：描述这个人的成长和变化
5. "未完待续"：一句话展望未来

语气：像一本温暖的传记，客观但有温度。"""

    content = await call_cheap_llm(
        [{"role": "user", "content": prompt}],
        temperature=0.8,
        max_tokens=1500,
    )

    # 从内容中提取标题作为文件名
    first_line = content.split("\n")[0].lstrip("# ").strip()
    safe_name = (title or first_line or f"{start_date}-{end_date}").replace(" ", "_")[:40]
    filename = f"{safe_name}.md"

    ch_dir = _chapters_dir(user_id)
    ch_dir.mkdir(parents=True, exist_ok=True)
    (ch_dir / filename).write_text(content, encoding="utf-8")

    return content
