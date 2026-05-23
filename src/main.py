from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from src.api.memory import router as memory_router
from src.api.init import router as init_router
from src.api.personality import router as personality_router
from src.api.chat import router as chat_router
from src.api.diary import router as diary_router

app = FastAPI(title="Growth Companion", version="0.1.0")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(init_router)
app.include_router(memory_router)
app.include_router(personality_router)
app.include_router(chat_router)
app.include_router(diary_router)


@app.get("/health")
async def health():
    return {"status": "ok"}
