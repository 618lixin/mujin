import { invoke } from "@tauri-apps/api/core";
import type {
  LifeChapterEntry,
  LifeChapterGenerateResult,
  WeeklySummaryEntry,
  WeeklySummaryGenerateResult,
} from "./types";
import { DEFAULT_USER_ID } from "./types";

export function getWeeklySummaries(): Promise<WeeklySummaryEntry[]> {
  return invoke("ai_get_weekly_summaries");
}

export function generateWeeklySummary(
  isoYear: number,
  isoWeek: number,
  userId = DEFAULT_USER_ID,
): Promise<WeeklySummaryGenerateResult> {
  return invoke("ai_generate_weekly_summary", {
    userId,
    isoYear,
    isoWeek,
  });
}

export function regenerateWeeklySummary(
  isoYear: number,
  isoWeek: number,
  userId = DEFAULT_USER_ID,
): Promise<WeeklySummaryGenerateResult> {
  return invoke("ai_regenerate_weekly_summary", {
    userId,
    isoYear,
    isoWeek,
  });
}

export function getLifeChapters(): Promise<LifeChapterEntry[]> {
  return invoke("ai_get_life_chapters");
}

export function generateLifeChapter(
  startDate: string,
  endDate: string,
  title?: string,
  userId = DEFAULT_USER_ID,
): Promise<LifeChapterGenerateResult> {
  return invoke("ai_generate_life_chapter", {
    userId,
    startDate,
    endDate,
    title: title ?? null,
  });
}
