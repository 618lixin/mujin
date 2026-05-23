from datetime import date, timedelta

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from src.services.diary import get_diary, generate_diary
from src.config import settings

router = APIRouter(prefix="/api/diary", tags=["diary"])


class GenerateDiaryRequest(BaseModel):
    target_date: str  # YYYY-MM-DD
    user_id: str = "default"


class BatchGenerateRequest(BaseModel):
    start_date: str
    end_date: str
    user_id: str = "default"


@router.get("")
async def list_diaries(
    user_id: str = "default",
    start_date: str | None = None,
    end_date: str | None = None,
):
    """查询日记列表"""
    diary_dir = settings.data_dir / user_id / "diaries"
    if not diary_dir.exists():
        return []

    results = []
    for f in sorted(diary_dir.glob("*.md")):
        d_str = f.stem  # YYYY-MM-DD
        try:
            d = date.fromisoformat(d_str)
        except ValueError:
            continue

        if start_date and d_str < start_date:
            continue
        if end_date and d_str > end_date:
            continue

        content = f.read_text(encoding="utf-8")
        results.append({
            "date": d_str,
            "summary": content[:100].replace("\n", " ") + "..." if len(content) > 100 else content,
        })

    return list(reversed(results))


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


@router.post("/{target_date}/regenerate")
async def regenerate_diary(target_date: str, user_id: str = "default"):
    """重新生成日记（覆盖已有）"""
    try:
        d = date.fromisoformat(target_date)
    except ValueError:
        raise HTTPException(status_code=400, detail="Invalid date format. Use YYYY-MM-DD.")

    # 删除已有日记文件
    diary_path = settings.data_dir / user_id / "diaries" / f"{target_date}.md"
    if diary_path.exists():
        diary_path.unlink()

    result = await generate_diary(user_id, d)
    if result is None:
        raise HTTPException(status_code=404, detail="No data for this date")
    return {"date": target_date, "content": result}


@router.post("/batch-generate")
async def batch_generate(req: BatchGenerateRequest):
    """批量补生成日期范围内的日记"""
    try:
        start = date.fromisoformat(req.start_date)
        end = date.fromisoformat(req.end_date)
    except ValueError:
        raise HTTPException(status_code=400, detail="Invalid date format")

    results = []
    current = start
    while current <= end:
        diary_path = settings.data_dir / req.user_id / "diaries" / f"{current.isoformat()}.md"
        if not diary_path.exists():
            result = await generate_diary(req.user_id, current)
            results.append({"date": current.isoformat(), "generated": result is not None})
        current += timedelta(days=1)

    return {"generated": len([r for r in results if r["generated"]]), "total_checked": (end - start).days + 1, "details": results}
