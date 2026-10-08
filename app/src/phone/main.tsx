// The phone's page, served by the account server at /m (spec 28.7). It opens
// either from a code on the computer (the part after `#` holds what the
// phone needs to join) or as a phone that is already connected.

import { StrictMode, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { PhoneClient } from "./client";
import { LockScreen } from "./LockScreen";
import { browserPlatform } from "./notices";
import { PairScreen } from "./PairScreen";
import { PhoneApp } from "./PhoneApp";
import type { LockApi } from "./PinSettings";
import { pairing, parseFragment } from "./pair";
import { browserRecords, type Session } from "./store";
import { createVault } from "./vault";
import "./phone.css";

const root = document.getElementById("root");
if (!root) {
  throw new Error("missing #root element");
}

const vault = createVault(browserRecords());
const origin = window.location.origin;
const sleep = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));
/** How long the page may be out of sight before it locks (spec 28.13). */
const LOCK_AFTER = 60_000;
const NUDGE = "botloft.phone.pin-nudge";

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
    store: vault,
    notices: browserPlatform(),
  });
  client.start();
  return client;
}

type View = "loading" | "pair" | "locked" | "app";

function Boot() {
  const [view, setView] = useState<View>("loading");
  const [session, setSession] = useState<Session | null>(null);
  const [client, setClient] = useState<PhoneClient | null>(null);
  const live = useRef<PhoneClient | null>(null);
  live.current = client;
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
    const start = async () => {
      const state = await vault.state();
      if (state === "locked") {
        return setView("locked");
      }
      const kept = state === "open" ? await vault.load() : null;
      setSession(kept);
      setView(kept ? "app" : "pair");
    };
    start().catch(() => setView("pair"));
  }, []);

  useEffect(() => {
    if (view !== "app" || !session) {
      return;
    }
    const started = clientFor(session);
    setClient(started);
    return () => started.stop();
  }, [view, session]);

  // With a PIN on, the page locks a minute after it leaves the screen.
  const lockNow = useCallback(async () => {
    if ((await vault.state()) !== "locked") {
      return;
    }
    vault.lock();
    live.current?.stop();
    setClient(null);
    setSession(null);
    setView("locked");
  }, []);

  useEffect(() => {
    if (view !== "app") {
      return;
    }
    let timer: ReturnType<typeof setTimeout> | undefined;
    let hiddenAt: number | null = null;
    const changed = () => {
      if (document.visibilityState === "hidden") {
        hiddenAt = Date.now();
        timer = setTimeout(() => void lockNow(), LOCK_AFTER);
        return;
      }
      clearTimeout(timer);
      if (hiddenAt !== null && Date.now() - hiddenAt >= LOCK_AFTER) {
        void lockNow();
      }
      hiddenAt = null;
    };
    document.addEventListener("visibilitychange", changed);
    return () => {
      clearTimeout(timer);
      document.removeEventListener("visibilitychange", changed);
    };
  }, [view, lockNow]);

  const lock = useMemo<LockApi>(
    () => ({
      state: async () =>
        (await vault.canLock()) ? ((await vault.state()) === "locked" ? "on" : "off") : "old",
      setPin: async (pin) => {
        const current = live.current;
        if (current) {
          await vault.setPin(current.snapshot(), pin);
        }
      },
      removePin: (pin) => vault.removePin(pin),
      nudgeDismissed: () => {
        try {
          return localStorage.getItem(NUDGE) === "no";
        } catch {
          return false;
        }
      },
      dismissNudge: () => {
        try {
          localStorage.setItem(NUDGE, "no");
        } catch {
          // Asked again next time; nothing is lost.
        }
      },
    }),
    [],
  );

  if (view === "loading") {
    return null;
  }
  if (view === "locked") {
    return (
      <LockScreen
        unlock={(pin) => vault.unlock(pin)}
        forget={() => vault.clear()}
        onOpen={(opened) => {
          setSession(opened);
          setView("app");
        }}
        onGone={() => {
          setSession(null);
          setView("pair");
        }}
      />
    );
  }
  if (view === "app") {
    return client ? (
      <PhoneApp
        api={client}
        lock={lock}
        onGone={() => {
          setClient(null);
          setSession(null);
          setView("pair");
        }}
      />
    ) : null;
  }
  const pair = fragment
    ? pairing(fragment, { origin, fetch: window.fetch.bind(window), sleep })
    : null;
  return (
    <PairScreen
      pair={pair}
      onSession={(connected) => {
        void vault.save(connected).then(() => {
          setSession(connected);
          setView("app");
        });
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
