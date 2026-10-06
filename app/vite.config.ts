import process from "node:process";
import { fileURLToPath } from "node:url";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { searchForWorkspaceRoot } from "vite";
import { defineConfig } from "vitest/config";

// Set by `tauri dev` when targeting a remote device; unused on desktop.
const host = process.env.TAURI_DEV_HOST;

// `vite build --mode setup` builds the setup window alone (setup.html,
// spec 15.7) into dist-setup, which the botloft-setup crate embeds.
export default defineConfig(({ mode }) => ({
  plugins: [react(), tailwindcss()],
  // Keep Rust compiler errors visible in the terminal.
  clearScreen: false,
  server: {
    // Tauri expects a fixed port (devUrl in tauri.conf.json).
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
    // The fake daemon's drafts use the daemon's own cursor script, so its
    // folder is the one served from outside the app (a `?raw` import is
    // checked with its query, which a single file path never matches).
    fs: {
      allow: [
        searchForWorkspaceRoot(process.cwd()),
        fileURLToPath(new URL("../crates/botloftd/src/screens", import.meta.url)),
        // The sheets of the Bot agency, which a test reads (spec 26.2).
        fileURLToPath(new URL("../crates/botloftd/catalog", import.meta.url)),
      ],
    },
  },
  build: {
    // WebView2 on Windows is evergreen Chromium.
    target: "chrome120",
    ...(mode === "setup" && { outDir: "dist-setup", rollupOptions: { input: "setup.html" } }),
  },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.{ts,tsx}"],
    // The mascot test reads its moods, flame and moment frames as text, and
    // the messages test the color tokens; other CSS stays empty.
    css: {
      include: [/mascot(-(flame|cheer|wake|sleep|glance))?\.css/, /src\/index\.css/],
    },
  },
}));
