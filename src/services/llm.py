import httpx
from src.config import settings


async def call_llm(
    messages: list[dict],
    model: str | None = None,
    temperature: float = 0.7,
    max_tokens: int = 2048,
) -> str:
    """调用 LLM API（OpenAI 兼容格式）"""
    model = model or settings.llm_model
    async with httpx.AsyncClient(timeout=60.0) as client:
        resp = await client.post(
            f"{settings.llm_base_url}/chat/completions",
            headers={"Authorization": f"Bearer {settings.llm_api_key}"},
            json={
                "model": model,
                "messages": messages,
                "temperature": temperature,
                "max_tokens": max_tokens,
            },
        )
        resp.raise_for_status()
        return resp.json()["choices"][0]["message"]["content"]


async def call_cheap_llm(
    messages: list[dict],
    temperature: float = 0.3,
    max_tokens: int = 1024,
) -> str:
    """调用便宜模型（情绪识别、日记生成等）"""
    return await call_llm(
        messages,
        model=settings.llm_cheap_model,
        temperature=temperature,
        max_tokens=max_tokens,
    )
