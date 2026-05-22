import { defineConfig } from "vite";

// Tauri expects a fixed port and ignores hot-reload on certain dirs.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: {
      // Don't watch the Rust side; cargo handles it.
      ignored: ["**/src-tauri/**"],
    },
  },
  // Produce a clean, relative-path build the webview can load.
  build: {
    target: "es2022",
    minify: !process.env.TAURI_ENV_DEBUG,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
