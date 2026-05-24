import asyncio
from contextlib import asynccontextmanager
from pathlib import Path

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from fastapi.responses import FileResponse

from src.api.memory import router as memory_router
from src.api.init import router as init_router
from src.api.chat import router as chat_router
from src.api.diary import router as diary_router
from src.api.summary import router as summary_router
from src.api.chapters import router as chapters_router
from src.api.heartbeat import router as heartbeat_router
from src.api.observations import router as observations_router
from src.api.topics import router as topics_router
from src.api.projects import router as projects_router
from src.api.growth_lines import router as growth_lines_router
from src.services.heartbeat import heartbeat_loop


@asynccontextmanager
async def lifespan(app: FastAPI):
    # Startup: 启动心跳后台任务
    task = asyncio.create_task(heartbeat_loop())
    yield
    # Shutdown: 取消心跳任务
    task.cancel()


app = FastAPI(title="Growth Companion", version="0.3.0", lifespan=lifespan)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(init_router)
app.include_router(memory_router)
app.include_router(chat_router)
app.include_router(diary_router)
app.include_router(summary_router)
app.include_router(chapters_router)
app.include_router(heartbeat_router)
app.include_router(observations_router)
app.include_router(topics_router)
app.include_router(projects_router)
app.include_router(growth_lines_router)


@app.get("/")
async def index():
    return FileResponse(Path(__file__).parent.parent / "static" / "index.html")


@app.get("/health")
async def health():
    return {"status": "ok"}
