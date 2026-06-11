import { describe, expect, test } from "vitest";
import {
  DEFAULT_LOCALE,
  normalizeLocale,
  resolveAppLocale,
  SUPPORTED_LOCALES,
} from "./locale-whitelist";

describe("locale whitelist", () => {
  test("only exposes Simplified Chinese as a supported locale", () => {
    expect(SUPPORTED_LOCALES).toEqual(["zh-CN"]);
    expect(DEFAULT_LOCALE).toBe("zh-CN");
  });

  test("normalizes Simplified Chinese aliases to the only supported locale", () => {
    expect(normalizeLocale("zh-CN")).toBe("zh-CN");
    expect(normalizeLocale("zh-cn")).toBe("zh-CN");
    expect(normalizeLocale("zh")).toBe("zh-CN");
    expect(normalizeLocale("zh-Hans")).toBe("zh-CN");
  });

  test("returns null for non-Simplified-Chinese locales", () => {
    expect(normalizeLocale("en-US")).toBeNull();
    expect(normalizeLocale("en-GB")).toBeNull();
    expect(normalizeLocale("zh-HK")).toBeNull();
    expect(normalizeLocale("zh-TW")).toBeNull();
    expect(normalizeLocale("fr-FR")).toBeNull();
    expect(normalizeLocale("")).toBeNull();
    expect(normalizeLocale(undefined)).toBeNull();
  });

  test("always resolves unsupported preferred or browser locales to Simplified Chinese", () => {
    expect(resolveAppLocale("en-US", "zh-CN")).toBe("zh-CN");
    expect(resolveAppLocale(undefined, "zh-HK")).toBe("zh-CN");
    expect(resolveAppLocale(undefined, "fr-FR")).toBe("zh-CN");
  });
});
