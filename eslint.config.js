import js from "@eslint/js";
import prettier from "eslint-config-prettier";
import reactHooks from "eslint-plugin-react-hooks";
import globals from "globals";
import tseslint from "typescript-eslint";

export default tseslint.config(
  { ignores: ["dist", "target", "src-tauri", "src/types/generated"] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["src/**/*.{ts,tsx}"],
    languageOptions: { globals: globals.browser },
    plugins: { "react-hooks": reactHooks },
    rules: reactHooks.configs.recommended.rules,
  },
  {
    // Tauri APIs may only be called from the typed wrappers in src/lib/tauri.
    files: ["src/**/*.{ts,tsx}"],
    ignores: ["src/lib/tauri/**"],
    rules: {
      "no-restricted-imports": [
        "error",
        { patterns: [{ group: ["@tauri-apps/*"], message: "Use the wrappers in src/lib/tauri." }] },
      ],
    },
  },
  prettier,
);
