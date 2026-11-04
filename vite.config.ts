import path from "node:path";
import process from "node:process";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [react(), tailwindcss()],
  resolve: { alias: { "@": path.resolve(import.meta.dirname, "./src") } },

  // The app loads from local files, and its first view needs three.js, so splitting would not make
  // it start faster. This limit stays close to the current size to flag unexpected growth, in kB.
  build: { chunkSizeWarningLimit: 1500 },

  // Options tailored for `tauri dev` and `tauri build`:
  // 1. do not obscure Rust errors
  clearScreen: false,
  // 2. Tauri expects a fixed port, so fail if it is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    // 3. do not watch the Rust sources
    watch: { ignored: ["**/src-tauri/**", "**/crates/**", "**/target/**"] },
  },
}));
