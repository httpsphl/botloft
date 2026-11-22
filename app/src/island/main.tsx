// The island's page (island.html, spec 31.3): its own connection to the
// daemon, in its own window. In `pnpm dev` without Tauri, a fake daemon
// and a desktop behind it (`island.html`).

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import packageJson from "../../package.json";
import { currentLocale } from "../i18n";
import type { BotloftApi } from "../lib/api";
import { connect, rpcUrl } from "../lib/client";
import { prefs } from "../shell/prefs";
import { onChosenElsewhere, server } from "../shell/server";
import { type IslandHost, tauriIslandHost } from "./host";
import { IslandApp } from "./IslandApp";
import "../index.css";
import "./island.css";

/** How often the island looks for the daemon while it does not run. */
const RETRY_MS = 5000;

async function tauriProps(): Promise<{ api: BotloftApi; host: IslandHost }> {
  const host = tauriIslandHost();
  const client = {
    name: "botloft-island",
    version: packageJson.version,
    locale: currentLocale(),
  };
  // A server chosen in the app: the island goes through the same tunnel
  // with the same key (spec 42.6).
  const remote = server.get();
  if (remote) {
    return { api: connect({ url: rpcUrl(remote.tunnel), token: remote.token, client }), host };
  }
  for (;;) {
    const daemon = await host.daemon().catch(() => null);
    if (daemon) {
      return { api: connect({ url: rpcUrl(daemon.port), token: daemon.token, client }), host };
    }
    await new Promise((resolve) => setTimeout(resolve, RETRY_MS));
  }
}

async function props(): Promise<{ api: BotloftApi; host: IslandHost }> {
  if ("__TAURI_INTERNALS__" in window) {
    return tauriProps();
  }
  if (import.meta.env.DEV) {
    const { islandPreview } = await import("./preview");
    return islandPreview();
  }
  throw new Error("The island runs inside the Botloft app");
}

const root = document.getElementById("root");
if (!root) {
  throw new Error("missing #root element");
}
if (prefs.lessMotion.get()) {
  document.documentElement.dataset.motion = "less";
}
// The app chose another server, or this computer: start over on it.
onChosenElsewhere(() => window.location.reload());
props().then(({ api, host }) => {
  createRoot(root).render(
    <StrictMode>
      <IslandApp api={api} host={host} />
    </StrictMode>,
  );
});
