from fastapi import APIRouter
from pydantic import BaseModel, Field

from src.services.personality_service import initialize_user

router = APIRouter(prefix="/api", tags=["init"])


class InitRequest(BaseModel):
    user_id: str = "default"
    mbti: str | None = None
    emotion_style: str = ""
    advice_preference: str = ""
    attachment_style: str = ""
    reflection_tendency: float = Field(default=0.5, ge=0.0, le=1.0)


@router.post("/init")
async def init_user(req: InitRequest):
    result = initialize_user(
        user_id=req.user_id,
        mbti=req.mbti,
        emotion_style=req.emotion_style,
        advice_preference=req.advice_preference,
        attachment_style=req.attachment_style,
        reflection_tendency=req.reflection_tendency,
    )
    return result
