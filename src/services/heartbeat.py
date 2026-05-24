import asyncio
import json
from datetime import datetime

from src.config import settings
from src.services.event_memory import decay_all_events, cleanup_forgotten_events


# 内存中的待发送主动消息
_pending_messages: dict[str, dict] = {}


def _scan_user_dirs() -> list[str]:
    """扫描 data/ 目录下所有有 user_profile.md 的用户"""
    data_dir = settings.data_dir
    if not data_dir.exists():
        return []
    users = []
    for d in data_dir.iterdir():
        if d.is_dir() and (d / "user_profile.md").exists():
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
    """一次心跳：遗忘维护 + 检查主动消息（仅时间驱动）"""
    for user_id in _scan_user_dirs():
        # 1. 遗忘曲线维护
        decay_all_events(user_id)
        cleanup_forgotten_events(user_id)

        # 2. 检查是否需要主动消息（仅时间驱动）
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
            should_send = True
            reason = "idle"

        if should_send:
            message = await _generate_proactive_message(user_id, idle_min, reason)
            if message:
                _pending_messages[user_id] = {
                    "message": message,
                    "reason": reason,
                    "created_at": datetime.now().isoformat(),
                }


async def _generate_proactive_message(
    user_id: str, idle_minutes: float, reason: str,
) -> str | None:
    """生成主动消息"""
    idle_desc = f"{idle_minutes:.0f} 分钟" if idle_minutes < 120 else f"{idle_minutes/60:.1f} 小时"

    prompt = f"""你是 Growth Companion，一个自然的 AI 朋友。
你已经有一段时间没有和用户说话了。

上次对话距今：{idle_desc}

请生成一条简短、自然的消息，打个招呼。
要求：
- 不要太长（1-2 句话）
- 不要太矫情或戏剧化
- 像一个真正的朋友那样自然
- 可以是一个简单的问候、一个小想法、或者对天气/时间的随口感想
- 不要提到你是 AI
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
