import { emit } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, test, vi } from "vitest";
import {
  PINBOARD_WINDOW_UNPINNED_EVENT,
  emitPinboardWindowUnpinned,
  syncPinnedNoteIds,
  pinboardSurfaceModeUnpinNoteId,
} from "./tileWindowEvents";

vi.mock("@tauri-apps/api/event", () => ({
  emit: vi.fn(),
}));

describe("pinboard window events", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  test("removes a note from the pinned set when its pinboard becomes a pad", () => {
    const next = syncPinnedNoteIds(new Set(["note-1", "note-2"]), "note-1", false);

    expect([...next]).toEqual(["note-2"]);
  });

  test("detects only pinboard to pad transitions as unpin events", () => {
    expect(pinboardSurfaceModeUnpinNoteId("pinboard", "pad", "note-1")).toBe("note-1");
    expect(pinboardSurfaceModeUnpinNoteId("pad", "pinboard", "note-1")).toBeNull();
    expect(pinboardSurfaceModeUnpinNoteId("pinboard", "pinboard", "note-1")).toBeNull();
    expect(pinboardSurfaceModeUnpinNoteId("pinboard", "pad", "")).toBeNull();
  });

  test("emits a global unpin event for the main window", async () => {
    await emitPinboardWindowUnpinned("note-1");

    expect(emit).toHaveBeenCalledWith(PINBOARD_WINDOW_UNPINNED_EVENT, "note-1");
  });
});
