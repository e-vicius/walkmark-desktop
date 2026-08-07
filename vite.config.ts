import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import path from "node:path";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  resolve: {
    alias: {
      $lib: path.resolve("./src/lib"),
    },
  },
  optimizeDeps: {
    // These ship .svelte sources; esbuild cannot pre-bundle them.
    exclude: ["@iconify/svelte", "bits-ui", "svelte-sonner", "mode-watcher"],
  },
  ssr: {
    noExternal: ["@iconify/svelte", "bits-ui", "svelte-sonner", "mode-watcher"],
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  build: {
    target: ["es2022", "safari16"],
    sourcemap: false,
  },
});
