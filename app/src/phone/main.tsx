// The phone's page, served by the account server at /m (spec 28.7). It opens
// either from a code on the computer (the part after `#` holds what the
// phone needs to join) or as a phone that is already connected.

import { StrictMode, useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { PhoneClient } from "./client";
import { browserPlatform } from "./notices";
import { PairScreen } from "./PairScreen";
import { PhoneApp } from "./PhoneApp";
import { pairing, parseFragment } from "./pair";
import { browserStore, type Session } from "./store";
import "./phone.css";

const root = document.getElementById("root");
if (!root) {
  throw new Error("missing #root element");
}

const store = browserStore();
const origin = window.location.origin;
const sleep = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

/** The dark theme follows the phone's. */
function followTheme(): void {
  const dark = window.matchMedia("(prefers-color-scheme: dark)");
  const apply = () => {
    document.documentElement.dataset.theme = dark.matches ? "dark" : "light";
  };
  apply();
  dark.addEventListener("change", apply);
}

function clientFor(session: Session): PhoneClient {
  const client = new PhoneClient(session, {
    origin,
    webSocket: WebSocket,
    fetch: window.fetch.bind(window),
    store,
    notices: browserPlatform(),
  });
  client.start();
  return client;
}

function Boot() {
  const [session, setSession] = useState<Session | null | undefined>(undefined);
  const [client, setClient] = useState<PhoneClient | null>(null);
  // Taken once, and then taken off the address: the secret it holds is for
  // joining and must not stay in the history.
  const [fragment] = useState(() => {
    const parsed = parseFragment(window.location.hash);
    if (window.location.hash) {
      history.replaceState(null, "", window.location.pathname);
    }
    return parsed;
  });

  useEffect(() => {
    store.load().then(
      (kept) => setSession(kept),
      () => setSession(null),
    );
  }, []);

  useEffect(() => {
    if (!session) {
      return;
    }
    const started = clientFor(session);
    setClient(started);
    return () => started.stop();
  }, [session]);

  if (session === undefined) {
    return null;
  }
  if (session && client) {
    return (
      <PhoneApp
        api={client}
        onGone={() => {
          setClient(null);
          setSession(null);
        }}
      />
    );
  }
  if (session) {
    return null;
  }
  const pair = fragment
    ? pairing(fragment, { origin, fetch: window.fetch.bind(window), sleep })
    : null;
  return (
    <PairScreen
      pair={pair}
      onSession={(connected) => {
        void store.save(connected).then(() => setSession(connected));
      }}
    />
  );
}

followTheme();
createRoot(root).render(
  <StrictMode>
    <Boot />
  </StrictMode>,
);

// Only the page's own files are kept for the next visit, never what a bot asked.
if ("serviceWorker" in navigator) {
  navigator.serviceWorker.register("/m/sw.js", { scope: "/m/" }).catch(() => {});
}
