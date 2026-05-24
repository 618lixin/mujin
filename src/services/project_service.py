"""项目档案服务：自动归并事件为项目 + 生成项目总结。"""

import json
import uuid
from datetime import datetime

from src.services.llm import call_cheap_llm
from src.services.event_memory import (
    query_events,
    query_projects,
    add_project,
    update_project,
    get_project,
)


MERGE_PROMPT = """你是一个 AI 陪伴者，正在整理用户的事件记录。

=== 已有项目 ===
{existing_projects}

=== 未归入项目的事件 ===
{unassigned_events}

请分析这些事件，判断：
1. 哪些事件属于已有项目？（按项目 ID 分组）
2. 是否发现了新的项目？（至少 2 个事件关联才算新项目）

以 JSON 格式输出（不要输出其他内容）：
{{
  "assignments": [
    {{"event_id": "xxx", "project_id": "yyy"}}
  ],
  "new_projects": [
    {{"title": "项目标题", "description": "简述", "event_ids": ["id1", "id2"]}}
  ]
}}

如果没有需要归并的，两个数组都为空。"""


SUMMARY_PROMPT = """你是一个 AI 陪伴者，正在为用户的项目生成总结。

项目：{title}
描述：{description}

=== 关联事件（按时间） ===
{events_timeline}

请生成一段项目总结，包括：
- 项目的起因和经过
- 关键转折点
- 当前状态和可能的走向
- 用自然语言叙述，像给朋友讲一个故事

直接输出总结文本。"""


def _get_assigned_event_ids(user_id: str) -> set[str]:
    """获取已被项目关联的事件 ID"""
    projects = query_projects(user_id)
    ids = set()
    for p in projects:
        ids.update(p["event_ids"])
    return ids


async def auto_merge_events(user_id: str) -> dict:
    """自动归并事件到项目，返回归并结果"""
    # 找出未被任何项目关联的事件
    all_events = query_events(user_id, limit=50, min_importance=0.3)
    assigned_ids = _get_assigned_event_ids(user_id)
    unassigned = [e for e in all_events if e.id not in assigned_ids]

    if not unassigned:
        return {"assignments": [], "new_projects": []}

    existing = query_projects(user_id)
    existing_text = "\n".join(
        f"- [{p['id']}] {p['title']}: {p['description']}"
        for p in existing
    ) if existing else "暂无项目"

    unassigned_text = "\n".join(
        f"- [{e.id}] ({e.created_at.strftime('%Y-%m-%d')}) {e.content}"
        for e in unassigned
    )

    prompt = MERGE_PROMPT.format(
        existing_projects=existing_text,
        unassigned_events=unassigned_text,
    )

    try:
        response = await call_cheap_llm(
            [{"role": "user", "content": prompt}],
            temperature=0.2,
            max_tokens=512,
        )
        text = response.strip()
        if text.startswith("```"):
            text = text.split("\n", 1)[1].rsplit("```", 1)[0].strip()
        data = json.loads(text)
    except (json.JSONDecodeError, IndexError):
        return {"assignments": [], "new_projects": []}

    # 执行归并
    assignments_done = []

    # 归入已有项目
    for a in data.get("assignments", []):
        pid = a.get("project_id")
        eid = a.get("event_id")
        if not pid or not eid:
            continue
        proj = get_project(user_id, pid)
        if proj:
            new_ids = list(set(proj["event_ids"] + [eid]))
            update_project(user_id, pid, event_ids=new_ids)
            assignments_done.append(a)

    # 创建新项目
    new_created = []
    for np in data.get("new_projects", []):
        title = np.get("title", "")
        event_ids = np.get("event_ids", [])
        if not title or len(event_ids) < 2:
            continue
        proj_id = str(uuid.uuid4())[:8]
        now = datetime.now().isoformat()
        add_project(
            user_id, proj_id, title,
            description=np.get("description", ""),
            event_ids=event_ids,
            start_date=now,
            end_date=now,
        )
        new_created.append({"id": proj_id, "title": title})

    return {"assignments": assignments_done, "new_projects": new_created}


async def generate_project_summary(user_id: str, project_id: str) -> str | None:
    """为项目生成 AI 总结"""
    proj = get_project(user_id, project_id)
    if not proj:
        return None

    events = query_events(user_id, limit=50, min_importance=0.0)
    proj_events = [e for e in events if e.id in proj["event_ids"]]
    proj_events.sort(key=lambda e: e.created_at)

    if not proj_events:
        return "暂无关联事件。"

    timeline_text = "\n".join(
        f"- [{e.created_at.strftime('%Y-%m-%d')}] {e.content}"
        for e in proj_events
    )

    prompt = SUMMARY_PROMPT.format(
        title=proj["title"],
        description=proj.get("description", ""),
        events_timeline=timeline_text,
    )

    try:
        result = await call_cheap_llm(
            [{"role": "user", "content": prompt}],
            temperature=0.3,
            max_tokens=500,
        )
        summary = result.strip() if result else None
        if summary:
            update_project(user_id, project_id, summary=summary)
        return summary
    except Exception:
        return None
