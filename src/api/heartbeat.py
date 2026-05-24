from fastapi import APIRouter

from src.services.heartbeat import get_pending_message

router = APIRouter(prefix="/api", tags=["heartbeat"])


@router.get("/heartbeat")
async def heartbeat(user_id: str = "default"):
    """前端轮询：检查主动消息"""
    message = get_pending_message(user_id)
    return {
        "proactive_message": message,
    }
