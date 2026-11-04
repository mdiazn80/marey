/** The languages of the UI, each with its name written in that language. */
export const LANGUAGES = [
  { code: "en", name: "English" },
  { code: "es", name: "Español" },
] as const;

export type Language = (typeof LANGUAGES)[number]["code"];

export const DEFAULT_LANGUAGE: Language = "en";

/** Returns the saved language if it is one we have, and the default otherwise. */
export function resolveLanguage(saved: string | null): Language {
  return LANGUAGES.find((language) => language.code === saved)?.code ?? DEFAULT_LANGUAGE;
}
