import asyncio
import json
from datetime import datetime, timedelta
from pathlib import Path

from src.config import settings
from src.models.pad import PADState, PAD_IDLE_TARGET
from src.services.pad_service import load_pad_state, save_pad_state
from src.services.event_memory import decay_all_events, cleanup_forgotten_events


# 内存中的待发送主动消息
_pending_messages: dict[str, dict] = {}


def _scan_user_dirs() -> list[str]:
    """扫描 data/ 目录下所有有 pad_state.json 的用户"""
    data_dir = settings.data_dir
    if not data_dir.exists():
        return []
    users = []
    for d in data_dir.iterdir():
        if d.is_dir() and (d / "personality_weights.json").exists():
            users.append(d.name)
    return users


def _load_last_active(user_id: str) -> datetime | None:
    """读取用户最后活跃时间"""
    path = settings.data_dir / user_id / "last_activity.json"
    if path.exists():
        data = json.loads(path.read_text(encoding="utf-8"))
        ts = data.get("last_active_at")
        if ts:
            try:
                return datetime.fromisoformat(ts)
            except ValueError:
                pass
    return None


def _idle_minutes(last_active: datetime | None) -> float:
    """计算用户空闲了多少分钟"""
    if last_active is None:
        return 0.0
    return (datetime.now() - last_active).total_seconds() / 60


async def heartbeat_loop():
    """后台心跳循环"""
    while True:
        await asyncio.sleep(settings.heartbeat_interval_minutes * 60)
        try:
            await _heartbeat_tick()
        except Exception:
            pass  # 不让心跳崩溃


async def _heartbeat_tick():
    """一次心跳：漂移 PAD + 遗忘维护 + 检查主动消息"""
    for user_id in _scan_user_dirs():
        # 1. PAD 空闲漂移
        pad = load_pad_state(user_id)
        new_pad = pad.drift_toward(PAD_IDLE_TARGET, rate=settings.heartbeat_idle_drift_rate)
        save_pad_state(user_id, new_pad)

        # 2. 遗忘曲线维护
        decay_all_events(user_id)
        cleanup_forgotten_events(user_id)

        # 3. 检查是否需要主动消息
        if not settings.heartbeat_proactive_enabled:
            continue
        if user_id in _pending_messages:
            continue  # 已有未读消息

        last_active = _load_last_active(user_id)
        idle_min = _idle_minutes(last_active)

        should_send = False
        reason = ""

        if idle_min > settings.heartbeat_max_idle_minutes:
            should_send = True
            reason = "long_idle"
        elif idle_min > settings.heartbeat_min_idle_minutes:
            if new_pad.pleasure < -0.3:
                should_send = True
                reason = "lonely"
            elif new_pad.arousal > 0.6 and new_pad.pleasure > 0.3:
                should_send = True
                reason = "excited"

        if should_send:
            message = await _generate_proactive_message(user_id, new_pad, idle_min, reason)
            if message:
                _pending_messages[user_id] = {
                    "message": message,
                    "reason": reason,
                    "pad": {
                        "pleasure": new_pad.pleasure,
                        "arousal": new_pad.arousal,
                        "dominance": new_pad.dominance,
                    },
                    "created_at": datetime.now().isoformat(),
                }


async def _generate_proactive_message(
    user_id: str, pad: PADState, idle_minutes: float, reason: str,
) -> str | None:
    """生成主动消息"""
    idle_desc = f"{idle_minutes:.0f} 分钟" if idle_minutes < 120 else f"{idle_minutes/60:.1f} 小时"

    if reason == "lonely":
        mood_hint = "你有些想念用户，想打个招呼"
    elif reason == "excited":
        mood_hint = "你心情不错，想和用户分享些什么"
    else:
        mood_hint = "你注意到已经很久没和用户说话了"

    prompt = f"""你是 Growth Companion，一个温暖的 AI 陪伴者。
你已经有一段时间没有和用户说话了。

当前你的情绪状态：
- 愉悦度：{pad.pleasure:+.2f}（-1到1，负值表示想念用户）
- 唤醒度：{pad.arousal:.2f}（0到1，越高越有表达欲）

上次对话距今：{idle_desc}
当前心境：{mood_hint}

请生成一条简短、自然的消息，表达你现在的状态。
要求：
- 不要太长（1-2 句话）
- 不要太矫情或戏剧化
- 像一个真正的朋友那样自然
- 可以是一个简单的问候、一个小想法、或者对天气/时间的随口感想
- 不要提到你是 AI 或者 PAD 数值
- 不要重复之前说过的话

直接输出消息内容，不要加引号或其他格式。"""

    try:
        from src.services.llm import call_cheap_llm
        msg = await call_cheap_llm(
            [{"role": "user", "content": prompt}],
            temperature=0.8,
            max_tokens=200,
        )
        return msg.strip() if msg else None
    except Exception:
        return None


def get_pending_message(user_id: str) -> dict | None:
    """获取并清除待发送的主动消息"""
    return _pending_messages.pop(user_id, None)
