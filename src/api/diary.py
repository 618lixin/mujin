from datetime import date

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from src.services.diary import get_diary, generate_diary

router = APIRouter(prefix="/api/diary", tags=["diary"])


class GenerateDiaryRequest(BaseModel):
    target_date: str  # YYYY-MM-DD
    user_id: str = "default"


@router.get("/{target_date}")
async def read_diary(target_date: str, user_id: str = "default"):
    try:
        d = date.fromisoformat(target_date)
    except ValueError:
        raise HTTPException(status_code=400, detail="Invalid date format. Use YYYY-MM-DD.")

    content = get_diary(user_id, d)
    if content is None:
        raise HTTPException(status_code=404, detail="Diary not found")
    return {"date": target_date, "content": content}


@router.post("/generate")
async def trigger_generate(req: GenerateDiaryRequest):
    try:
        d = date.fromisoformat(req.target_date)
    except ValueError:
        raise HTTPException(status_code=400, detail="Invalid date format. Use YYYY-MM-DD.")

    result = await generate_diary(req.user_id, d)
    if result is None:
        raise HTTPException(status_code=409, detail="Diary already exists or no data for this date")
    return {"date": req.target_date, "content": result}
