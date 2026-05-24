"""主题对比服务：收集同一主题下的事件/观察，LLM 生成跨时间对比。"""

from src.services.llm import call_cheap_llm
from src.services.event_memory import (
    get_topic,
    get_topic_links,
    query_events,
    query_observations,
)

COMPARE_PROMPT = """你是一个 AI 陪伴者，正在帮用户回顾某个主题上的变化。

主题：{topic_name}
主题描述：{topic_description}

=== 按时间排序的相关记录 ===
{timeline}

请生成一段跨时间的对比描述，展示用户在这个主题上的变化轨迹。

要求：
- 按时间顺序叙述，标注每个时间点的状态
- 诚实描述变化，不夸大
- 如果没有明显变化，说明"在这个主题上保持稳定"
- 用自然语言，不要用编号列表
- 最后用一句话总结整体变化方向

直接输出对比文本，不要加标题或格式。"""


async def generate_topic_compare(user_id: str, topic_id: str) -> str | None:
    """生成主题对比文本"""
    topic = get_topic(user_id, topic_id)
    if not topic:
        return None

    links = get_topic_links(user_id, topic_id)
    if not links:
        return "暂无足够记录生成对比。"

    # 收集关联的事件和观察
    timeline_items = []

    event_ids = [l["item_id"] for l in links if l["item_type"] == "event"]
    if event_ids:
        events = query_events(user_id, limit=50, min_importance=0.0)
        for e in events:
            if e.id in event_ids:
                timeline_items.append({
                    "date": e.created_at.strftime("%Y-%m-%d"),
                    "type": "event",
                    "content": e.content,
                    "emotions": ", ".join(e.emotions) if e.emotions else "",
                })

    obs_ids = [l["item_id"] for l in links if l["item_type"] == "observation"]
    if obs_ids:
        observations = query_observations(user_id, limit=50)
        for o in observations:
            if o["id"] in obs_ids:
                timeline_items.append({
                    "date": o["date"],
                    "type": "observation",
                    "content": o["content"],
                })

    # 按日期排序
    timeline_items.sort(key=lambda x: x["date"])

    if not timeline_items:
        return "暂无足够记录生成对比。"

    timeline_text = "\n".join(
        f"- [{item['date']}] {item['content']}"
        + (f" ({item['emotions']})" if item.get("emotions") else "")
        for item in timeline_items
    )

    prompt = COMPARE_PROMPT.format(
        topic_name=topic["name"],
        topic_description=topic.get("description", ""),
        timeline=timeline_text,
    )

    try:
        result = await call_cheap_llm(
            [{"role": "user", "content": prompt}],
            temperature=0.3,
            max_tokens=500,
        )
        return result.strip() if result else None
    except Exception:
        return None
