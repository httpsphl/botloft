// Entry of the setup window (setup.html, spec 15.7).

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import type { SetupHost } from "./host";
import { Setup } from "./Setup";
import "../index.css";
import "./setup.css";

const root = document.getElementById("root");
if (!root) {
  throw new Error("missing #root element");
}

async function host(): Promise<SetupHost> {
  if ("__TAURI_INTERNALS__" in window) {
    const { tauriSetupHost } = await import("./tauriSetupHost");
    return tauriSetupHost();
  }
  if (import.meta.env.DEV) {
    const { previewSetupHost } = await import("./fakeSetupHost");
    return previewSetupHost(window.location.search);
  }
  throw new Error("The Botloft setup runs inside its own window");
}

host().then((setupHost) => {
  createRoot(root).render(
    <StrictMode>
      <Setup host={setupHost} />
    </StrictMode>,
  );
});
