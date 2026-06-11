import { emit } from "@tauri-apps/api/event";
import type { NoteSurfaceMode } from "./surfaceMode";

export const PINBOARD_WINDOW_CLOSED_EVENT = "pinboard-window-closed";
export const PINBOARD_WINDOW_UNPINNED_EVENT = "pinboard-window-unpinned";

export function syncPinnedNoteIds(
  current: Set<string>,
  noteId: string,
  pinned: boolean,
): Set<string> {
  const next = new Set(current);
  if (pinned) {
    next.add(noteId);
  } else {
    next.delete(noteId);
  }
  return next;
}

export function pinboardSurfaceModeUnpinNoteId(
  currentMode: NoteSurfaceMode,
  nextMode: NoteSurfaceMode,
  noteId: string,
): string | null {
  return currentMode === "pinboard" && nextMode === "pad" && noteId ? noteId : null;
}

export function emitPinboardWindowUnpinned(noteId: string): Promise<void> {
  return emit(PINBOARD_WINDOW_UNPINNED_EVENT, noteId);
}
