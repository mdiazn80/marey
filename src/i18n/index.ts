// Sets up translations. The language is read synchronously, so the first paint is already correct.
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { setTrayLabels } from "@/lib/tauri";
import { DEFAULT_LANGUAGE, resolveLanguage } from "./languages";
import en from "./locales/en.json";
import es from "./locales/es.json";

const STORAGE_KEY = "marey.language";

function readSavedLanguage(): string | null {
  try {
    return localStorage.getItem(STORAGE_KEY);
  } catch {
    return null;
  }
}

function saveLanguage(language: string) {
  try {
    localStorage.setItem(STORAGE_KEY, language);
  } catch {
    // The choice then lasts only for this run.
  }
}

/** Applies the language to what lives outside React: the document and the native tray menu. */
function applyOutsideReact(language: string) {
  document.documentElement.lang = language;
  setTrayLabels(i18n.t("tray.show"), i18n.t("tray.quit")).catch((error) =>
    console.warn("could not translate the tray menu", error),
  );
}

void i18n.use(initReactI18next).init({
  resources: { en: { translation: en }, es: { translation: es } },
  lng: resolveLanguage(readSavedLanguage()),
  fallbackLng: DEFAULT_LANGUAGE,
  // React already escapes what it renders.
  interpolation: { escapeValue: false },
});

applyOutsideReact(i18n.language);
i18n.on("languageChanged", (language) => {
  saveLanguage(language);
  applyOutsideReact(language);
});
