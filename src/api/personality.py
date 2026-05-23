from fastapi import APIRouter

from src.services.personality_service import load_weights, load_turn_counter
from src.services.personality_engine import load_last_reflection_date

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
