from fastapi import APIRouter

from src.services.event_memory import query_observations

router = APIRouter(prefix="/api/observations", tags=["observations"])


@router.get("")
async def list_observations(user_id: str = "default", category: str | None = None, limit: int = 20):
    """获取定性观察列表"""
    return query_observations(user_id, category=category, limit=limit)


@router.get("/{category}")
async def list_observations_by_category(category: str, user_id: str = "default", limit: int = 20):
    """按类别查看观察"""
    return query_observations(user_id, category=category, limit=limit)
