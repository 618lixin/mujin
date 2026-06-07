import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, test, vi } from "vitest";
import {
  generateLifeChapter,
  generateWeeklySummary,
  getLifeChapters,
  getWeeklySummaries,
  regenerateWeeklySummary,
  updateLifeChapter,
  updateWeeklySummary,
} from "./growthReview";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("growth review api", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  test("getWeeklySummaries invokes ai_get_weekly_summaries", async () => {
    vi.mocked(invoke).mockResolvedValue([]);

    await getWeeklySummaries();

    expect(invoke).toHaveBeenCalledWith("ai_get_weekly_summaries");
  });

  test("generateWeeklySummary invokes ai_generate_weekly_summary", async () => {
    vi.mocked(invoke).mockResolvedValue({ isoYear: 2026, isoWeek: 23 });

    await generateWeeklySummary(2026, 23);

    expect(invoke).toHaveBeenCalledWith("ai_generate_weekly_summary", {
      userId: "default",
      isoYear: 2026,
      isoWeek: 23,
    });
  });

  test("regenerateWeeklySummary invokes ai_regenerate_weekly_summary", async () => {
    vi.mocked(invoke).mockResolvedValue({ isoYear: 2026, isoWeek: 23 });

    await regenerateWeeklySummary(2026, 23, "u1");

    expect(invoke).toHaveBeenCalledWith("ai_regenerate_weekly_summary", {
      userId: "u1",
      isoYear: 2026,
      isoWeek: 23,
    });
  });

  test("updateWeeklySummary invokes ai_update_weekly_summary", async () => {
    vi.mocked(invoke).mockResolvedValue({ noteId: "n1" });

    await updateWeeklySummary(2026, 23, "# edited");

    expect(invoke).toHaveBeenCalledWith("ai_update_weekly_summary", {
      isoYear: 2026,
      isoWeek: 23,
      content: "# edited",
    });
  });

  test("getLifeChapters invokes ai_get_life_chapters", async () => {
    vi.mocked(invoke).mockResolvedValue([]);

    await getLifeChapters();

    expect(invoke).toHaveBeenCalledWith("ai_get_life_chapters");
  });

  test("generateLifeChapter passes null title by default", async () => {
    vi.mocked(invoke).mockResolvedValue({ noteId: "n1" });

    await generateLifeChapter("2026-06-01", "2026-06-30");

    expect(invoke).toHaveBeenCalledWith("ai_generate_life_chapter", {
      userId: "default",
      startDate: "2026-06-01",
      endDate: "2026-06-30",
      title: null,
    });
  });

  test("generateLifeChapter passes editable title when provided", async () => {
    vi.mocked(invoke).mockResolvedValue({ noteId: "n1" });

    await generateLifeChapter("2026-06-01", "2026-06-30", "June Chapter", "u1");

    expect(invoke).toHaveBeenCalledWith("ai_generate_life_chapter", {
      userId: "u1",
      startDate: "2026-06-01",
      endDate: "2026-06-30",
      title: "June Chapter",
    });
  });

  test("updateLifeChapter invokes ai_update_life_chapter", async () => {
    vi.mocked(invoke).mockResolvedValue({ noteId: "n1" });

    await updateLifeChapter("n1", "Edited title", "# edited");

    expect(invoke).toHaveBeenCalledWith("ai_update_life_chapter", {
      noteId: "n1",
      title: "Edited title",
      content: "# edited",
    });
  });
});
