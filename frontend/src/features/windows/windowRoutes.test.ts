import { describe, expect, it } from "vitest";
import { buildDiaryUrl, buildPinboardUrl, getInitialRoute, routeFromSearch } from "./windowRoutes";

describe("window routes", () => {
  it("parses supported routes and note ids", () => {
    expect(routeFromSearch("?view=diary&noteId=abc-123")).toEqual({
      view: "diary",
      noteId: "abc-123",
    });
    expect(routeFromSearch("?view=pinboard&noteId=note-1")).toEqual({
      view: "pinboard",
      noteId: "note-1",
    });
    expect(routeFromSearch("?view=unknown")).toEqual({ view: "main" });
  });

  it("builds app urls for dynamic windows", () => {
    expect(buildDiaryUrl()).toBe("index.html?view=diary");
    expect(buildDiaryUrl("abc 123")).toBe("index.html?view=diary&noteId=abc+123");
    expect(buildPinboardUrl("note-1")).toBe("index.html?view=pinboard&noteId=note-1");
  });

  it("reads the browser location by default", () => {
    expect(getInitialRoute(new URL("https://persona-diary.test/?view=main"))).toEqual({
      view: "main",
    });
  });
});
