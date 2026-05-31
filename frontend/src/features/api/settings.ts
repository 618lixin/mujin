import { invoke } from "@tauri-apps/api/core";
import type { AiConfig } from "./types";

export function getAiConfig(): Promise<AiConfig> {
  return invoke("ai_config_get");
}

export function saveAiConfig(config: AiConfig): Promise<void> {
  return invoke("ai_config_save", { config });
}
