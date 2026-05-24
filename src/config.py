from pathlib import Path
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    # LLM 配置
    llm_api_key: str = ""
    llm_base_url: str = "https://api.openai.com/v1"
    llm_model: str = "gpt-4o-mini"
    llm_cheap_model: str = "gpt-4o-mini"  # 用于情绪识别、日记生成

    # 数据目录
    data_dir: Path = Path("data")

    # 核心记忆容量（字符数）
    profile_max_chars: int = 1200
    notes_max_chars: int = 800
    capacity_warning_pct: float = 0.8  # 80% 时触发压缩警告

    # 对话历史
    max_history_turns: int = 20

    # 遗忘曲线
    forget_min_strength: float = 0.05    # 低于此强度的事件可被清理
    forget_base_stability: float = 30.0  # 基础记忆稳定天数
    forget_recall_boost: float = 0.5     # 回忆时稳定性增幅 (50%)

    # 心跳
    heartbeat_interval_minutes: int = 30         # 心跳间隔 (分钟)
    heartbeat_min_idle_minutes: int = 120        # 最短空闲才触发主动消息
    heartbeat_max_idle_minutes: int = 1440       # 最长空闲强制签到 (24h)
    heartbeat_proactive_enabled: bool = True     # 是否启用主动消息

    # 记忆检索
    memory_retrieval_enabled: bool = True        # 是否启用 LLM 判断记忆检索
    memory_retrieval_max_chars: int = 500        # 检索结果注入 prompt 的字符上限
    memory_retrieval_max_events: int = 3         # 最多检索事件数
    memory_retrieval_max_conversations: int = 3  # 最多检索对话数

    # 笔记自动更新
    notes_auto_update_interval: int = 10         # 每 N 轮触发一次笔记更新
    notes_auto_update_enabled: bool = True       # 是否启用笔记自动更新

    model_config = {"env_prefix": "GC_", "env_file": ".env"}


settings = Settings()
