from fastapi import APIRouter
from pydantic import BaseModel

from src.services.event_memory import query_projects, get_project, update_project
from src.services.project_service import auto_merge_events, generate_project_summary

router = APIRouter(prefix="/api/projects", tags=["projects"])


@router.get("")
async def list_projects(user_id: str = "default", status: str | None = None, limit: int = 50):
    """项目档案列表"""
    return query_projects(user_id, status=status, limit=limit)


@router.get("/{project_id}")
async def get_project_detail(project_id: str, user_id: str = "default"):
    """查看项目详情"""
    proj = get_project(user_id, project_id)
    if not proj:
        return {"error": "项目不存在"}
    return proj


@router.post("/generate")
async def generate_projects(user_id: str = "default"):
    """手动触发项目归并"""
    result = await auto_merge_events(user_id)
    return result


class ProjectPatchRequest(BaseModel):
    user_id: str = "default"
    title: str | None = None
    description: str | None = None
    status: str | None = None
    event_ids: list[str] | None = None


@router.patch("/{project_id}")
async def patch_project(project_id: str, req: ProjectPatchRequest):
    """编辑项目（手动调整关联事件）"""
    proj = get_project(req.user_id, project_id)
    if not proj:
        return {"error": "项目不存在"}

    updates = {}
    if req.title is not None:
        updates["title"] = req.title
    if req.description is not None:
        updates["description"] = req.description
    if req.status is not None:
        updates["status"] = req.status
    if req.event_ids is not None:
        updates["event_ids"] = req.event_ids

    if updates:
        update_project(req.user_id, project_id, **updates)

    return get_project(req.user_id, project_id)
