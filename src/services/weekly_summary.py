from datetime import date, timedelta
from pathlib import Path

from src.config import settings
from src.services.llm import call_cheap_llm
from src.services.event_memory import query_events


def _user_dir(user_id: str) -> Path:
    return settings.data_dir / user_id


def _summary_path(user_id: str, year: int, week: int) -> Path:
    return _user_dir(user_id) / "summaries" / f"week-{year}-W{week:02d}.md"


def list_summaries(user_id: str) -> list[dict]:
    """列出所有已生成的周报"""
    summary_dir = _user_dir(user_id) / "summaries"
    if not summary_dir.exists():
        return []
    results = []
    for f in sorted(summary_dir.glob("week-*.md")):
        # week-2026-W21.md
        name = f.stem  # week-2026-W21
        parts = name.split("-")
        if len(parts) >= 3:
            year = int(parts[1])
            week = int(parts[2].lstrip("W"))
            first_line = f.read_text(encoding="utf-8").split("\n")[0].lstrip("# ").strip()
            results.append({"year": year, "week": week, "title": first_line, "filename": f.name})
    return list(reversed(results))


def get_summary(user_id: str, year: int, week: int) -> str | None:
    path = _summary_path(user_id, year, week)
    if path.exists():
        return path.read_text(encoding="utf-8")
    return None


async def generate_summary(user_id: str, year: int, week: int) -> str | None:
    """生成指定周的周报"""
    path = _summary_path(user_id, year, week)
    if path.exists():
        return None  # 已存在

    # 计算该周的日期范围（ISO week）
    # ISO week 的周一
    jan4 = date(year, 1, 4)
    week1_monday = jan4 - timedelta(days=jan4.isoweekday() - 1)
    target_monday = week1_monday + timedelta(weeks=week - 1)
    target_sunday = target_monday + timedelta(days=6)

    # 查询该周的事件
    start_str = target_monday.isoformat()
    end_str = target_sunday.isoformat()
    events = query_events(user_id, limit=50, min_importance=0.3, start_date=start_str, end_date=end_str)

    if not events:
        return None  # 无数据

    events_text = "\n".join(
        f"- [{e.created_at.strftime('%m/%d')}] {e.summary or e.content} "
        f"({', '.join(e.emotions)}, 重要性{e.importance:.0%})"
        for e in events
    )

    few_events_note = "\n注意：本周对话较少，总结可能不完整。" if len(events) < 3 else ""

    prompt = f"""根据以下数据生成 {year} 年第 {week} 周的成长总结。

时间范围：{target_monday.isoformat()} ~ {target_sunday.isoformat()}
{few_events_note}

本周事件：
{events_text}

请用 Markdown 格式生成周报，包含：
1. 标题（# 格式）：给这一周起一个有温度的名字
2. "本周概览"：2-3 句话总结这周
3. "情绪变化"：描述情绪起伏
4. "重要事件"：列出关键事件
5. "成长观察"：发现一个积极的信号或变化

语气：温暖、客观、有洞察力。"""

    content = await call_cheap_llm(
        [{"role": "user", "content": prompt}],
        temperature=0.7,
        max_tokens=1024,
    )

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
    return content
