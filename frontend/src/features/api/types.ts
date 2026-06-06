export const DEFAULT_USER_ID = "default";

export interface ChatMessage {
  role: "user" | "assistant" | "system";
  content: string;
}

export interface ChatDaySummary {
  date: string;
  messageCount: number;
}

export interface EmotionResult {
  emotions: string[];
  eventType?: string;
  importance: number;
  summary: string;
  topics: string[];
}

export interface ChatSendResult {
  reply: string;
  emotion: EmotionResult;
  turnCount: number;
}

export interface QueryEventsParams {
  limit?: number;
  minImportance?: number;
  eventType?: string;
  startDate?: string;
  endDate?: string;
  minStrength?: number;
}

export interface EventMemory {
  id: string;
  content: string;
  emotions: string[];
  importance: number;
  eventType?: string;
  strength: number;
  stability: number;
  recallCount: number;
  lastRecalledAt?: string;
  createdAt: string;
  updatedAt: string;
}

export interface MemoryPatch {
  action: "add" | "replace" | "remove";
  target: "profile" | "notes";
  content: string;
  oldText?: string;
}

export interface CoreMemoryStats {
  content: string;
  chars: number;
  maxChars: number;
  pct: number;
  nearLimit: boolean;
}

export interface CoreMemoryResponse {
  profile: CoreMemoryStats;
  notes: CoreMemoryStats;
}

export interface Topic {
  id: string;
  name: string;
  description: string;
  firstMentioned?: string;
  lastMentioned?: string;
  mentionCount: number;
}

export interface TopicLink {
  topicId: string;
  itemId: string;
  itemType: string;
}

export interface TopicDetail {
  topicId: string;
  links: TopicLink[];
}

export interface Observation {
  id: string;
  date: string;
  content: string;
  category?: string;
  source?: string;
  createdAt: string;
}

export interface Project {
  id: string;
  title: string;
  description: string;
  status: string;
  startDate?: string;
  endDate?: string;
  eventIds: string[];
  summary: string;
  createdAt: string;
  updatedAt: string;
}

export interface GrowthLine {
  id: string;
  dimension: string;
  records: unknown[];
}

export interface PendingMessage {
  message: string;
  reason: string;
  createdAt: string;
}

export interface ConversationTurn {
  id: number;
  summary: string;
  emotions: string[];
  createdAt: string;
}

export interface AiConfig {
  llmApiKey: string;
  llmBaseUrl: string;
  llmModel: string;
  llmCheapModel: string;
  profileMaxChars: number;
  notesMaxChars: number;
  capacityWarningPct: number;
  maxHistoryTurns: number;
  forgetMinStrength: number;
  forgetBaseStability: number;
  forgetRecallBoost: number;
  heartbeatIntervalMinutes: number;
  heartbeatMinIdleMinutes: number;
  heartbeatMaxIdleMinutes: number;
  heartbeatProactiveEnabled: boolean;
}

export interface QuickExtractResult {
  extracted: boolean;
  reason?: "empty" | "below_threshold";
  eventId?: string;
  noteId?: string;
  emotion?: EmotionResult;
}

export interface DiaryEntry {
  date: string;
  noteId: string;
  title: string;
  content: string;
  createdAt: string;
  updatedAt: string;
}

export interface DiaryGenerateResult {
  date: string;
  noteId: string;
  title: string;
  content: string;
  sourceEventCount: number;
  sourceTurnCount: number;
  sourceNoteCount: number;
  regenerated: boolean;
}
