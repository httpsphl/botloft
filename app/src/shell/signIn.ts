// Opening Botloft when the owner signs in to Windows (spec 15.2): the app
// opens then when the bots start with Windows and the owner wants Botloft
// near the clock or its window open. Opened that way, the window stays
// hidden while Botloft waits near the clock, and shows if anything goes
// wrong before it connects.

import { useEffect, useState } from "react";
import type { LinkStep } from "../features/onboarding/link";
import type { Host } from "../lib/host";
import { useApp, useHost } from "../store/context";
import { prefs, usePref } from "./prefs";

export function useOpenAtSignIn(): void {
  const host = useHost();
  const startWithWindows = useApp((state) => state.settings?.startWithWindows);
  const keep = usePref(prefs.whenClosed) === "keep";
  const tray = usePref(prefs.tray);
  const openWindow = usePref(prefs.openAtSignIn);
  useEffect(() => {
    // Not read yet, or an older daemon: leave it as it is.
    if (startWithWindows === undefined) {
      return;
    }
    host.setOpenAtSignIn(startWithWindows && ((keep && tray) || openWindow)).catch(() => {});
  }, [host, startWithWindows, keep, tray, openWindow]);
}

const CALM: LinkStep["step"][] = ["checking", "installing", "connecting", "connected"];

export function useSignInWindow(host: Host, step: LinkStep["step"]): void {
  const [atSignIn, setAtSignIn] = useState(false);
  useEffect(() => {
    host.launchedAtSignIn().then(setAtSignIn, () => {});
  }, [host]);
  useEffect(() => {
    if (!atSignIn) {
      return;
    }
    const nearClock =
      prefs.whenClosed.get() === "keep" && prefs.tray.get() && !prefs.openAtSignIn.get();
    if (!nearClock || !CALM.includes(step)) {
      host.window.show().catch(() => {});
    }
  }, [host, atSignIn, step]);
}
