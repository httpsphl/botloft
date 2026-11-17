import process from "node:process";
import { fileURLToPath } from "node:url";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { searchForWorkspaceRoot } from "vite";
import { defineConfig } from "vitest/config";

// Set by `tauri dev` when targeting a remote device; unused on desktop.
const host = process.env.TAURI_DEV_HOST;

// Minifying drops comments; the libraries' license notices stay, as GSAP's
// license asks.
const notices = { comments: { legal: true } };

// `vite build --mode setup` builds the setup window alone (setup.html,
// spec 15.7) into dist-setup, which the botloft-setup crate embeds.
// `vite build --mode phone` builds the phone's page (src/phone, spec 28.7)
// into dist-phone, which the account server serves at /m.
// `--mode site` is the public site (src/site, spec 39), which the account
// server serves at /; `pnpm build:site` (scripts/site.mjs) builds it into
// dist-site with a page per language.
export default defineConfig(({ mode }) => ({
  plugins: [react(), tailwindcss()],
  ...(mode === "phone" && {
    root: fileURLToPath(new URL("./src/phone", import.meta.url)),
    base: "/m/",
    publicDir: fileURLToPath(new URL("./public-phone", import.meta.url)),
  }),
  ...(mode === "site" && {
    root: fileURLToPath(new URL("./src/site", import.meta.url)),
    base: "/",
    publicDir: fileURLToPath(new URL("./public-site", import.meta.url)),
  }),
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
        // The bytes the computer and the phone must agree on (spec 28.3).
        fileURLToPath(new URL("../docs/test-vectors", import.meta.url)),
        // The site's screenshots, the same as the README's (spec 39).
        fileURLToPath(new URL("../docs/assets", import.meta.url)),
      ],
    },
  },
  build: {
    // WebView2 on Windows is evergreen Chromium.
    target: "chrome120",
    // The app's two windows: the main one and the island (spec 31.3).
    ...(mode !== "setup" &&
      mode !== "phone" &&
      mode !== "site" && {
        rollupOptions: { input: ["index.html", "island.html"], output: notices },
      }),
    ...(mode === "setup" && {
      outDir: "dist-setup",
      rollupOptions: { input: "setup.html", output: notices },
    }),
    ...(mode === "phone" && {
      target: "es2022",
      outDir: fileURLToPath(new URL("./dist-phone", import.meta.url)),
      emptyOutDir: true,
      rollupOptions: { output: notices },
    }),
    ...(mode === "site" && {
      target: "es2022",
      outDir: fileURLToPath(new URL("./dist-site", import.meta.url)),
      emptyOutDir: true,
      rollupOptions: { output: notices },
    }),
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
