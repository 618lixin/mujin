from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from src.services.weekly_summary import get_summary, generate_summary, list_summaries

router = APIRouter(prefix="/api/summary", tags=["summary"])


class WeeklySummaryRequest(BaseModel):
    year: int
    week: int
    user_id: str = "default"


@router.get("/weekly")
async def get_all_summaries(user_id: str = "default"):
    return list_summaries(user_id)


@router.get("/weekly/{year}/{week}")
async def get_weekly_summary(year: int, week: int, user_id: str = "default"):
    content = get_summary(user_id, year, week)
    if content is None:
        raise HTTPException(status_code=404, detail="Summary not found")
    return {"year": year, "week": week, "content": content}


@router.post("/weekly")
async def create_weekly_summary(req: WeeklySummaryRequest):
    result = await generate_summary(req.user_id, req.year, req.week)
    if result is None:
        raise HTTPException(status_code=409, detail="Summary already exists or no data for this week")
    return {"year": req.year, "week": req.week, "content": result}
