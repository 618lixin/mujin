from fastapi import APIRouter

from src.services.personality_service import load_weights, load_turn_counter
from src.services.personality_engine import load_last_reflection_date
from src.services.event_memory import get_personality_snapshots
from src.services.pad_service import load_pad_state

router = APIRouter(prefix="/api/personality", tags=["personality"])


@router.get("")
async def get_personality(user_id: str = "default"):
    weights = load_weights(user_id)
    turn_count = load_turn_counter(user_id)
    last_reflection = load_last_reflection_date(user_id)
    return {
        "weights": weights.weights,
        "weights_description": weights.to_description(),
        "turn_count": turn_count,
        "last_reflection": last_reflection,
    }


@router.get("/history")
async def get_personality_history(user_id: str = "default", limit: int = 50):
    return get_personality_snapshots(user_id, limit)


@router.get("/pad")
async def get_pad_state(user_id: str = "default"):
    """获取当前 PAD 情感状态"""
    state = load_pad_state(user_id)
    return {
        "pleasure": state.pleasure,
        "arousal": state.arousal,
        "dominance": state.dominance,
        "description": state.to_description(),
    }
