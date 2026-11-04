import { describe, expect, it } from "vitest";
import { DEFAULT_LANGUAGE, resolveLanguage } from "./languages";
import en from "./locales/en.json";
import es from "./locales/es.json";

/** Lists every string as a `dotted.key` and its value. */
function flatten(strings: object, prefix = ""): [string, string][] {
  return Object.entries(strings).flatMap(([key, value]) =>
    typeof value === "string"
      ? [[`${prefix}${key}`, value] as [string, string]]
      : flatten(value as object, `${prefix}${key}.`),
  );
}

const placeholders = (text: string) => (text.match(/{{\w+}}/g) ?? []).sort();

describe("translations", () => {
  const english = new Map(flatten(en));
  const spanish = new Map(flatten(es));

  it("have the same keys in every language", () => {
    expect([...spanish.keys()].sort()).toEqual([...english.keys()].sort());
  });

  it("have no empty strings", () => {
    for (const [key, value] of [...english, ...spanish]) expect(value, key).not.toBe("");
  });

  it("use the same placeholders in every language", () => {
    for (const [key, value] of english) {
      expect(placeholders(spanish.get(key) ?? ""), key).toEqual(placeholders(value));
    }
  });
});

describe("resolveLanguage", () => {
  it("keeps a supported language", () => {
    expect(resolveLanguage("es")).toBe("es");
  });

  it("defaults to English", () => {
    expect(DEFAULT_LANGUAGE).toBe("en");
    expect(resolveLanguage(null)).toBe("en");
    expect(resolveLanguage("fr")).toBe("en");
  });
});
