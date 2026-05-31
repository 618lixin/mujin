import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, test, vi } from "vitest";
import {
  deleteEvent,
  getCoreMemory,
  getEvents,
  getPendingMessage,
  getTopicDetail,
  getTopics,
  maintainMemory,
  patchCoreMemory,
  searchConversations,
} from "./memory";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("memory api", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  test("wraps core memory commands", async () => {
    vi.mocked(invoke).mockResolvedValue({});
    const patch = { action: "add" as const, target: "profile" as const, content: "likes tea" };

    await getCoreMemory();
    await patchCoreMemory(patch, "u1");

    expect(invoke).toHaveBeenNthCalledWith(1, "ai_get_core_memory", { userId: "default" });
    expect(invoke).toHaveBeenNthCalledWith(2, "ai_patch_core_memory", {
      userId: "u1",
      patch,
    });
  });

  test("wraps event, topic, maintenance, and search commands", async () => {
    vi.mocked(invoke).mockResolvedValue([]);

    await getEvents({ limit: 10, minImportance: 0.6 }, "u1");
    await deleteEvent("evt1", "u1");
    await getTopics(12, "u1");
    await getTopicDetail("topic1", "u1");
    await maintainMemory("u1");
    await searchConversations("career", 3, "u1");
    await getPendingMessage("u1");

    expect(invoke).toHaveBeenNthCalledWith(1, "ai_get_events", {
      userId: "u1",
      params: { limit: 10, minImportance: 0.6 },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "ai_delete_event", {
      userId: "u1",
      eventId: "evt1",
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "ai_get_topics", { userId: "u1", limit: 12 });
    expect(invoke).toHaveBeenNthCalledWith(4, "ai_get_topic_detail", {
      userId: "u1",
      topicId: "topic1",
    });
    expect(invoke).toHaveBeenNthCalledWith(5, "ai_maintain_memory", { userId: "u1" });
    expect(invoke).toHaveBeenNthCalledWith(6, "ai_search_conversations", {
      userId: "u1",
      query: "career",
      limit: 3,
    });
    expect(invoke).toHaveBeenNthCalledWith(7, "ai_get_pending_message", { userId: "u1" });
  });
});
