import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

// Tauri serves the built files from disk, so relative asset paths are required.
export default defineConfig({
  plugins: [solid()],
  base: "./",
  clearScreen: false,
  server: {
    // Not Vite's default 5173: another project on this machine holds that port
    // permanently, and a silently different port would break `tauri dev`, whose
    // devUrl must match exactly.
    port: 5273,
    strictPort: true,
  },
  build: {
    // The webview is always the macOS WKWebView shipped with the minimum
    // supported system, so there is no older engine to down-level for.
    target: "safari15",
    sourcemap: false,
    emptyOutDir: true,
  },
});
