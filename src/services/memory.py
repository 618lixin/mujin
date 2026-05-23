from pathlib import Path

from src.config import settings
from src.models.memory import CoreMemory, MemoryPatch


def _user_dir(user_id: str) -> Path:
    return settings.data_dir / user_id


def _profile_path(user_id: str) -> Path:
    return _user_dir(user_id) / "user_profile.md"


def _notes_path(user_id: str) -> Path:
    return _user_dir(user_id) / "companion_notes.md"


def load_core_memory(user_id: str) -> CoreMemory:
    """读取核心记忆，文件不存在则返回空内容"""
    profile_file = _profile_path(user_id)
    notes_file = _notes_path(user_id)

    profile_content = profile_file.read_text(encoding="utf-8") if profile_file.exists() else ""
    notes_content = notes_file.read_text(encoding="utf-8") if notes_file.exists() else ""

    return CoreMemory(
        profile_content=profile_content,
        notes_content=notes_content,
        profile_max_chars=settings.profile_max_chars,
        notes_max_chars=settings.notes_max_chars,
    )


def save_core_memory(user_id: str, memory: CoreMemory) -> None:
    """保存核心记忆到文件"""
    user_dir = _user_dir(user_id)
    user_dir.mkdir(parents=True, exist_ok=True)
    _profile_path(user_id).write_text(memory.profile_content, encoding="utf-8")
    _notes_path(user_id).write_text(memory.notes_content, encoding="utf-8")


def patch_core_memory(user_id: str, patch: MemoryPatch) -> CoreMemory:
    """执行 add/replace/remove 操作"""
    memory = load_core_memory(user_id)
    target_file = _profile_path(user_id) if patch.target == "profile" else _notes_path(user_id)
    max_chars = memory.profile_max_chars if patch.target == "profile" else memory.notes_max_chars

    current = memory.profile_content if patch.target == "profile" else memory.notes_content

    if patch.action == "add":
        new_content = (current + "\n" + patch.content).strip() if current else patch.content
    elif patch.action == "replace":
        if not patch.old_text or patch.old_text not in current:
            raise ValueError(f"old_text '{patch.old_text}' not found in content")
        new_content = current.replace(patch.old_text, patch.content)
    elif patch.action == "remove":
        if not patch.old_text or patch.old_text not in current:
            raise ValueError(f"old_text '{patch.old_text}' not found in content")
        new_content = current.replace(patch.old_text, "").strip()
    else:
        raise ValueError(f"Unknown action: {patch.action}")

    if len(new_content) > max_chars:
        raise ValueError(
            f"Content exceeds limit: {len(new_content)}/{max_chars} chars. "
            f"Current content:\n{current}"
        )

    if patch.target == "profile":
        memory.profile_content = new_content
    else:
        memory.notes_content = new_content

    save_core_memory(user_id, memory)
    return memory


def format_memory_for_prompt(memory: CoreMemory) -> str:
    """生成注入 system prompt 的冻结块格式"""
    profile_pct = int(memory.profile_pct * 100)
    notes_pct = int(memory.notes_pct * 100)

    blocks = []

    if memory.profile_content:
        blocks.append(
            f"{'=' * 48}\n"
            f"USER PROFILE (用户画像) [{profile_pct}% — {memory.profile_usage}/{memory.profile_max_chars} chars]\n"
            f"{'=' * 48}\n"
            f"{memory.profile_content}"
        )

    if memory.notes_content:
        blocks.append(
            f"{'=' * 48}\n"
            f"COMPANION NOTES (AI 笔记) [{notes_pct}% — {memory.notes_usage}/{memory.notes_max_chars} chars]\n"
            f"{'=' * 48}\n"
            f"{memory.notes_content}"
        )

    return "\n\n".join(blocks)
