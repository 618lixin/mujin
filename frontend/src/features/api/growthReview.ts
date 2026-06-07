import { invoke } from "@tauri-apps/api/core";
import type {
  LifeChapterEntry,
  LifeChapterGenerateResult,
  LifeChapterUpdateResult,
  WeeklySummaryEntry,
  WeeklySummaryGenerateResult,
  WeeklySummaryUpdateResult,
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

export function updateWeeklySummary(
  isoYear: number,
  isoWeek: number,
  content: string,
): Promise<WeeklySummaryUpdateResult> {
  return invoke("ai_update_weekly_summary", {
    isoYear,
    isoWeek,
    content,
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

export function updateLifeChapter(
  noteId: string,
  title: string,
  content: string,
): Promise<LifeChapterUpdateResult> {
  return invoke("ai_update_life_chapter", {
    noteId,
    title,
    content,
  });
}
