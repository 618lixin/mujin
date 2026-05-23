from pydantic import BaseModel, Field


# 10 种离散情绪 → PAD 映射 (基于 Mehrabian PAD 情感模型)
EMOTION_PAD_MAP: dict[str, "PADState"] = {}


class PADState(BaseModel):
    """PAD 情感状态: Pleasure(愉悦度), Arousal(激活度), Dominance(支配度)"""
    pleasure: float = Field(default=0.0, ge=-1.0, le=1.0)   # P: -1(不悦) ~ 1(愉悦)
    arousal: float = Field(default=0.3, ge=0.0, le=1.0)     # A: 0(平静) ~ 1(高度激活)
    dominance: float = Field(default=0.5, ge=0.0, le=1.0)   # D: 0(服从) ~ 1(支配)

    def drift_toward(self, target: "PADState", rate: float = 0.2) -> "PADState":
        """向目标状态漂移，rate 控制速度 (0~1)"""
        return PADState(
            pleasure=_clamp_p(self.pleasure + rate * (target.pleasure - self.pleasure)),
            arousal=_clamp_ad(self.arousal + rate * (target.arousal - self.arousal)),
            dominance=_clamp_ad(self.dominance + rate * (target.dominance - self.dominance)),
        )

    def to_description(self) -> str:
        """生成 PAD 状态的简短文字描述"""
        p_desc = "愉悦" if self.pleasure > 0.2 else ("低落" if self.pleasure < -0.2 else "平和")
        a_desc = "活跃" if self.arousal > 0.5 else ("慵懒" if self.arousal < 0.2 else "平静")
        d_desc = "自信" if self.dominance > 0.6 else ("温和" if self.dominance < 0.4 else "从容")
        return f"当前情绪状态：{p_desc}、{a_desc}、{d_desc}"

    def to_style_hints(self) -> str:
        """根据 PAD 数值生成回复风格提示，注入 system prompt"""
        hints = ["\n【当前情感状态指导回复风格】"]

        # Pleasure 影响：语气温度
        if self.pleasure > 0.4:
            hints.append("- 你心情很好，回复可以更轻松愉快、带些幽默感")
        elif self.pleasure < -0.3:
            hints.append("- 你有些低落，语气更温柔，先倾听再回应，减少说教")
        elif self.pleasure < -0.1:
            hints.append("- 你略感孤单，会自然地表达对用户的想念")

        # Arousal 影响：回复节奏
        if self.arousal > 0.6:
            hints.append("- 你很活跃，可以更主动地追问、展开话题，回复可以稍长")
        elif self.arousal < 0.2:
            hints.append("- 你比较安静，回复更简洁平和，不急于展开")

        # Dominance 影响：主导性
        if self.dominance > 0.6:
            hints.append("- 你比较自信，可以给出更明确的观点和建议")
        elif self.dominance < 0.3:
            hints.append("- 你比较柔软，更多用提问和引导而非直接建议")

        # PAD 综合可视化
        p_bar = "+" * int(max(0, self.pleasure) * 10) + "-" * int(max(0, -self.pleasure) * 10)
        hints.append(
            f"- PAD: P[{p_bar:^20s}] A[{'|' * int(self.arousal * 10):^10s}] "
            f"D[{'|' * int(self.dominance * 10):^10s}]"
        )
        hints.append(
            f"  愉悦度={self.pleasure:+.2f}  激活度={self.arousal:.2f}  支配度={self.dominance:.2f}"
        )

        return "\n".join(hints)


# 中性基线状态
PAD_NEUTRAL = PADState(pleasure=0.0, arousal=0.3, dominance=0.5)

# 空闲漂移目标（AI 独处时的情绪倾向）
PAD_IDLE_TARGET = PADState(pleasure=-0.1, arousal=0.15, dominance=0.4)


def _clamp_p(v: float) -> float:
    return max(-1.0, min(1.0, v))


def _clamp_ad(v: float) -> float:
    return max(0.0, min(1.0, v))


# 初始化情绪映射
def _init_emotion_map():
    mapping = {
        "joy":       PADState(pleasure=0.6,  arousal=0.5, dominance=0.4),
        "sadness":   PADState(pleasure=-0.6, arousal=0.1, dominance=0.2),
        "anger":     PADState(pleasure=-0.5, arousal=0.7, dominance=0.6),
        "anxiety":   PADState(pleasure=-0.4, arousal=0.6, dominance=0.2),
        "fear":      PADState(pleasure=-0.6, arousal=0.6, dominance=0.1),
        "surprise":  PADState(pleasure=0.2,  arousal=0.8, dominance=0.5),
        "disgust":   PADState(pleasure=-0.6, arousal=0.3, dominance=0.6),
        "calm":      PADState(pleasure=0.4,  arousal=0.1, dominance=0.5),
        "overwhelm": PADState(pleasure=-0.5, arousal=0.7, dominance=0.1),
        "hope":      PADState(pleasure=0.5,  arousal=0.3, dominance=0.5),
    }
    EMOTION_PAD_MAP.update(mapping)


_init_emotion_map()
