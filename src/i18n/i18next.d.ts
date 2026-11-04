import "i18next";
import type en from "./locales/en.json";

// Makes `t` accept only the keys that exist in the English strings.
declare module "i18next" {
  interface CustomTypeOptions {
    defaultNS: "translation";
    resources: { translation: typeof en };
  }
}
