from datetime import datetime
from pydantic import BaseModel, Field
from typing import Literal

EVENT_TYPES = Literal["conflict", "milestone", "emotion", "decision"]


class Event(BaseModel):
    id: str
    content: str
    emotions: list[str] = Field(default_factory=list)
    importance: float = Field(ge=0.0, le=1.0)
    event_type: EVENT_TYPES | None = None
    strength: float = Field(default=1.0, ge=0.0, le=1.0)
    stability: float = Field(default=30.0, ge=1.0)  # 记忆稳定天数
    recall_count: int = Field(default=0, ge=0)
    last_recalled_at: datetime | None = None
    created_at: datetime = Field(default_factory=datetime.now)
    updated_at: datetime = Field(default_factory=datetime.now)
