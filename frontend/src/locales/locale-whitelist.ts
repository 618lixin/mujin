export const SUPPORTED_LOCALES = ["zh-CN"] as const;

export type SupportedLocale = (typeof SUPPORTED_LOCALES)[number];

export const DEFAULT_LOCALE: SupportedLocale = "zh-CN";

const SUPPORTED_LOCALE_SET = new Set<string>(SUPPORTED_LOCALES);

const LOCALE_ALIASES: Record<string, SupportedLocale> = {
  zh: "zh-CN",
  "zh-cn": "zh-CN",
  "zh-hans": "zh-CN",
  "zh-sg": "zh-CN",
};

function canonicalizeLocale(locale: string): string {
  try {
    return Intl.getCanonicalLocales(locale)[0] ?? locale;
  } catch {
    return locale;
  }
}

export function normalizeLocale(locale?: string | null): SupportedLocale | null {
  if (!locale) {
    return null;
  }

  const trimmed = locale.trim();
  if (!trimmed) {
    return null;
  }

  const canonical = canonicalizeLocale(trimmed);
  if (SUPPORTED_LOCALE_SET.has(canonical)) {
    return canonical as SupportedLocale;
  }

  return LOCALE_ALIASES[canonical.toLowerCase()] ?? null;
}

export function resolveAppLocale(
  preferredLocale?: string | null,
  navigatorLocale?: string | null,
): SupportedLocale {
  return normalizeLocale(preferredLocale) ?? normalizeLocale(navigatorLocale) ?? DEFAULT_LOCALE;
}
