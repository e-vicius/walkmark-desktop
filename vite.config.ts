import { defineConfig, type Plugin } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import path from "node:path";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

/** Tailwind v4 chokes when it receives a full SFC instead of the style block. */
function svelteStyleModuleFix(): Plugin {
  const styleModule = /(?:\?|&)type=style(?:&|$)|&lang\.css/;

  function extractStyles(code: string): string | null {
    if (!code.includes("<style")) return null;
    const styles = [...code.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/gi)];
    if (styles.length === 0) return null;
    return styles.map((match) => match[1]).join("\n");
  }

  return {
    name: "svelte-style-module-fix",
    enforce: "pre",
    async load(id) {
      if (!styleModule.test(id)) return;
      const sourceId = id.split("?")[0]!;
      const { readFile } = await import("node:fs/promises");
      let code: string;
      try {
        code = await readFile(sourceId, "utf8");
      } catch {
        return;
      }
      return extractStyles(code) ?? undefined;
    },
    transform: {
      filter: { id: styleModule },
      handler(code) {
        const css = extractStyles(code);
        if (css == null) return;
        return { code: css, map: null };
      },
    },
  };
}

export default defineConfig({
  plugins: [svelte(), svelteStyleModuleFix(), tailwindcss()],
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
