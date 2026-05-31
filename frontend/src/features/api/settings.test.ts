import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, test, vi } from "vitest";
import { getAiConfig, saveAiConfig } from "./settings";
import type { AiConfig } from "./types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("ai settings api", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  test("loads and saves AI configuration through Tauri commands", async () => {
    const config: AiConfig = {
      llmApiKey: "key",
      llmBaseUrl: "https://api.example.com/v1",
      llmModel: "model",
      llmCheapModel: "cheap",
      profileMaxChars: 1200,
      notesMaxChars: 800,
      capacityWarningPct: 0.8,
      maxHistoryTurns: 20,
      forgetMinStrength: 0.05,
      forgetBaseStability: 30,
      forgetRecallBoost: 0.5,
      heartbeatIntervalMinutes: 30,
      heartbeatMinIdleMinutes: 120,
      heartbeatMaxIdleMinutes: 1440,
      heartbeatProactiveEnabled: true,
    };
    vi.mocked(invoke).mockResolvedValueOnce(config).mockResolvedValueOnce(undefined);

    await expect(getAiConfig()).resolves.toEqual(config);
    await saveAiConfig(config);

    expect(invoke).toHaveBeenNthCalledWith(1, "ai_config_get");
    expect(invoke).toHaveBeenNthCalledWith(2, "ai_config_save", { config });
  });
});
