from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from src.services.chapter import list_chapters, get_chapter, generate_chapter

router = APIRouter(prefix="/api/chapters", tags=["chapters"])


class GenerateChapterRequest(BaseModel):
    start_date: str
    end_date: str
    title: str | None = None
    user_id: str = "default"


@router.get("")
async def get_all_chapters(user_id: str = "default"):
    return list_chapters(user_id)


@router.get("/{filename}")
async def get_single_chapter(filename: str, user_id: str = "default"):
    content = get_chapter(user_id, filename)
    if content is None:
        raise HTTPException(status_code=404, detail="Chapter not found")
    return {"filename": filename, "content": content}


@router.post("/generate")
async def create_chapter(req: GenerateChapterRequest):
    result = await generate_chapter(req.user_id, req.start_date, req.end_date, req.title)
    if result is None:
        raise HTTPException(status_code=404, detail="No data in this date range")
    return {"content": result}
