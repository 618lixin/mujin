from pydantic import BaseModel, Field


DIMENSIONS = ["Ti", "Te", "Fi", "Fe", "Si", "Se", "Ni", "Ne"]

# MBTI → 八维权重映射表
MBTI_WEIGHTS: dict[str, dict[str, float]] = {
    "INTJ": {"Ti": 0.5, "Te": 0.6, "Fi": 0.5, "Fe": 0.2, "Si": 0.2, "Se": 0.1, "Ni": 0.9, "Ne": 0.4},
    "INTP": {"Ti": 0.9, "Te": 0.3, "Fi": 0.3, "Fe": 0.2, "Si": 0.3, "Se": 0.1, "Ni": 0.5, "Ne": 0.7},
    "ENTJ": {"Ti": 0.4, "Te": 0.9, "Fi": 0.2, "Fe": 0.3, "Si": 0.2, "Se": 0.3, "Ni": 0.7, "Ne": 0.5},
    "ENTP": {"Ti": 0.7, "Te": 0.4, "Fi": 0.2, "Fe": 0.3, "Si": 0.1, "Se": 0.2, "Ni": 0.5, "Ne": 0.9},
    "INFJ": {"Ti": 0.4, "Te": 0.2, "Fi": 0.5, "Fe": 0.7, "Si": 0.3, "Se": 0.1, "Ni": 0.9, "Ne": 0.4},
    "INFP": {"Ti": 0.3, "Te": 0.2, "Fi": 0.9, "Fe": 0.4, "Si": 0.3, "Se": 0.1, "Ni": 0.5, "Ne": 0.7},
    "ENFJ": {"Ti": 0.2, "Te": 0.4, "Fi": 0.4, "Fe": 0.9, "Si": 0.3, "Se": 0.3, "Ni": 0.7, "Ne": 0.5},
    "ENFP": {"Ti": 0.3, "Te": 0.2, "Fi": 0.6, "Fe": 0.5, "Si": 0.1, "Se": 0.2, "Ni": 0.4, "Ne": 0.9},
    "ISTJ": {"Ti": 0.4, "Te": 0.7, "Fi": 0.3, "Fe": 0.2, "Si": 0.9, "Se": 0.2, "Ni": 0.3, "Ne": 0.2},
    "ISFJ": {"Ti": 0.3, "Te": 0.3, "Fi": 0.4, "Fe": 0.7, "Si": 0.9, "Se": 0.2, "Ni": 0.3, "Ne": 0.2},
    "ESTJ": {"Ti": 0.4, "Te": 0.9, "Fi": 0.2, "Fe": 0.3, "Si": 0.7, "Se": 0.4, "Ni": 0.3, "Ne": 0.3},
    "ESFJ": {"Ti": 0.2, "Te": 0.4, "Fi": 0.3, "Fe": 0.9, "Si": 0.7, "Se": 0.4, "Ni": 0.3, "Ne": 0.3},
    "ISTP": {"Ti": 0.9, "Te": 0.3, "Fi": 0.3, "Fe": 0.1, "Si": 0.4, "Se": 0.7, "Ni": 0.3, "Ne": 0.3},
    "ISFP": {"Ti": 0.3, "Te": 0.1, "Fi": 0.9, "Fe": 0.3, "Si": 0.4, "Se": 0.7, "Ni": 0.3, "Ne": 0.4},
    "ESTP": {"Ti": 0.5, "Te": 0.4, "Fi": 0.2, "Fe": 0.2, "Si": 0.3, "Se": 0.9, "Ni": 0.2, "Ne": 0.5},
    "ESFP": {"Ti": 0.2, "Te": 0.2, "Fi": 0.4, "Fe": 0.5, "Si": 0.3, "Se": 0.9, "Ni": 0.2, "Ne": 0.5},
}

DEFAULT_WEIGHTS = {d: 0.5 for d in DIMENSIONS}


class PersonalityWeights(BaseModel):
    weights: dict[str, float] = Field(
        default_factory=lambda: dict(DEFAULT_WEIGHTS),
        description="八维人格权重 Ti/Te/Fi/Fe/Si/Se/Ni/Ne",
    )

    def get(self, dimension: str) -> float:
        return self.weights.get(dimension, 0.5)

    def adjust(self, dimension: str, delta: float) -> None:
        """调整指定维度权重，自动 clamp 到 [0.0, 1.0]"""
        current = self.weights.get(dimension, 0.5)
        self.weights[dimension] = max(0.0, min(1.0, current + delta))

    def apply_compensation(self, adjustments: dict[str, float]) -> "PersonalityWeights":
        """生成带临时补偿的副本（不修改原始权重）"""
        compensated = self.model_copy()
        for dim, delta in adjustments.items():
            compensated.adjust(dim, delta)
        return compensated

    def to_description(self) -> str:
        """生成用于 prompt 的人格权重描述"""
        lines = ["当前 AI 陪伴者的人格功能权重："]
        for dim in DIMENSIONS:
            val = self.weights[dim]
            bar = "█" * int(val * 10) + "░" * (10 - int(val * 10))
            lines.append(f"  {dim}: {bar} {val:.2f}")
        return "\n".join(lines)

    @classmethod
    def from_mbti(cls, mbti: str | None) -> "PersonalityWeights":
        if mbti and mbti.upper() in MBTI_WEIGHTS:
            return cls(weights=dict(MBTI_WEIGHTS[mbti.upper()]))
        return cls()
