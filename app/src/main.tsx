import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import packageJson from "../package.json";
import { App } from "./App";
import type { Connect } from "./features/onboarding/link";
import { connect, rpcUrl } from "./lib/client";
import type { Host } from "./lib/host";
import { tauriHost } from "./lib/tauriHost";
import { prefs } from "./shell/prefs";
import { currentZoom } from "./shell/zoom";
import "./index.css";

const root = document.getElementById("root");
if (!root) {
  throw new Error("missing #root element");
}

async function props(): Promise<{ host: Host; connect: Connect }> {
  if ("__TAURI_INTERNALS__" in window) {
    const client = { name: "botloft-app", version: packageJson.version };
    return {
      host: tauriHost(),
      connect: (port, token) => connect({ url: rpcUrl(port), token, client }),
    };
  }
  if (import.meta.env.DEV) {
    const { previewProps } = await import("./dev/preview");
    return previewProps(window.location.search);
  }
  throw new Error("Botloft runs inside its desktop app");
}

props().then(({ host, connect: open }) => {
  // Before the first paint, so the window never flashes at the small size
  // nor moves when the owner asked for less motion.
  host.setZoom(currentZoom()).catch(() => {});
  if (prefs.lessMotion.get()) {
    document.documentElement.dataset.motion = "less";
  }
  createRoot(root).render(
    <StrictMode>
      <App host={host} connect={open} />
    </StrictMode>,
  );
});
