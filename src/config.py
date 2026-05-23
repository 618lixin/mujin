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

    # Reflection 触发
    reflection_turn_interval: int = 30

    model_config = {"env_prefix": "GC_", "env_file": ".env"}


settings = Settings()
