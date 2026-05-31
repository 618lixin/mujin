import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, test, vi } from "vitest";
import { clearChatHistory, getChatHistory, sendChatMessage, startChatStream } from "./chat";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("chat api", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  test("sends chat messages to the Tauri command with the default user", async () => {
    vi.mocked(invoke).mockResolvedValue({ reply: "hello", emotion: {}, turnCount: 1 });

    await sendChatMessage("hi");

    expect(invoke).toHaveBeenCalledWith("chat_send", {
      userId: "default",
      message: "hi",
    });
  });

  test("starts stream and manages history commands", async () => {
    vi.mocked(invoke).mockResolvedValueOnce("stream-1").mockResolvedValueOnce([]).mockResolvedValueOnce(undefined);

    await expect(startChatStream("hi", "u1")).resolves.toBe("stream-1");
    await getChatHistory("u1");
    await clearChatHistory("u1");

    expect(invoke).toHaveBeenNthCalledWith(1, "chat_stream_start", {
      userId: "u1",
      message: "hi",
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "ai_get_history", { userId: "u1" });
    expect(invoke).toHaveBeenNthCalledWith(3, "ai_clear_history", { userId: "u1" });
  });
});
