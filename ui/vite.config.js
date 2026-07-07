import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Multipage : une entrée .html par fenêtre Tauri (pas de SPA).
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  build: {
    outDir: "dist",
    emptyOutDir: true,
    target: "safari15",
    rollupOptions: {
      input: {
        index: "index.html",
        dictionary: "dictionary.html",
        journal: "journal.html",
        stats: "stats.html",
        settings: "settings.html",
      },
    },
  },
});
