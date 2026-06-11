export type AppView = "main" | "diary" | "pinboard";

export interface AppRoute {
  view: AppView;
  noteId?: string;
}

export function getInitialRoute(url: URL = new URL(window.location.href)): AppRoute {
  return routeFromSearch(url.search);
}

export function routeFromSearch(search: string): AppRoute {
  const params = new URLSearchParams(search);
  const view = params.get("view");
  const noteId = params.get("noteId") ?? undefined;

  if (view === "diary") return noteId ? { view, noteId } : { view };
  if (view === "pinboard") return noteId ? { view, noteId } : { view };
  return { view: "main" };
}

export function buildDiaryUrl(noteId?: string): string {
  return buildUrl("diary", noteId);
}

export function buildPinboardUrl(noteId: string): string {
  return buildUrl("pinboard", noteId);
}

function buildUrl(view: AppView, noteId?: string): string {
  const params = new URLSearchParams({ view });
  if (noteId) params.set("noteId", noteId);
  return `index.html?${params.toString()}`;
}
