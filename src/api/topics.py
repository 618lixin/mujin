from fastapi import APIRouter

from src.services.event_memory import query_topics, get_topic, get_topic_links, query_events, query_observations
from src.services.topic_service import generate_topic_compare

router = APIRouter(prefix="/api/topics", tags=["topics"])


@router.get("")
async def list_topics(user_id: str = "default", limit: int = 50):
    """主题列表"""
    return query_topics(user_id, limit=limit)


@router.get("/{topic_id}")
async def get_topic_detail(topic_id: str, user_id: str = "default"):
    """主题详情 + 关联事件/观察"""
    topic = get_topic(user_id, topic_id)
    if not topic:
        return {"error": "主题不存在"}

    links = get_topic_links(user_id, topic_id)

    # 收集关联的事件
    event_ids = [l["item_id"] for l in links if l["item_type"] == "event"]
    events = []
    if event_ids:
        all_events = query_events(user_id, limit=50, min_importance=0.0)
        events = [
            {"id": e.id, "content": e.content, "date": e.created_at.isoformat(),
             "emotions": e.emotions, "importance": e.importance}
            for e in all_events if e.id in event_ids
        ]

    # 收集关联的观察
    obs_ids = [l["item_id"] for l in links if l["item_type"] == "observation"]
    observations = []
    if obs_ids:
        all_obs = query_observations(user_id, limit=50)
        observations = [o for o in all_obs if o["id"] in obs_ids]

    return {
        **topic,
        "events": events,
        "observations": observations,
    }


@router.get("/{topic_id}/compare")
async def compare_topic(topic_id: str, user_id: str = "default"):
    """主题对比（跨时间）"""
    topic = get_topic(user_id, topic_id)
    if not topic:
        return {"error": "主题不存在"}

    compare_text = await generate_topic_compare(user_id, topic_id)
    return {
        "topic": topic["name"],
        "compare": compare_text or "数据不足，暂时无法生成对比。",
    }
