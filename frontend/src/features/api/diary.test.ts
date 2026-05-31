import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, test, vi } from "vitest";
import {
  generateDiary,
  getDiary,
  getDiaryList,
  quickExtract,
  regenerateDiary,
} from "./diary";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("diary api", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  test("triggers quick extraction for a saved note", async () => {
    vi.mocked(invoke).mockResolvedValue({ extracted: true, eventId: "evt1" });

    await quickExtract("note-1", "Title", "Content", "u1");

    expect(invoke).toHaveBeenCalledWith("quick_extract", {
      userId: "u1",
      noteId: "note-1",
      title: "Title",
      content: "Content",
    });
  });

  test("generateDiary invokes ai_generate_diary with default user", async () => {
    vi.mocked(invoke).mockResolvedValue({
      date: "2026-05-30",
      noteId: "note-1",
      title: "2026-05-30",
      content: "# 2026-05-30\n\n今天...",
      sourceEventCount: 2,
      sourceTurnCount: 5,
      regenerated: false,
    });

    await generateDiary("2026-05-30");

    expect(invoke).toHaveBeenCalledWith("ai_generate_diary", {
      userId: "default",
      date: "2026-05-30",
    });
  });

  test("generateDiary passes null date when not provided", async () => {
    vi.mocked(invoke).mockResolvedValue({
      date: "2026-05-30",
      noteId: "note-1",
      title: "2026-05-30",
      content: "# 2026-05-30\n\n...",
      sourceEventCount: 0,
      sourceTurnCount: 0,
      regenerated: false,
    });

    await generateDiary();

    expect(invoke).toHaveBeenCalledWith("ai_generate_diary", {
      userId: "default",
      date: null,
    });
  });

  test("getDiaryList invokes ai_get_diary_list", async () => {
    vi.mocked(invoke).mockResolvedValue([]);

    await getDiaryList(30);

    expect(invoke).toHaveBeenCalledWith("ai_get_diary_list", {
      userId: "default",
      limit: 30,
    });
  });

  test("getDiary invokes ai_get_diary", async () => {
    vi.mocked(invoke).mockResolvedValue(null);

    await getDiary("2026-05-30");

    expect(invoke).toHaveBeenCalledWith("ai_get_diary", {
      userId: "default",
      date: "2026-05-30",
    });
  });

  test("regenerateDiary invokes ai_regenerate_diary", async () => {
    vi.mocked(invoke).mockResolvedValue({
      date: "2026-05-30",
      noteId: "note-1",
      title: "2026-05-30",
      content: "# 2026-05-30\n\n重新生成...",
      sourceEventCount: 2,
      sourceTurnCount: 5,
      regenerated: true,
    });

    await regenerateDiary("2026-05-30");

    expect(invoke).toHaveBeenCalledWith("ai_regenerate_diary", {
      userId: "default",
      date: "2026-05-30",
    });
  });
});
