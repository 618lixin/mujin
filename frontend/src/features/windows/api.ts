import { invoke } from "@tauri-apps/api/core";

export interface WindowBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export function openDiaryWindow(noteId?: string, bounds?: WindowBounds): Promise<string> {
  return invoke("open_diary_window", {
    noteId: noteId ?? null,
    bounds: bounds ?? null,
  });
}

export function openPinboardWindow(noteId: string, bounds?: WindowBounds): Promise<string> {
  return invoke("open_tile_window", { noteId, bounds: bounds ?? null });
}

export function togglePinboardWindow(noteId: string, bounds?: WindowBounds): Promise<boolean> {
  return invoke("toggle_tile_window", { noteId, bounds: bounds ?? null });
}

export function openNoteInEditor(noteId: string): Promise<void> {
  return invoke("open_note_in_editor", { noteId });
}

export function takeStartupFile(): Promise<string | null> {
  return invoke("take_startup_file");
}
