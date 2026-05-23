import json
from pathlib import Path

from src.config import settings
from src.models.personality import PersonalityWeights
from src.services.memory import save_core_memory, load_core_memory
from src.services.event_memory import init_db
from src.models.memory import CoreMemory


def _user_dir(user_id: str) -> Path:
    return settings.data_dir / user_id


def _weights_path(user_id: str) -> Path:
    return _user_dir(user_id) / "personality_weights.json"


def load_weights(user_id: str) -> PersonalityWeights:
    path = _weights_path(user_id)
    if path.exists():
        data = json.loads(path.read_text(encoding="utf-8"))
        return PersonalityWeights(weights=data["weights"])
    return PersonalityWeights()


def save_weights(user_id: str, weights: PersonalityWeights) -> None:
    path = _weights_path(user_id)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps({"weights": weights.weights}, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )


def _turn_counter_path(user_id: str) -> Path:
    return _user_dir(user_id) / "turn_counter.json"


def load_turn_counter(user_id: str) -> int:
    path = _turn_counter_path(user_id)
    if path.exists():
        return json.loads(path.read_text(encoding="utf-8")).get("count", 0)
    return 0


def save_turn_counter(user_id: str, count: int) -> None:
    path = _turn_counter_path(user_id)
    path.write_text(json.dumps({"count": count}), encoding="utf-8")


def _last_reflection_path(user_id: str) -> Path:
    return _user_dir(user_id) / "last_reflection.json"


def load_last_reflection_date(user_id: str) -> str | None:
    path = _last_reflection_path(user_id)
    if path.exists():
        return json.loads(path.read_text(encoding="utf-8")).get("date")
    return None


def save_last_reflection_date(user_id: str, date_str: str) -> None:
    path = _last_reflection_path(user_id)
    path.write_text(json.dumps({"date": date_str}), encoding="utf-8")


def initialize_user(
    user_id: str,
    mbti: str | None = None,
    emotion_style: str = "",
    advice_preference: str = "",
    attachment_style: str = "",
    reflection_tendency: float = 0.5,
) -> dict:
    """初始化用户：创建画像文件、权重、数据库"""
    user_dir = _user_dir(user_id)
    user_dir.mkdir(parents=True, exist_ok=True)
    (user_dir / "diaries").mkdir(exist_ok=True)

    # 生成 user_profile.md
    profile_lines = []
    if mbti:
        profile_lines.append(f"MBTI：{mbti.upper()}")
    if emotion_style:
        profile_lines.append(f"情绪表达：{emotion_style}")
    if advice_preference:
        profile_lines.append(f"建议偏好：{advice_preference}")
    if attachment_style:
        profile_lines.append(f"依恋风格：{attachment_style}")
    profile_lines.append(f"反思倾向：{reflection_tendency}")

    profile_content = "\n".join(profile_lines)[: settings.profile_max_chars]
    memory = CoreMemory(profile_content=profile_content)
    save_core_memory(user_id, memory)

    # 初始化八维权重
    weights = PersonalityWeights.from_mbti(mbti)
    save_weights(user_id, weights)

    # 初始化事件数据库
    init_db(user_id)

    # 初始化对话计数器
    save_turn_counter(user_id, 0)

    return {
        "user_id": user_id,
        "profile_chars": len(profile_content),
        "weights": weights.weights,
    }
