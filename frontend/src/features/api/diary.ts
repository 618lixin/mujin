import { invoke } from "@tauri-apps/api/core";
import type {
  DiaryEntry,
  DiaryGenerateResult,
  QuickExtractResult,
} from "./types";
import { DEFAULT_USER_ID } from "./types";

export function quickExtract(
  noteId: string,
  title: string,
  content: string,
  userId = DEFAULT_USER_ID,
): Promise<QuickExtractResult> {
  return invoke("quick_extract", {
    userId,
    noteId,
    title,
    content,
  });
}

export function generateDiary(
  date?: string,
  userId = DEFAULT_USER_ID,
): Promise<DiaryGenerateResult> {
  return invoke("ai_generate_diary", {
    userId,
    date: date ?? null,
  });
}

export function getDiaryList(
  limit = 30,
  userId = DEFAULT_USER_ID,
): Promise<DiaryEntry[]> {
  return invoke("ai_get_diary_list", {
    userId,
    limit,
  });
}

export function getDiary(
  date: string,
  userId = DEFAULT_USER_ID,
): Promise<DiaryEntry | null> {
  return invoke("ai_get_diary", {
    userId,
    date,
  });
}

export function regenerateDiary(
  date: string,
  userId = DEFAULT_USER_ID,
): Promise<DiaryGenerateResult> {
  return invoke("ai_regenerate_diary", {
    userId,
    date,
  });
}
