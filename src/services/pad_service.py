import json
from pathlib import Path

from src.config import settings
from src.models.pad import PADState, PAD_NEUTRAL, EMOTION_PAD_MAP


def _pad_path(user_id: str) -> Path:
    return settings.data_dir / user_id / "pad_state.json"


def load_pad_state(user_id: str) -> PADState:
    """加载 PAD 状态，不存在则返回中性基线"""
    path = _pad_path(user_id)
    if path.exists():
        data = json.loads(path.read_text(encoding="utf-8"))
        return PADState(
            pleasure=data.get("pleasure", 0.0),
            arousal=data.get("arousal", 0.3),
            dominance=data.get("dominance", 0.5),
        )
    return PADState()


def save_pad_state(user_id: str, state: PADState) -> None:
    """持久化 PAD 状态"""
    path = _pad_path(user_id)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(state.model_dump(), ensure_ascii=False, indent=2),
        encoding="utf-8",
    )


def update_pad_from_emotions(current: PADState, emotions: list[str]) -> PADState:
    """
    根据检测到的情绪更新 PAD 状态。
    - 有情绪时：将各情绪的 PAD 目标取均值，然后向其漂移
    - 无情绪时：向中性基线缓慢回归
    """
    if not emotions:
        # 无情绪 → 向中性回归
        return current.drift_toward(PAD_NEUTRAL, rate=settings.pad_decay_rate)

    # 计算所有检测情绪的 PAD 均值
    valid = [EMOTION_PAD_MAP[e] for e in emotions if e in EMOTION_PAD_MAP]
    if not valid:
        return current.drift_toward(PAD_NEUTRAL, rate=settings.pad_decay_rate)

    target = PADState(
        pleasure=sum(s.pleasure for s in valid) / len(valid),
        arousal=sum(s.arousal for s in valid) / len(valid),
        dominance=sum(s.dominance for s in valid) / len(valid),
    )

    return current.drift_toward(target, rate=settings.pad_drift_rate)
