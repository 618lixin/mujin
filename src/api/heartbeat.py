from fastapi import APIRouter

from src.services.heartbeat import get_pending_message
from src.services.pad_service import load_pad_state

router = APIRouter(prefix="/api", tags=["heartbeat"])


@router.get("/heartbeat")
async def heartbeat(user_id: str = "default"):
    """前端轮询：检查主动消息 + 获取 PAD 状态"""
    message = get_pending_message(user_id)
    pad_state = load_pad_state(user_id)
    return {
        "proactive_message": message,
        "pad": {
            "pleasure": pad_state.pleasure,
            "arousal": pad_state.arousal,
            "dominance": pad_state.dominance,
        },
    }
