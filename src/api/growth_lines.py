from fastapi import APIRouter

from src.services.event_memory import query_growth_lines, get_growth_line

router = APIRouter(prefix="/api/growth-lines", tags=["growth-lines"])


@router.get("")
async def list_growth_lines(user_id: str = "default", limit: int = 50):
    """成长线列表"""
    return query_growth_lines(user_id, limit=limit)


@router.get("/{dimension}")
async def get_growth_line_detail(dimension: str, user_id: str = "default"):
    """查看某维度成长线"""
    gl = get_growth_line(user_id, dimension)
    if not gl:
        return {"error": "该维度成长线不存在"}
    return gl
