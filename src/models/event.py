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
    created_at: datetime = Field(default_factory=datetime.now)
    updated_at: datetime = Field(default_factory=datetime.now)
