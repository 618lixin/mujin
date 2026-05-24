from fastapi import APIRouter
from pydantic import BaseModel

from src.services.personality_service import initialize_user

router = APIRouter(prefix="/api", tags=["init"])


class InitRequest(BaseModel):
    user_id: str = "default"
    emotion_style: str = ""
    advice_preference: str = ""


@router.post("/init")
async def init_user(req: InitRequest):
    result = initialize_user(
        user_id=req.user_id,
        emotion_style=req.emotion_style,
        advice_preference=req.advice_preference,
    )
    return result
