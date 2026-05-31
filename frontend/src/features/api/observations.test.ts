import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, test, vi } from "vitest";
import { getGrowthLines, getObservations, getProjects } from "./observations";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("observations api", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  test("wraps growth panel commands with nullable filters", async () => {
    vi.mocked(invoke).mockResolvedValue([]);

    await getObservations(undefined, 20, "u1");
    await getProjects("active", 10, "u1");
    await getGrowthLines(8, "u1");

    expect(invoke).toHaveBeenNthCalledWith(1, "ai_get_observations", {
      userId: "u1",
      category: null,
      limit: 20,
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "ai_get_projects", {
      userId: "u1",
      status: "active",
      limit: 10,
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "ai_get_growth_lines", {
      userId: "u1",
      limit: 8,
    });
  });
});
