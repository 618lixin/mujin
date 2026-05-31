import { invoke } from "@tauri-apps/api/core";
import type {
  ConversationTurn,
  CoreMemoryResponse,
  EventMemory,
  MemoryPatch,
  PendingMessage,
  QueryEventsParams,
  Topic,
  TopicDetail,
} from "./types";
import { DEFAULT_USER_ID } from "./types";

export function getCoreMemory(userId = DEFAULT_USER_ID): Promise<CoreMemoryResponse> {
  return invoke("ai_get_core_memory", { userId });
}

export function patchCoreMemory(
  patch: MemoryPatch,
  userId = DEFAULT_USER_ID,
): Promise<CoreMemoryResponse> {
  return invoke("ai_patch_core_memory", { userId, patch });
}

export function getEvents(
  params: QueryEventsParams = {},
  userId = DEFAULT_USER_ID,
): Promise<EventMemory[]> {
  return invoke("ai_get_events", { userId, params });
}

export function deleteEvent(eventId: string, userId = DEFAULT_USER_ID): Promise<boolean> {
  return invoke("ai_delete_event", { userId, eventId });
}

export function getTopics(limit = 50, userId = DEFAULT_USER_ID): Promise<Topic[]> {
  return invoke("ai_get_topics", { userId, limit });
}

export function getTopicDetail(topicId: string, userId = DEFAULT_USER_ID): Promise<TopicDetail> {
  return invoke("ai_get_topic_detail", { userId, topicId });
}

export function maintainMemory(userId = DEFAULT_USER_ID): Promise<{
  decayedCount: number;
  cleanedCount: number;
}> {
  return invoke("ai_maintain_memory", { userId });
}

export function searchConversations(
  query: string,
  limit = 5,
  userId = DEFAULT_USER_ID,
): Promise<ConversationTurn[]> {
  return invoke("ai_search_conversations", { userId, query, limit });
}

export function getPendingMessage(userId = DEFAULT_USER_ID): Promise<PendingMessage | null> {
  return invoke("ai_get_pending_message", { userId });
}
