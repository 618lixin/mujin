import { invoke } from "@tauri-apps/api/core";
import type { GrowthLine, Observation, Project } from "./types";
import { DEFAULT_USER_ID } from "./types";

export function getObservations(
  category?: string,
  limit = 50,
  userId = DEFAULT_USER_ID,
): Promise<Observation[]> {
  return invoke("ai_get_observations", {
    userId,
    category: category || null,
    limit,
  });
}

export function getProjects(
  status?: string,
  limit = 50,
  userId = DEFAULT_USER_ID,
): Promise<Project[]> {
  return invoke("ai_get_projects", {
    userId,
    status: status || null,
    limit,
  });
}

export function getGrowthLines(limit = 50, userId = DEFAULT_USER_ID): Promise<GrowthLine[]> {
  return invoke("ai_get_growth_lines", { userId, limit });
}
