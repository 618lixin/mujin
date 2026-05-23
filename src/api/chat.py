import json
import traceback

from fastapi import APIRouter
from fastapi.responses import StreamingResponse
from pydantic import BaseModel

from src.services.chat import chat_turn, chat_turn_stream

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


@router.post("/chat/stream")
async def chat_stream(req: ChatRequest):
    async def generate():
        try:
            async for event_type, data in chat_turn_stream(req.user_id, req.message):
                if event_type == "token":
                    yield f"data: {json.dumps({'type': 'token', 'content': data}, ensure_ascii=False)}\n\n"
                elif event_type == "done":
                    yield f"data: {json.dumps({'type': 'done', 'meta': data}, ensure_ascii=False)}\n\n"
        except Exception as e:
            yield f"data: {json.dumps({'type': 'error', 'message': str(e)}, ensure_ascii=False)}\n\n"

    return StreamingResponse(generate(), media_type="text/event-stream")
