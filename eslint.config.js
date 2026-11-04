import js from "@eslint/js";
import prettier from "eslint-config-prettier";
import reactHooks from "eslint-plugin-react-hooks";
import globals from "globals";
import tseslint from "typescript-eslint";

const PARENT_IMPORTS = { group: ["../*"], message: "Import through the @ alias." };

export default tseslint.config(
  { ignores: ["dist", "target", "src-tauri", "src/types/generated", "src/components/ui"] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["src/**/*.{ts,tsx}"],
    languageOptions: { globals: globals.browser },
    plugins: { "react-hooks": reactHooks },
    rules: reactHooks.configs.recommended.rules,
  },
  {
    // Code outside the current feature is imported through the @ alias.
    files: ["src/**/*.{ts,tsx}"],
    rules: { "no-restricted-imports": ["error", { patterns: [PARENT_IMPORTS] }] },
  },
  {
    // Tauri APIs may only be called from the typed wrappers in src/lib/tauri.
    files: ["src/**/*.{ts,tsx}"],
    ignores: ["src/lib/tauri/**"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            PARENT_IMPORTS,
            { group: ["@tauri-apps/*"], message: "Use the wrappers in src/lib/tauri." },
          ],
        },
      ],
    },
  },
  prettier,
);
