from datetime import date, datetime
from pydantic import BaseModel, Field


class Diary(BaseModel):
    date: date
    content: str
    generated_at: datetime = Field(default_factory=datetime.now)
