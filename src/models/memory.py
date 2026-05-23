from typing import Literal
from pydantic import BaseModel, Field


class CoreMemory(BaseModel):
    profile_content: str = ""
    notes_content: str = ""
    profile_max_chars: int = 1200
    notes_max_chars: int = 800

    @property
    def profile_usage(self) -> int:
        return len(self.profile_content)

    @property
    def notes_usage(self) -> int:
        return len(self.notes_content)

    @property
    def profile_pct(self) -> float:
        return self.profile_usage / self.profile_max_chars if self.profile_max_chars else 0

    @property
    def notes_pct(self) -> float:
        return self.notes_usage / self.notes_max_chars if self.notes_max_chars else 0

    @property
    def profile_near_limit(self) -> bool:
        return self.profile_pct >= 0.8

    @property
    def notes_near_limit(self) -> bool:
        return self.notes_pct >= 0.8


class MemoryPatch(BaseModel):
    action: Literal["add", "replace", "remove"]
    target: Literal["profile", "notes"]
    content: str
    old_text: str | None = None  # for replace/remove
