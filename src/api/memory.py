from fastapi import APIRouter, HTTPException

from src.models.memory import MemoryPatch
from src.services.memory import load_core_memory, patch_core_memory
from src.services.event_memory import query_events, delete_event, search_conversations, get_insights

router = APIRouter(prefix="/api/memory", tags=["memory"])


@router.get("/core")
async def get_core_memory(user_id: str = "default"):
    memory = load_core_memory(user_id)
    return {
        "profile": {
            "content": memory.profile_content,
            "chars": memory.profile_usage,
            "max_chars": memory.profile_max_chars,
            "pct": round(memory.profile_pct, 2),
            "near_limit": memory.profile_near_limit,
        },
        "notes": {
            "content": memory.notes_content,
            "chars": memory.notes_usage,
            "max_chars": memory.notes_max_chars,
            "pct": round(memory.notes_pct, 2),
            "near_limit": memory.notes_near_limit,
        },
    }


@router.patch("/core")
async def update_core_memory(patch: MemoryPatch, user_id: str = "default"):
    try:
        memory = patch_core_memory(user_id, patch)
        return {"status": "ok", "profile_chars": memory.profile_usage, "notes_chars": memory.notes_usage}
    except ValueError as e:
        raise HTTPException(status_code=409, detail=str(e))


@router.get("/events")
async def get_events(
    user_id: str = "default",
    limit: int = 20,
    min_importance: float = 0.0,
    event_type: str | None = None,
    start_date: str | None = None,
    end_date: str | None = None,
):
    events = query_events(user_id, limit, min_importance, event_type, start_date, end_date)
    return [e.model_dump(mode="json") for e in events]


@router.delete("/events/{event_id}")
async def remove_event(event_id: str, user_id: str = "default"):
    deleted = delete_event(user_id, event_id)
    if not deleted:
        raise HTTPException(status_code=404, detail="Event not found")
    return {"status": "ok"}


@router.get("/search")
async def search_memory(query: str, user_id: str = "default", limit: int = 5):
    """跨会话搜索历史对话"""
    results = search_conversations(user_id, query, limit)
    return results


@router.get("/insights")
async def get_memory_insights(
    user_id: str = "default",
    category: str | None = None,
    limit: int = 20,
):
    """查询结构化洞察"""
    return get_insights(user_id, category, limit)
