import traceback

from fastapi import APIRouter
from pydantic import BaseModel

from src.services.chat import chat_turn

router = APIRouter(prefix="/api", tags=["chat"])


class ChatRequest(BaseModel):
    message: str
    user_id: str = "default"


@router.post("/chat")
async def chat(req: ChatRequest):
    try:
        result = await chat_turn(req.user_id, req.message)
        return result
    except Exception as e:
        traceback.print_exc()
        return {"error": str(e), "type": type(e).__name__}
