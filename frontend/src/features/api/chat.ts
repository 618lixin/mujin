import { invoke } from "@tauri-apps/api/core";
import type { ChatDaySummary, ChatMessage, ChatSendResult } from "./types";
import { DEFAULT_USER_ID } from "./types";

export function sendChatMessage(
  message: string,
  userId = DEFAULT_USER_ID,
): Promise<ChatSendResult> {
  return invoke("chat_send", { userId, message });
}

export function startChatStream(message: string, userId = DEFAULT_USER_ID): Promise<string> {
  return invoke("chat_stream_start", { userId, message });
}

export function getChatDays(userId = DEFAULT_USER_ID): Promise<ChatDaySummary[]> {
  return invoke("ai_get_chat_days", { userId });
}

export function getChatHistory(userId = DEFAULT_USER_ID, date?: string): Promise<ChatMessage[]> {
  return invoke("ai_get_history", { userId, date: date ?? null });
}

export function clearChatHistory(userId = DEFAULT_USER_ID, date?: string): Promise<void> {
  return invoke("ai_clear_history", { userId, date: date ?? null });
}
