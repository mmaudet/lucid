import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Fenêtre unique (barre latérale) : une seule entrée index.html.
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  build: {
    outDir: "dist",
    emptyOutDir: true,
    target: "safari15",
  },
});
